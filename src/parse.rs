// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::graph::{Graphs, error, join};
use crate::xml::{Document, Node};
use crate::*;
use glam::{DQuat, DVec3, EulerRot};
use std::collections::BTreeSet;

pub(crate) fn load(
    xml: &str,
    uri: &str,
    resolver: &dyn ResourceResolver,
) -> Result<Sdf, Diagnostic> {
    let mut parser = Parser {
        uri,
        xml,
        resolver,
        includes: vec![uri.into()],
    };
    let doc = Document::parse(xml).map_err(|e| Diagnostic {
        code: e.code,
        source: Source {
            uri: uri.into(),
            line: e.pos().row,
            column: e.pos().col,
        },
        message: e.to_string(),
    })?;
    let root = doc.root_element();
    parser.check_root(root)?;
    let mut worlds = vec![];
    let mut models = vec![];
    parser.allowed(root, &["world", "model"], &["version"])?;
    for child in elements(root) {
        if child.has_tag_name("world") {
            worlds.push(parser.world(child)?);
        } else {
            models.push(parser.model(child, "::", None)?);
        }
    }
    if worlds.is_empty() && models.is_empty() {
        return Err(parser.err(root, ErrorCode::Missing, "sdf requires a world or model"));
    }
    if models.len() > 1 {
        return Err(parser.err(
            root,
            ErrorCode::InvalidValue,
            "root may contain only one model",
        ));
    }
    if !worlds.is_empty() && !models.is_empty() {
        return Err(parser.err(
            root,
            ErrorCode::InvalidValue,
            "root cannot mix worlds and models",
        ));
    }
    let graphs = Graphs::build(&worlds, &models, &parser.source(root))?;
    for model in models.iter().chain(worlds.iter().flat_map(|w| &w.models)) {
        validate_model(model, &graphs)?;
    }
    Ok(Sdf {
        version: root.attribute("version").unwrap_or_default().into(),
        worlds,
        models,
        graphs,
    })
}

struct Parser<'a> {
    uri: &'a str,
    xml: &'a str,
    resolver: &'a dyn ResourceResolver,
    includes: Vec<String>,
}
impl Parser<'_> {
    fn source(&self, node: Node<'_, '_>) -> Source {
        let pos = node.document().text_pos_at(node.range().start);
        Source {
            uri: self.uri.into(),
            line: pos.row,
            column: pos.col,
        }
    }
    fn err(&self, node: Node<'_, '_>, code: ErrorCode, message: impl Into<String>) -> Diagnostic {
        error(code, &self.source(node), message)
    }
    fn check_root(&self, root: Node<'_, '_>) -> Result<(), Diagnostic> {
        if !root.has_tag_name("sdf") {
            return Err(self.err(root, ErrorCode::Xml, "root element must be sdf"));
        }
        if !matches!(
            root.attribute("version"),
            Some("1.7" | "1.8" | "1.9" | "1.10" | "1.11" | "1.12")
        ) {
            return Err(self.err(root, ErrorCode::Version, "supported versions are SDF 1.7 through 1.12; earlier conversion is not implemented"));
        }
        Ok(())
    }
    fn allowed(
        &self,
        node: Node<'_, '_>,
        children: &[&str],
        attrs: &[&str],
    ) -> Result<(), Diagnostic> {
        for attr in node.attributes() {
            if attr.namespace().is_some() || !attrs.contains(&attr.name()) {
                return Err(self.err(
                    node,
                    ErrorCode::Unsupported,
                    format!(
                        "unsupported {} attribute {}",
                        node.tag_name().name(),
                        attr.name()
                    ),
                ));
            }
        }
        for child in elements(node) {
            if child.tag_name().namespace().is_some()
                || !children.contains(&child.tag_name().name())
            {
                return Err(self.err(
                    child,
                    ErrorCode::Unsupported,
                    format!("unsupported element {}", child.tag_name().name()),
                ));
            }
        }
        // Repeated scalar/schema singleton elements must never be silently ignored.
        let repeated = [
            "world",
            "model",
            "include",
            "link",
            "frame",
            "joint",
            "collision",
            "sensor",
            "plugin",
        ];
        let mut seen = BTreeSet::new();
        for child in elements(node) {
            let name = child.tag_name().name();
            if !repeated.contains(&name) && !seen.insert(name) {
                return Err(self.err(
                    child,
                    ErrorCode::InvalidValue,
                    format!("duplicate {name} element"),
                ));
            }
        }
        Ok(())
    }
    fn attr(&self, node: Node<'_, '_>, name: &str) -> Result<String, Diagnostic> {
        node.attribute(name)
            .filter(|s| !s.trim().is_empty())
            .map(str::to_string)
            .ok_or_else(|| {
                self.err(
                    node,
                    ErrorCode::Missing,
                    format!("missing {name} attribute"),
                )
            })
    }
    fn name(&self, node: Node<'_, '_>) -> Result<String, Diagnostic> {
        let name = self.attr(node, "name")?;
        if name.contains("::") && node.document().root_element().attribute("version") == Some("1.7")
        {
            return Err(self.err(
                node,
                ErrorCode::Unsupported,
                "SDF1.7 flattened-name conversion is not implemented",
            ));
        }
        if name.contains("::") || name == "world" || name.starts_with("__") || name.trim() != name {
            return Err(self.err(node, ErrorCode::InvalidValue, "reserved or invalid name"));
        }
        Ok(name)
    }
    fn text(&self, node: Node<'_, '_>, name: &str) -> Result<String, Diagnostic> {
        let value = child(node, name)
            .ok_or_else(|| self.err(node, ErrorCode::Missing, format!("missing {name}")))?;
        self.allowed(value, &[], &[])?;
        value
            .text()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .ok_or_else(|| self.err(value, ErrorCode::Missing, format!("missing {name}")))
    }
    fn number(&self, node: Node<'_, '_>, default: f64) -> Result<f64, Diagnostic> {
        self.allowed(node, &[], &[])?;
        let text = node.text().unwrap_or("").trim();
        if text.is_empty() {
            return Ok(default);
        }
        text.parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
            .ok_or_else(|| self.err(node, ErrorCode::InvalidValue, "expected finite number"))
    }
    fn scalar(&self, node: Node<'_, '_>, name: &str, default: f64) -> Result<f64, Diagnostic> {
        child(node, name).map_or(Ok(default), |n| self.number(n, default))
    }
    fn boolean(&self, node: Node<'_, '_>, name: &str, default: bool) -> Result<bool, Diagnostic> {
        match child(node, name) {
            None => Ok(default),
            Some(n) => {
                self.allowed(n, &[], &[])?;
                match n.text().unwrap_or("").trim() {
                    "true" | "1" => Ok(true),
                    "false" | "0" => Ok(false),
                    _ => Err(self.err(n, ErrorCode::InvalidValue, "expected boolean")),
                }
            }
        }
    }
    fn values<const N: usize>(&self, node: Node<'_, '_>) -> Result<[f64; N], Diagnostic> {
        let attrs: &[&str] = match node.tag_name().name() {
            "pose" => &["relative_to", "degrees", "rotation_format"],
            "xyz" => &["expressed_in"],
            "custom_rpy" | "grav_dir_x" => &["parent_frame"],
            _ => &[],
        };
        self.allowed(node, &[], attrs)?;
        let values = node
            .text()
            .unwrap_or("")
            .split_whitespace()
            .map(str::parse::<f64>)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| self.err(node, ErrorCode::InvalidValue, "expected numeric vector"))?;
        if values.iter().any(|n| !n.is_finite()) {
            return Err(self.err(node, ErrorCode::InvalidValue, "expected finite vector"));
        }
        values.try_into().map_err(|_| {
            self.err(
                node,
                ErrorCode::InvalidValue,
                format!("expected {N} values"),
            )
        })
    }
    fn pose(&self, node: Node<'_, '_>) -> Result<Pose, Diagnostic> {
        let Some(pose) = child(node, "pose") else {
            return Ok(Pose::default());
        };
        self.allowed(pose, &[], &["relative_to", "degrees", "rotation_format"])?;
        let degrees = match pose.attribute("degrees").unwrap_or("false") {
            "false" | "0" => false,
            "true" | "1" => true,
            _ => return Err(self.err(pose, ErrorCode::InvalidValue, "invalid degrees attribute")),
        };
        if pose.text().unwrap_or("").trim().is_empty() {
            return Ok(Pose {
                relative_to: pose.attribute("relative_to").unwrap_or("").into(),
                ..Pose::default()
            });
        }
        let (translation, rotation) = match pose.attribute("rotation_format").unwrap_or("euler_rpy")
        {
            "euler_rpy" => {
                let v = self.values::<6>(pose)?;
                let factor = if degrees {
                    std::f64::consts::PI / 180.
                } else {
                    1.
                };
                (
                    [v[0], v[1], v[2]],
                    DQuat::from_euler(EulerRot::ZYX, v[5] * factor, v[4] * factor, v[3] * factor),
                )
            }
            "quat_xyzw" if !degrees => {
                let v = self.values::<7>(pose)?;
                let q = DQuat::from_xyzw(v[3], v[4], v[5], v[6]);
                if q.length_squared() < f64::MIN_POSITIVE || !q.length_squared().is_finite() {
                    return Err(self.err(pose, ErrorCode::InvalidValue, "invalid quaternion norm"));
                }
                ([v[0], v[1], v[2]], q.normalize())
            }
            _ => {
                return Err(self.err(
                    pose,
                    ErrorCode::InvalidValue,
                    "invalid rotation format or degrees with quaternion",
                ));
            }
        };
        Ok(Pose {
            transform: Transform {
                translation,
                rotation: rotation.to_array(),
            },
            relative_to: pose.attribute("relative_to").unwrap_or("").into(),
        })
    }
    fn plugins(&self, node: Node<'_, '_>) -> Result<Vec<Plugin>, Diagnostic> {
        elements(node)
            .filter(|n| n.has_tag_name("plugin"))
            .map(|p| {
                Ok(Plugin {
                    name: self.attr(p, "name")?,
                    filename: self.attr(p, "filename")?,
                    xml: self.xml[p.range()].into(),
                })
            })
            .collect()
    }
    fn world(&mut self, node: Node<'_, '_>) -> Result<World, Diagnostic> {
        self.allowed(
            node,
            &["model", "include", "frame", "gravity", "plugin"],
            &["name"],
        )?;
        let name = self.name(node)?;
        let gravity = child(node, "gravity").map_or(Ok([0., 0., -9.8]), |n| self.values(n))?;
        let models = self.models(node, &name)?;
        Ok(World {
            name,
            gravity,
            models,
            frames: self.frames(node)?,
            plugins: self.plugins(node)?,
            source: self.source(node),
        })
    }
    fn frames(&self, node: Node<'_, '_>) -> Result<Vec<Frame>, Diagnostic> {
        elements(node)
            .filter(|n| n.has_tag_name("frame"))
            .map(|n| {
                self.allowed(n, &["pose"], &["name", "attached_to"])?;
                Ok(Frame {
                    name: self.name(n)?,
                    attached_to: n.attribute("attached_to").unwrap_or("").into(),
                    pose: self.pose(n)?,
                    source: self.source(n),
                })
            })
            .collect()
    }
    fn models(&mut self, node: Node<'_, '_>, scope: &str) -> Result<Vec<Model>, Diagnostic> {
        elements(node)
            .filter(|n| n.has_tag_name("model") || n.has_tag_name("include"))
            .map(|n| {
                if n.has_tag_name("model") {
                    self.model(n, scope, None)
                } else {
                    self.include(n, scope)
                }
            })
            .collect()
    }
    fn model(
        &mut self,
        node: Node<'_, '_>,
        scope: &str,
        rename: Option<&str>,
    ) -> Result<Model, Diagnostic> {
        if scope.split("::").count() > 64 {
            return Err(self.err(
                node,
                ErrorCode::Resource,
                "model nesting limit (64) exceeded",
            ));
        }
        self.allowed(
            node,
            &[
                "pose", "static", "link", "model", "include", "frame", "joint", "plugin",
            ],
            &["name", "canonical_link", "placement_frame"],
        )?;
        let name = rename.map_or_else(|| self.name(node), |n| Ok(n.into()))?;
        let scoped_name = join(scope, &name);
        let pose = self.pose(node)?;
        if scope == "::" && !pose.relative_to.is_empty() {
            return Err(self.err(
                node,
                ErrorCode::InvalidValue,
                "top-level model pose relative_to must be empty",
            ));
        }
        let links = elements(node)
            .filter(|n| n.has_tag_name("link"))
            .map(|n| self.link(n, &scoped_name))
            .collect::<Result<_, _>>()?;
        let joints = elements(node)
            .filter(|n| n.has_tag_name("joint"))
            .map(|n| self.joint(n))
            .collect::<Result<_, _>>()?;
        Ok(Model {
            name,
            scoped_name: scoped_name.clone(),
            pose,
            is_static: self.boolean(node, "static", false)?,
            canonical_link: node.attribute("canonical_link").unwrap_or("").into(),
            placement_frame: node.attribute("placement_frame").unwrap_or("").into(),
            links,
            joints,
            models: self.models(node, &scoped_name)?,
            frames: self.frames(node)?,
            plugins: self.plugins(node)?,
            source: self.source(node),
        })
    }
    fn include(&mut self, node: Node<'_, '_>, scope: &str) -> Result<Model, Diagnostic> {
        self.allowed(
            node,
            &["uri", "name", "pose", "static", "placement_frame"],
            &[],
        )?;
        if self.includes.len() >= 64 {
            return Err(self.err(
                node,
                ErrorCode::Resource,
                "include depth limit (64) exceeded",
            ));
        }
        let uri = self.text(node, "uri")?;
        let resource = self
            .resolver
            .resolve(&uri, self.uri)
            .map_err(|e| self.err(node, ErrorCode::Resource, e))?;
        if self.includes.contains(&resource.canonical_uri) {
            return Err(self.err(node, ErrorCode::Cycle, "cyclic include"));
        }
        let mut includes = self.includes.clone();
        includes.push(resource.canonical_uri.clone());
        let mut imported = Parser {
            uri: &resource.canonical_uri,
            xml: &resource.xml,
            resolver: self.resolver,
            includes,
        };
        let doc = Document::parse(&resource.xml).map_err(|e| Diagnostic {
            code: e.code,
            source: Source {
                uri: resource.canonical_uri.clone(),
                line: e.pos().row,
                column: e.pos().col,
            },
            message: e.to_string(),
        })?;
        let root = doc.root_element();
        imported.check_root(root)?;
        imported.allowed(root, &["model"], &["version"])?;
        let all = elements(root).collect::<Vec<_>>();
        if all.len() != 1 {
            return Err(self.err(
                node,
                ErrorCode::InvalidValue,
                "include requires exactly one model",
            ));
        }
        let rename = child(node, "name")
            .map(|_| self.text(node, "name"))
            .transpose()?;
        if let Some(name) = &rename
            && (name.contains("::") || name.starts_with("__") || name == "world")
        {
            return Err(self.err(node, ErrorCode::InvalidValue, "invalid include name"));
        }
        let mut model = imported.model(all[0], scope, rename.as_deref())?;
        if let Some(placement) = child(node, "placement_frame") {
            if child(node, "pose").is_none() {
                return Err(self.err(
                    placement,
                    ErrorCode::Missing,
                    "include placement_frame requires pose",
                ));
            }
            self.allowed(placement, &[], &[])?;
            model.placement_frame = placement.text().unwrap_or("").trim().into();
        }
        if child(node, "pose").is_some() {
            model.pose = self.pose(node)?;
        }
        if child(node, "static").is_some() {
            model.is_static = self.boolean(node, "static", false)?;
        }
        Ok(model)
    }
    fn link(&self, node: Node<'_, '_>, scope: &str) -> Result<Link, Diagnostic> {
        self.allowed(
            node,
            &["pose", "inertial", "collision", "sensor", "plugin"],
            &["name"],
        )?;
        let name = self.name(node)?;
        let collisions = elements(node)
            .filter(|n| n.has_tag_name("collision"))
            .map(|n| self.collision(n))
            .collect::<Result<Vec<_>, _>>()?;
        let sensors = elements(node)
            .filter(|n| n.has_tag_name("sensor"))
            .map(|n| self.sensor(n))
            .collect::<Result<Vec<_>, _>>()?;
        for names in [
            collisions.iter().map(|c| &c.name).collect::<Vec<_>>(),
            sensors.iter().map(|s| &s.name).collect(),
        ] {
            if names.iter().copied().collect::<BTreeSet<_>>().len() != names.len() {
                return Err(self.err(
                    node,
                    ErrorCode::DuplicateName,
                    "duplicate collision or sensor name",
                ));
            }
        }
        Ok(Link {
            scoped_name: join(scope, &name),
            name,
            pose: self.pose(node)?,
            inertial: self.inertial(node)?,
            collisions,
            sensors,
            plugins: self.plugins(node)?,
            source: self.source(node),
        })
    }
    fn inertial(&self, link: Node<'_, '_>) -> Result<Inertial, Diagnostic> {
        let Some(node) = child(link, "inertial") else {
            return Ok(Inertial::default());
        };
        self.allowed(node, &["mass", "inertia", "pose"], &[])?;
        let mut result = Inertial {
            mass: self.scalar(node, "mass", 1.)?,
            pose: self.pose(node)?,
            ..Inertial::default()
        };
        if let Some(inertia) = child(node, "inertia") {
            self.allowed(inertia, &["ixx", "iyy", "izz", "ixy", "ixz", "iyz"], &[])?;
            for (i, name) in ["ixx", "iyy", "izz", "ixy", "ixz", "iyz"]
                .iter()
                .enumerate()
            {
                result.inertia[i] = self.scalar(inertia, name, result.inertia[i])?;
            }
        }
        let [a, b, c, d, e, f] = result.inertia;
        // Sylvester's criterion and triangle inequalities on the second moment
        // matrix enforce physical inertia, including products of inertia.
        let second = [
            (b + c - a) / 2.,
            (a + c - b) / 2.,
            (a + b - c) / 2.,
            -d,
            -e,
            -f,
        ];
        let [x, y, z, u, v, w] = second;
        let determinant = x * y * z + 2. * u * v * w - x * w * w - y * v * v - z * u * u;
        if result.mass <= 0.
            || a <= 0.
            || a * b - d * d <= 0.
            || a * b * c + 2. * d * e * f - a * f * f - b * e * e - c * d * d <= 0.
            || x < 0.
            || y < 0.
            || z < 0.
            || x * y - u * u < 0.
            || x * z - v * v < 0.
            || y * z - w * w < 0.
            || determinant < 0.
        {
            return Err(self.err(
                node,
                ErrorCode::InvalidValue,
                "mass/inertia must be positive and physically realizable",
            ));
        }
        Ok(result)
    }
    fn collision(&self, node: Node<'_, '_>) -> Result<Collision, Diagnostic> {
        self.allowed(node, &["pose", "geometry"], &["name"])?;
        let geometry = child(node, "geometry")
            .ok_or_else(|| self.err(node, ErrorCode::Missing, "collision requires geometry"))?;
        self.allowed(geometry, &["box", "sphere", "cylinder", "plane"], &[])?;
        let shapes = elements(geometry).collect::<Vec<_>>();
        if shapes.len() != 1 {
            return Err(self.err(
                geometry,
                ErrorCode::InvalidValue,
                "geometry requires exactly one shape",
            ));
        }
        let shape = shapes[0];
        let geometry = match shape.tag_name().name() {
            "box" => {
                self.allowed(shape, &["size"], &[])?;
                Geometry::Box {
                    size: child(shape, "size").map_or(Ok([1.; 3]), |n| self.values(n))?,
                }
            }
            "sphere" => {
                self.allowed(shape, &["radius"], &[])?;
                Geometry::Sphere {
                    radius: self.scalar(shape, "radius", 1.)?,
                }
            }
            "cylinder" => {
                self.allowed(shape, &["radius", "length"], &[])?;
                Geometry::Cylinder {
                    radius: self.scalar(shape, "radius", 1.)?,
                    length: self.scalar(shape, "length", 1.)?,
                }
            }
            _ => {
                self.allowed(shape, &["normal", "size"], &[])?;
                let normal = child(shape, "normal").map_or(Ok([0., 0., 1.]), |n| self.values(n))?;
                if DVec3::from_array(normal).length_squared() < f64::MIN_POSITIVE
                    || !DVec3::from_array(normal).length_squared().is_finite()
                {
                    return Err(self.err(shape, ErrorCode::InvalidValue, "plane normal is zero"));
                }
                Geometry::Plane {
                    normal: DVec3::from_array(normal).normalize().to_array(),
                    size: child(shape, "size").map_or(Ok([1.; 2]), |n| self.values(n))?,
                }
            }
        };
        let positive = match &geometry {
            Geometry::Box { size } => size.iter().all(|v| *v > 0.),
            Geometry::Sphere { radius } => *radius > 0.,
            Geometry::Cylinder { radius, length } => *radius > 0. && *length > 0.,
            Geometry::Plane { size, .. } => size.iter().all(|v| *v > 0.),
        };
        if !positive {
            return Err(self.err(
                shape,
                ErrorCode::InvalidValue,
                "shape dimensions must be positive",
            ));
        }
        Ok(Collision {
            name: self.name(node)?,
            pose: self.pose(node)?,
            geometry,
            source: self.source(node),
        })
    }
    fn joint(&self, node: Node<'_, '_>) -> Result<Joint, Diagnostic> {
        self.allowed(
            node,
            &["parent", "child", "pose", "axis"],
            &["name", "type"],
        )?;
        let kind = match self.attr(node, "type")?.as_str() {
            "fixed" => JointKind::Fixed,
            "revolute" => JointKind::Revolute,
            "prismatic" => JointKind::Prismatic,
            "continuous" => JointKind::Continuous,
            _ => return Err(self.err(node, ErrorCode::Unsupported, "unsupported joint type")),
        };
        let axis = child(node, "axis")
            .map(|axis| {
                self.allowed(axis, &["xyz", "limit"], &[])?;
                let (xyz, expressed_in) = match child(axis, "xyz") {
                    Some(xyz) => {
                        self.allowed(xyz, &[], &["expressed_in"])?;
                        (
                            self.values(xyz)?,
                            xyz.attribute("expressed_in").unwrap_or("").into(),
                        )
                    }
                    None => ([0., 0., 1.], String::new()),
                };
                let length = DVec3::from_array(xyz).length();
                if length <= f64::MIN_POSITIVE || !length.is_finite() {
                    return Err(self.err(axis, ErrorCode::InvalidValue, "invalid joint axis"));
                }
                let mut result = Axis {
                    xyz: (DVec3::from_array(xyz) / length).to_array(),
                    expressed_in,
                    lower: -1e16,
                    upper: 1e16,
                    effort: f64::INFINITY,
                    velocity: f64::INFINITY,
                };
                if let Some(limit) = child(axis, "limit") {
                    self.allowed(limit, &["lower", "upper", "effort", "velocity"], &[])?;
                    result.lower = self.scalar(limit, "lower", result.lower)?;
                    result.upper = self.scalar(limit, "upper", result.upper)?;
                    result.effort = self.scalar(limit, "effort", result.effort)?;
                    result.velocity = self.scalar(limit, "velocity", result.velocity)?;
                    if result.effort < 0. {
                        result.effort = f64::INFINITY;
                    }
                    if result.velocity < 0. {
                        result.velocity = f64::INFINITY;
                    }
                }
                if result.lower > result.upper {
                    return Err(self.err(
                        axis,
                        ErrorCode::InvalidValue,
                        "joint lower limit exceeds upper limit",
                    ));
                }
                Ok(result)
            })
            .transpose()?;
        let child = self.text(node, "child")?;
        if child == "world" {
            return Err(self.err(node, ErrorCode::InvalidValue, "joint child cannot be world"));
        }
        Ok(Joint {
            name: self.name(node)?,
            kind,
            parent: self.text(node, "parent")?,
            child,
            pose: self.pose(node)?,
            axis,
            source: self.source(node),
        })
    }
    fn imu(&self, sensor: Node<'_, '_>) -> Result<Imu, Diagnostic> {
        let Some(node) = child(sensor, "imu") else {
            return Ok(Imu::default());
        };
        self.allowed(
            node,
            &[
                "angular_velocity",
                "linear_acceleration",
                "orientation_reference_frame",
                "enable_orientation",
            ],
            &[],
        )?;
        let mut imu = Imu {
            enable_orientation: self.boolean(node, "enable_orientation", true)?,
            ..Imu::default()
        };
        for (name, noises) in [
            ("angular_velocity", &mut imu.angular_velocity),
            ("linear_acceleration", &mut imu.linear_acceleration),
        ] {
            if let Some(vector) = child(node, name) {
                self.allowed(vector, &["x", "y", "z"], &[])?;
                for (i, component) in ["x", "y", "z"].iter().enumerate() {
                    if let Some(axis) = child(vector, component) {
                        self.allowed(axis, &["noise"], &[])?;
                        if let Some(noise) = child(axis, "noise") {
                            noises[i] = self.noise(noise)?;
                        }
                    }
                }
            }
        }
        if let Some(reference) = child(node, "orientation_reference_frame") {
            self.allowed(
                reference,
                &["localization", "custom_rpy", "grav_dir_x"],
                &[],
            )?;
            if child(reference, "localization").is_some() {
                imu.localization = self.text(reference, "localization")?;
            }
            if let Some(rpy) = child(reference, "custom_rpy") {
                imu.custom_rpy = self.values(rpy)?;
                imu.custom_rpy_parent_frame = rpy.attribute("parent_frame").unwrap_or("").into();
            }
            if let Some(gravity) = child(reference, "grav_dir_x") {
                imu.gravity_dir_x = self.values(gravity)?;
                imu.gravity_dir_x_parent_frame =
                    gravity.attribute("parent_frame").unwrap_or("").into();
            }
        }
        Ok(imu)
    }
    fn noise(&self, node: Node<'_, '_>) -> Result<Noise, Diagnostic> {
        self.allowed(
            node,
            &[
                "mean",
                "stddev",
                "bias_mean",
                "bias_stddev",
                "dynamic_bias_stddev",
                "dynamic_bias_correlation_time",
                "precision",
            ],
            &["type"],
        )?;
        let kind = match node
            .attribute("type")
            .unwrap_or("none")
            .to_ascii_lowercase()
            .as_str()
        {
            "none" => NoiseKind::None,
            "gaussian" => NoiseKind::Gaussian,
            "gaussian_quantized" => NoiseKind::GaussianQuantized,
            _ => return Err(self.err(node, ErrorCode::InvalidValue, "invalid noise type")),
        };
        let noise = Noise {
            kind,
            mean: self.scalar(node, "mean", 0.)?,
            stddev: self.scalar(node, "stddev", 0.)?,
            bias_mean: self.scalar(node, "bias_mean", 0.)?,
            bias_stddev: self.scalar(node, "bias_stddev", 0.)?,
            dynamic_bias_stddev: self.scalar(node, "dynamic_bias_stddev", 0.)?,
            dynamic_bias_correlation_time: self.scalar(
                node,
                "dynamic_bias_correlation_time",
                0.,
            )?,
            precision: self.scalar(node, "precision", 0.)?,
        };
        if noise.stddev < 0. || noise.bias_stddev < 0. || noise.dynamic_bias_stddev < 0. {
            return Err(self.err(
                node,
                ErrorCode::InvalidValue,
                "noise standard deviations must be nonnegative",
            ));
        }
        Ok(noise)
    }
    fn sensor(&self, node: Node<'_, '_>) -> Result<Sensor, Diagnostic> {
        self.allowed(
            node,
            &[
                "pose",
                "update_rate",
                "always_on",
                "enable_metrics",
                "topic",
                "imu",
            ],
            &["name", "type"],
        )?;
        let kind = self.attr(node, "type")?;
        if kind != "imu" {
            return Err(self.err(
                node,
                ErrorCode::Unsupported,
                "only IMU sensor envelope is supported",
            ));
        }
        let update_rate = self.scalar(node, "update_rate", 0.)?;
        if update_rate < 0. {
            return Err(self.err(node, ErrorCode::InvalidValue, "negative sensor update rate"));
        }
        Ok(Sensor {
            name: self.name(node)?,
            kind,
            pose: self.pose(node)?,
            update_rate,
            always_on: self.boolean(node, "always_on", false)?,
            enable_metrics: self.boolean(node, "enable_metrics", false)?,
            imu: self.imu(node)?,
            topic: child(node, "topic")
                .map_or(String::new(), |n| n.text().unwrap_or("").trim().into()),
            configuration_xml: child(node, "imu")
                .map_or(String::new(), |n| self.xml[n.range()].into()),
            source: self.source(node),
        })
    }
}
fn elements<'a, 'b>(node: Node<'a, 'b>) -> impl Iterator<Item = Node<'a, 'b>> {
    node.children()
}
fn child<'a, 'b>(node: Node<'a, 'b>, tag: &str) -> Option<Node<'a, 'b>> {
    elements(node).find(|n| n.has_tag_name(tag))
}
fn validate_model(model: &Model, graphs: &Graphs) -> Result<(), Diagnostic> {
    let scope = &model.scoped_name;
    for joint in &model.joints {
        let parent = graphs.joint_parent(&joint.parent, scope, &joint.source)?;
        let child = graphs.validate_reference(&joint.child, scope, &joint.source)?;
        let parent_body = graphs.attached_body(&parent)?;
        let child_body = graphs.attached_body(&child)?;
        if child_body == "::" || !child_body.contains("::") || child_body == parent_body {
            return Err(error(
                ErrorCode::InvalidValue,
                &joint.source,
                "invalid joint endpoints",
            ));
        }
        if let Some(axis) = &joint.axis
            && !axis.expressed_in.is_empty()
        {
            graphs.validate_reference(&axis.expressed_in, scope, &joint.source)?;
        }
    }
    for link in &model.links {
        for pose in std::iter::once(&link.inertial.pose)
            .chain(link.collisions.iter().map(|c| &c.pose))
            .chain(link.sensors.iter().map(|s| &s.pose))
        {
            graphs.resolve_pose(pose, scope, &link.scoped_name, "::")?;
        }
    }
    for nested in &model.models {
        validate_model(nested, graphs)?;
    }
    Ok(())
}
