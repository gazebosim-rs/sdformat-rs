// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::graph::{Graphs, error, join};
use crate::xml::{Document, Node, legacy_whitespace};
use crate::*;
use glam::{DQuat, DVec3, EulerRot};
use std::{borrow::Cow, collections::BTreeSet};

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
            Some("1.6" | "1.7" | "1.8" | "1.9" | "1.10" | "1.11" | "1.12")
        ) {
            return Err(self.err(root, ErrorCode::Version, "supported versions are SDF 1.6 through 1.12; earlier conversion is not implemented"));
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
            "visual",
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
        if name.contains("::")
            && matches!(
                node.document().root_element().attribute("version"),
                Some("1.6" | "1.7")
            )
        {
            return Err(self.err(
                node,
                ErrorCode::Unsupported,
                "legacy SDF flattened-name conversion is not implemented",
            ));
        }
        if name.contains("::") || reserved_name(&name) || name.trim() != name {
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
        let text = param_text(node);
        if text.is_empty() {
            return Ok(default);
        }
        let text = if text.eq_ignore_ascii_case("true") {
            "1"
        } else if text.eq_ignore_ascii_case("false") {
            "0"
        } else {
            &text
        };
        crate::number::scalar_float(text)
            .map_err(|code| self.err(node, code, "scalar floating-point conversion failed"))
    }
    fn scalar(&self, node: Node<'_, '_>, name: &str, default: f64) -> Result<f64, Diagnostic> {
        child(node, name).map_or(Ok(default), |n| self.number(n, default))
    }
    fn boolean(&self, node: Node<'_, '_>, name: &str, default: bool) -> Result<bool, Diagnostic> {
        match child(node, name) {
            None => Ok(default),
            Some(n) => {
                self.allowed(n, &[], &[])?;
                let text = param_text(n);
                match text.to_ascii_lowercase().as_str() {
                    "true" | "1" => Ok(true),
                    "false" | "0" => Ok(false),
                    "" => Ok(default),
                    _ => Err(self.err(n, ErrorCode::InvalidValue, "expected boolean")),
                }
            }
        }
    }
    fn values<const N: usize>(&self, node: Node<'_, '_>) -> Result<[f64; N], Diagnostic> {
        let attrs: &[&str] = match node.tag_name().name() {
            "pose" => pose_attributes(node),
            "xyz" => &["expressed_in"],
            "custom_rpy" | "grav_dir_x" => &["parent_frame"],
            _ => &[],
        };
        self.allowed(node, &[], attrs)?;
        let values = node
            .text()
            .unwrap_or("")
            .split(legacy_whitespace)
            .filter(|part| !part.is_empty())
            .map(|part| {
                if node.has_tag_name("pose") {
                    // Native ParsePose converts individual tokens with stod;
                    // vector streams intentionally retain their distinct path.
                    crate::number::scalar_float(part)
                } else {
                    part.parse::<f64>().map_err(|_| ErrorCode::InvalidValue)
                }
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|code| self.err(node, code, "expected numeric vector"))?;
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
        self.allowed(pose, &[], pose_attributes(pose))?;
        // The 1.6 -> 1.7 move overwrites relative_to even for an empty frame.
        let relative_to = if is_16(pose) {
            pose.attribute("frame")
                .or_else(|| pose.attribute("relative_to"))
                .unwrap_or("")
                .trim_matches([' ', '\t', '\n'])
                .into()
        } else {
            pose.attribute("relative_to").unwrap_or("").into()
        };
        let degrees = match pose.attribute("degrees").unwrap_or("false") {
            "false" | "0" => false,
            "true" | "1" => true,
            _ => return Err(self.err(pose, ErrorCode::InvalidValue, "invalid degrees attribute")),
        };
        if param_text(pose).is_empty() {
            return Ok(Pose {
                relative_to,
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
            relative_to,
        })
    }
    fn plugins(&self, node: Node<'_, '_>) -> Result<Vec<Plugin>, Diagnostic> {
        elements(node)
            .filter(|n| n.has_tag_name("plugin"))
            .map(|p| {
                let filename = p
                    .attribute("filename")
                    .map(|value| value.trim_matches([' ', '\t', '\n']))
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| self.err(p, ErrorCode::Missing, "missing filename attribute"))?;
                Ok(Plugin {
                    name: p
                        .attribute("name")
                        .unwrap_or("")
                        .trim_matches([' ', '\t', '\n'])
                        .into(),
                    filename: filename.into(),
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
        let mut frames = self.frames(node)?;
        if is_16(node) {
            // World loads models before frames, regardless of declaration order.
            let mut names = models.iter().map(|model| model.name.clone()).collect();
            let mut typed_names = BTreeSet::new();
            for (frame, element) in frames
                .iter_mut()
                .zip(elements(node).filter(|n| n.has_tag_name("frame")))
            {
                frame.name =
                    self.legacy_name(element, "frame", &frame.name, &mut names, &mut typed_names)?;
            }
        }
        Ok(World {
            name,
            gravity,
            models,
            frames,
            plugins: self.plugins(node)?,
            source: self.source(node),
        })
    }
    fn frames(&self, node: Node<'_, '_>) -> Result<Vec<Frame>, Diagnostic> {
        elements(node)
            .filter(|n| n.has_tag_name("frame"))
            .map(|n| self.frame(n))
            .collect()
    }
    fn frame(&self, node: Node<'_, '_>) -> Result<Frame, Diagnostic> {
        self.allowed(node, &["pose"], &["name", "attached_to"])?;
        Ok(Frame {
            name: self.name(node)?,
            attached_to: node.attribute("attached_to").unwrap_or("").into(),
            pose: self.pose(node)?,
            source: self.source(node),
        })
    }
    fn legacy_name(
        &self,
        node: Node<'_, '_>,
        kind: &'static str,
        original: &str,
        names: &mut BTreeSet<String>,
        typed_names: &mut BTreeSet<(&'static str, String)>,
    ) -> Result<String, Diagnostic> {
        if !typed_names.insert((kind, original.into())) {
            return Err(self.err(
                node,
                ErrorCode::DuplicateName,
                format!("duplicate {kind} name {original}"),
            ));
        }
        let mut name = original.to_string();
        if kind != "model" && names.contains(&name) {
            name = format!("{original}_{kind}");
            let mut index = 0usize;
            while names.contains(&name) {
                name = format!("{original}_{kind}{index}");
                index += 1;
            }
        }
        names.insert(name.clone());
        Ok(name)
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
        let mut links = Vec::new();
        let mut joints = Vec::new();
        let mut models = Vec::new();
        let mut frames = Vec::new();
        let mut names = BTreeSet::new();
        let mut typed_names = BTreeSet::new();
        // Legacy collisions rename only the element loaded later. Future names
        // must not be reserved early; references retain their declared strings.
        for member in elements(node) {
            match member.tag_name().name() {
                "link" => {
                    let mut link = self.link(member, &scoped_name)?;
                    if is_16(node) {
                        link.name = self.legacy_name(
                            member,
                            "link",
                            &link.name,
                            &mut names,
                            &mut typed_names,
                        )?;
                        link.scoped_name = join(&scoped_name, &link.name);
                    }
                    links.push(link);
                }
                "joint" => {
                    let mut joint = self.joint(member)?;
                    if is_16(node) {
                        joint.name = self.legacy_name(
                            member,
                            "joint",
                            &joint.name,
                            &mut names,
                            &mut typed_names,
                        )?;
                    }
                    joints.push(joint);
                }
                "frame" => {
                    let mut frame = self.frame(member)?;
                    if is_16(node) {
                        frame.name = self.legacy_name(
                            member,
                            "frame",
                            &frame.name,
                            &mut names,
                            &mut typed_names,
                        )?;
                    }
                    frames.push(frame);
                }
                "model" | "include" => {
                    let nested = if member.has_tag_name("model") {
                        self.model(member, &scoped_name, None)?
                    } else {
                        self.include(member, &scoped_name)?
                    };
                    if is_16(node) {
                        self.legacy_name(
                            member,
                            "model",
                            &nested.name,
                            &mut names,
                            &mut typed_names,
                        )?;
                    }
                    models.push(nested);
                }
                _ => {}
            }
        }
        Ok(Model {
            name,
            scoped_name: scoped_name.clone(),
            pose,
            is_static: self.boolean(node, "static", false)?,
            canonical_link: node.attribute("canonical_link").unwrap_or("").into(),
            placement_frame: node.attribute("placement_frame").unwrap_or("").into(),
            include: None,
            links,
            joints,
            models,
            frames,
            plugins: self.plugins(node)?,
            source: self.source(node),
        })
    }
    fn include(&mut self, node: Node<'_, '_>, scope: &str) -> Result<Model, Diagnostic> {
        self.allowed(
            node,
            &["uri", "name", "pose", "static", "placement_frame", "plugin"],
            &[],
        )?;
        if self.includes.len() >= 64 {
            return Err(self.err(
                node,
                ErrorCode::Resource,
                "include depth limit (64) exceeded",
            ));
        }
        let uri_node =
            child(node, "uri").ok_or_else(|| self.err(node, ErrorCode::Missing, "missing uri"))?;
        self.allowed(uri_node, &[], &[])?;
        let uri = param_text(uri_node).into_owned();
        if uri.is_empty() {
            return Err(self.err(uri_node, ErrorCode::Missing, "missing uri"));
        }
        // Native file lookup uses the XML text, while Model::Uri is a parsed
        // string parameter. CDATA leading/trailing whitespace differs here.
        let request_uri = if uri_node.text_is_cdata() {
            uri_node.text().unwrap_or("")
        } else {
            &uri
        };
        let resource = self
            .resolver
            .resolve(request_uri, self.uri)
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
        let rename = self.optional_text(node, "name")?;
        if rename.as_deref() == Some("") {
            return Err(self.err(node, ErrorCode::Missing, "missing include name"));
        }
        if let Some(name) = &rename
            && (name.contains("::") || reserved_name(name))
        {
            return Err(self.err(node, ErrorCode::InvalidValue, "invalid include name"));
        }
        let mut model = imported.model(all[0], scope, rename.as_deref())?;
        let pose = child(node, "pose").map(|_| self.pose(node)).transpose()?;
        let placement_frame = if let Some(placement) = child(node, "placement_frame") {
            if pose.is_none() {
                return Err(self.err(
                    placement,
                    ErrorCode::Missing,
                    "include placement_frame requires pose",
                ));
            }
            self.allowed(placement, &[], &[])?;
            let value = param_text(placement).into_owned();
            model.placement_frame = value.clone();
            Some(value)
        } else {
            None
        };
        if let Some(pose) = &pose {
            model.pose = pose.clone();
        }
        let is_static = child(node, "static")
            .map(|_| self.boolean(node, "static", false))
            .transpose()?;
        if let Some(is_static) = is_static {
            model.is_static = is_static;
        }
        let plugins = self.plugins(node)?;
        model.plugins.extend(plugins.iter().cloned());
        model.include = Some(Box::new(Include {
            uri,
            name: rename,
            pose,
            is_static,
            placement_frame,
            plugins,
            source: self.source(node),
            xml: self.xml[node.range()].into(),
        }));
        Ok(model)
    }
    fn link(&self, node: Node<'_, '_>, scope: &str) -> Result<Link, Diagnostic> {
        self.allowed(
            node,
            &[
                "pose",
                "inertial",
                "collision",
                "visual",
                "sensor",
                "plugin",
            ],
            &["name"],
        )?;
        let name = self.name(node)?;
        let collisions = elements(node)
            .filter(|n| n.has_tag_name("collision"))
            .map(|n| self.collision(n))
            .collect::<Result<Vec<_>, _>>()?;
        let visuals = elements(node)
            .filter(|n| n.has_tag_name("visual"))
            .map(|n| self.visual(n))
            .collect::<Result<Vec<_>, _>>()?;
        let sensors = elements(node)
            .filter(|n| n.has_tag_name("sensor"))
            .map(|n| self.sensor(n))
            .collect::<Result<Vec<_>, _>>()?;
        for names in [
            collisions.iter().map(|c| &c.name).collect::<Vec<_>>(),
            sensors.iter().map(|s| &s.name).collect(),
            visuals.iter().map(|v| &v.name).collect(),
        ] {
            if names.iter().copied().collect::<BTreeSet<_>>().len() != names.len() {
                return Err(self.err(
                    node,
                    ErrorCode::DuplicateName,
                    "duplicate collision, visual or sensor name",
                ));
            }
        }
        Ok(Link {
            scoped_name: join(scope, &name),
            name,
            pose: self.pose(node)?,
            inertial: self.inertial(node)?,
            collisions,
            visuals,
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
        self.allowed(node, &["pose", "geometry", "surface"], &["name"])?;
        let geometry = child(node, "geometry")
            .ok_or_else(|| self.err(node, ErrorCode::Missing, "collision requires geometry"))?;
        Ok(Collision {
            name: self.name(node)?,
            pose: self.pose(node)?,
            geometry: self.geometry(geometry)?,
            surface: child(node, "surface").map_or(Ok(Surface::default()), |n| self.surface(n))?,
            source: self.source(node),
        })
    }
    fn nonnegative(&self, node: Node<'_, '_>, name: &str, default: f64) -> Result<f64, Diagnostic> {
        let value = self.scalar(node, name, default)?;
        if value < 0. {
            Err(self.err(
                child(node, name).unwrap_or(node),
                ErrorCode::InvalidValue,
                format!("{name} must be nonnegative"),
            ))
        } else {
            Ok(value)
        }
    }
    fn optional_values<const N: usize>(
        &self,
        node: Node<'_, '_>,
        name: &str,
        default: [f64; N],
    ) -> Result<[f64; N], Diagnostic> {
        child(node, name).map_or(Ok(default), |value| {
            self.allowed(value, &[], &[])?;
            if param_text(value).is_empty() {
                Ok(default)
            } else {
                self.values(value)
            }
        })
    }
    fn surface(&self, node: Node<'_, '_>) -> Result<Surface, Diagnostic> {
        self.allowed(node, &["contact", "friction"], &[])?;
        let contact = child(node, "contact").map_or(Ok(Contact::default()), |n| {
            self.allowed(n, &["collide_bitmask", "category_bitmask"], &[])?;
            Ok(Contact {
                // Native absent-group default differs from the schema default
                // used by an explicitly empty collide_bitmask element.
                collide_bitmask: if child(n, "collide_bitmask").is_some() {
                    self.unsigned(n, "collide_bitmask", 65535)? as u16
                } else {
                    0xff
                },
                category_bitmask: child(n, "category_bitmask")
                    .map(|_| {
                        self.unsigned(n, "category_bitmask", 65535)
                            .map(|v| v as u16)
                    })
                    .transpose()?,
                source: Some(self.source(n)),
            })
        })?;
        let friction = child(node, "friction").map_or(Ok(Friction::default()), |n| {
            self.allowed(n, &["ode", "bullet", "torsional"], &[])?;
            let ode = child(n, "ode").map_or(Ok(OdeFriction::default()), |v| {
                self.allowed(v, &["mu", "mu2", "fdir1", "slip1", "slip2"], &[])?;
                Ok(OdeFriction {
                    mu: self.nonnegative(v, "mu", 1.)?,
                    mu2: self.nonnegative(v, "mu2", 1.)?,
                    fdir1: self.optional_values(v, "fdir1", [0.; 3])?,
                    slip1: self.scalar(v, "slip1", 0.)?,
                    slip2: self.scalar(v, "slip2", 0.)?,
                    source: Some(self.source(v)),
                })
            })?;
            let bullet = child(n, "bullet")
                .map(|v| {
                    self.allowed(
                        v,
                        &["friction", "friction2", "fdir1", "rolling_friction"],
                        &[],
                    )?;
                    Ok(BulletFriction {
                        friction: self.nonnegative(v, "friction", 1.)?,
                        friction2: self.nonnegative(v, "friction2", 1.)?,
                        fdir1: self.optional_values(v, "fdir1", [0.; 3])?,
                        rolling_friction: self.scalar(v, "rolling_friction", 1.)?,
                        source: Some(self.source(v)),
                    })
                })
                .transpose()?;
            let torsional = child(n, "torsional")
                .map(|v| {
                    self.allowed(
                        v,
                        &[
                            "coefficient",
                            "use_patch_radius",
                            "patch_radius",
                            "surface_radius",
                            "ode",
                        ],
                        &[],
                    )?;
                    let ode_slip = child(v, "ode").map_or(Ok(0.), |o| {
                        self.allowed(o, &["slip"], &[])?;
                        self.scalar(o, "slip", 0.)
                    })?;
                    Ok(TorsionalFriction {
                        coefficient: self.nonnegative(v, "coefficient", 1.)?,
                        use_patch_radius: self.boolean(v, "use_patch_radius", true)?,
                        patch_radius: self.nonnegative(v, "patch_radius", 0.)?,
                        surface_radius: self.nonnegative(v, "surface_radius", 0.)?,
                        ode_slip,
                        source: Some(self.source(v)),
                    })
                })
                .transpose()?;
            Ok(Friction {
                ode,
                bullet,
                torsional,
                source: Some(self.source(n)),
            })
        })?;
        Ok(Surface {
            contact,
            friction,
            source: Some(self.source(node)),
        })
    }
    fn geometry(&self, geometry: Node<'_, '_>) -> Result<Geometry, Diagnostic> {
        self.allowed(
            geometry,
            &[
                "box",
                "sphere",
                "cylinder",
                "capsule",
                "cone",
                "ellipsoid",
                "mesh",
                "plane",
            ],
            &[],
        )?;
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
            "capsule" | "cone" => {
                self.allowed(shape, &["radius", "length"], &[])?;
                let default_radius = if shape.has_tag_name("capsule") {
                    0.5
                } else {
                    1.
                };
                let radius = self.required_scalar(shape, "radius", default_radius)?;
                let length = self.required_scalar(shape, "length", 1.)?;
                // Native DOM replaces nonpositive dimensions with these defaults.
                let radius = if radius > 0. { radius } else { 0.5 };
                let length = if length > 0. { length } else { 1. };
                if shape.has_tag_name("capsule") {
                    Geometry::Capsule { radius, length }
                } else {
                    Geometry::Cone { radius, length }
                }
            }
            "ellipsoid" => {
                self.allowed(shape, &["radii"], &[])?;
                let radii = child(shape, "radii").map_or(Ok([1.; 3]), |n| self.values(n))?;
                Geometry::Ellipsoid {
                    radii: if radii.iter().all(|r| *r > 0.) {
                        radii
                    } else {
                        [1.; 3]
                    },
                }
            }
            "mesh" => Geometry::Mesh(self.mesh(shape)?),
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
            Geometry::Cylinder { radius, length }
            | Geometry::Capsule { radius, length }
            | Geometry::Cone { radius, length } => *radius > 0. && *length > 0.,
            Geometry::Ellipsoid { radii } => radii.iter().all(|r| *r > 0.),
            Geometry::Mesh(_) => true,
            Geometry::Plane { size, .. } => size.iter().all(|v| *v > 0.),
        };
        if !positive {
            return Err(self.err(
                shape,
                ErrorCode::InvalidValue,
                "shape dimensions must be positive",
            ));
        }
        Ok(geometry)
    }
    fn required_scalar(
        &self,
        node: Node<'_, '_>,
        name: &str,
        default: f64,
    ) -> Result<f64, Diagnostic> {
        if let Some(value) = child(node, name)
            && param_text(value).is_empty()
        {
            return Err(self.err(value, ErrorCode::Missing, format!("missing {name}")));
        }
        self.scalar(node, name, default)
    }
    fn unsigned(&self, node: Node<'_, '_>, name: &str, default: u32) -> Result<u32, Diagnostic> {
        let Some(value) = child(node, name) else {
            return Ok(default);
        };
        self.allowed(value, &[], &[])?;
        let text = param_text(value);
        if text.is_empty() {
            return Ok(default);
        }
        let lower = text.to_ascii_lowercase();
        let text = match lower.as_str() {
            "true" => "1",
            "false" => "0",
            _ => text.as_ref(),
        };
        // Param.cc uses Linux stoul, then casts to unsigned int: a numeric
        // prefix is enough, negative values and the final u32 cast wrap.
        let radix = if lower.starts_with("0x") { 16 } else { 10 };
        let text = text.trim_start_matches(legacy_whitespace);
        let (negative, digits) = match text.as_bytes().first() {
            Some(b'-') => (true, &text[1..]),
            Some(b'+') => (false, &text[1..]),
            _ => (false, text),
        };
        let digits = if radix == 16 && digits.as_bytes().get(2).is_some_and(u8::is_ascii_hexdigit) {
            &digits[2..]
        } else {
            digits
        };
        let count = digits
            .bytes()
            .take_while(|b| {
                if radix == 16 {
                    b.is_ascii_hexdigit()
                } else {
                    b.is_ascii_digit()
                }
            })
            .count();
        let number = u64::from_str_radix(&digits[..count], radix)
            .map_err(|_| self.err(value, ErrorCode::InvalidValue, "expected unsigned integer"))?;
        Ok(if negative {
            number.wrapping_neg()
        } else {
            number
        } as u32)
    }
    fn mesh_string(
        &self,
        node: Node<'_, '_>,
        name: &str,
        default: &str,
    ) -> Result<String, Diagnostic> {
        let Some(value) = child(node, name) else {
            return Ok(default.into());
        };
        self.allowed(value, &[], &[])?;
        let text = param_text(value);
        if text.is_empty() {
            return Err(self.err(value, ErrorCode::Missing, format!("missing {name}")));
        }
        Ok(text.into_owned())
    }
    fn mesh(&self, shape: Node<'_, '_>) -> Result<Mesh, Diagnostic> {
        self.allowed(
            shape,
            &["uri", "scale", "submesh", "convex_decomposition"],
            &["optimization"],
        )?;
        let uri = self.mesh_string(shape, "uri", "__default__")?;
        let scale = child(shape, "scale").map_or(Ok([1.; 3]), |n| {
            self.allowed(n, &[], &[])?;
            if param_text(n).is_empty() {
                Ok([1.; 3])
            } else {
                self.values(n)
            }
        })?;
        let (submesh, center_submesh) = if let Some(submesh) = child(shape, "submesh") {
            self.allowed(submesh, &["name", "center"], &[])?;
            let name = self.mesh_string(submesh, "name", "__default__")?;
            if name == "__default__" {
                return Err(self.err(submesh, ErrorCode::Missing, "submesh requires a name"));
            }
            (name, self.boolean(submesh, "center", false)?)
        } else {
            (String::new(), false)
        };
        let optimization = match shape
            .attribute("optimization")
            .unwrap_or("")
            .trim_matches([' ', '\t', '\n'])
        {
            "convex_hull" => MeshOptimization::ConvexHull,
            "convex_decomposition" => MeshOptimization::ConvexDecomposition,
            _ => MeshOptimization::None,
        };
        let convex_decomposition = child(shape, "convex_decomposition")
            .map(|node| {
                self.allowed(node, &["max_convex_hulls", "voxel_resolution"], &[])?;
                Ok(ConvexDecomposition {
                    max_convex_hulls: self.unsigned(node, "max_convex_hulls", 16)?,
                    voxel_resolution: self.unsigned(node, "voxel_resolution", 200000)?,
                })
            })
            .transpose()?;
        Ok(Mesh {
            uri,
            scale,
            submesh,
            center_submesh,
            optimization,
            convex_decomposition,
            source: self.source(shape),
        })
    }
    fn visual(&self, node: Node<'_, '_>) -> Result<Visual, Diagnostic> {
        self.allowed(node, &["pose", "geometry", "plugin"], &["name"])?;
        let geometry = child(node, "geometry")
            .map(|n| {
                if elements(n).next().is_none() {
                    self.allowed(n, &[], &[])?;
                    Ok(None)
                } else {
                    self.geometry(n).map(Some)
                }
            })
            .transpose()?
            .flatten();
        Ok(Visual {
            name: self.name(node)?,
            pose: self.pose(node)?,
            geometry,
            plugins: self.plugins(node)?,
            source: self.source(node),
        })
    }
    fn joint(&self, node: Node<'_, '_>) -> Result<Joint, Diagnostic> {
        self.allowed(
            node,
            &["parent", "child", "pose", "axis", "axis2"],
            &["name", "type"],
        )?;
        let kind = match self.attr(node, "type")?.as_str() {
            "fixed" => JointKind::Fixed,
            "revolute" => JointKind::Revolute,
            "prismatic" => JointKind::Prismatic,
            "continuous" => JointKind::Continuous,
            "universal" => JointKind::Universal,
            "revolute2" => JointKind::Revolute2,
            _ => return Err(self.err(node, ErrorCode::Unsupported, "unsupported joint type")),
        };
        let axis = child(node, "axis").map(|n| self.axis(n)).transpose()?;
        let axis2 = child(node, "axis2").map(|n| self.axis(n)).transpose()?;
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
            axis2,
            source: self.source(node),
        })
    }
    fn axis(&self, axis: Node<'_, '_>) -> Result<Axis, Diagnostic> {
        self.allowed(
            axis,
            if is_16(axis) {
                &["xyz", "limit", "use_parent_model_frame"]
            } else {
                &["xyz", "limit"]
            },
            &[],
        )?;
        // Upstream's conversion table recognizes exactly these four spellings;
        // every other value is removed without being interpreted as a boolean.
        let parent_model = is_16(axis)
            && child(axis, "use_parent_model_frame").is_some_and(|n| {
                let text = n.text().unwrap_or("");
                // TinyXML's ordinary whitespace collapse does not apply to CDATA.
                let text = if n.text_is_cdata() {
                    text
                } else {
                    text.trim_matches(legacy_whitespace)
                };
                matches!(text, "true" | "True" | "TRUE" | "1")
            });
        if parent_model && child(axis, "xyz").is_none() {
            // The native conversion creates an empty required xyz element.
            return Err(self.err(
                axis,
                ErrorCode::Missing,
                "legacy axis conversion requires a nonempty xyz element",
            ));
        }
        let (xyz, mut expressed_in) = match child(axis, "xyz") {
            Some(xyz) => {
                self.allowed(xyz, &[], &["expressed_in"])?;
                (
                    self.values(xyz)?,
                    xyz.attribute("expressed_in").unwrap_or("").into(),
                )
            }
            None => ([0., 0., 1.], String::new()),
        };
        if parent_model {
            expressed_in = "__model__".into();
        }
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
    fn optional_text(&self, node: Node<'_, '_>, name: &str) -> Result<Option<String>, Diagnostic> {
        child(node, name)
            .map(|value| {
                self.allowed(value, &[], &[])?;
                Ok(param_text(value).into_owned())
            })
            .transpose()
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
                "frame_id",
                "gz_frame_id",
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
            topic: self
                .optional_text(node, "topic")?
                .filter(|topic| topic != "__default__")
                .unwrap_or_default(),
            frame_id: self.optional_text(node, "frame_id")?.unwrap_or_default(),
            gz_frame_id: self.optional_text(node, "gz_frame_id")?,
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
fn is_16(node: Node<'_, '_>) -> bool {
    node.document().root_element().attribute("version") == Some("1.6")
}
fn pose_attributes(node: Node<'_, '_>) -> &'static [&'static str] {
    if is_16(node) {
        &["frame", "relative_to", "degrees", "rotation_format"]
    } else {
        &["relative_to", "degrees", "rotation_format"]
    }
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
        for axis in joint.axis.iter().chain(&joint.axis2) {
            if !axis.expressed_in.is_empty() {
                graphs.validate_reference(&axis.expressed_in, scope, &joint.source)?;
            }
        }
    }
    for nested in &model.models {
        validate_model(nested, graphs)?;
    }
    Ok(())
}

// Utils.cc reserves a surrounding pair of double underscores, not every prefix.
fn reserved_name(name: &str) -> bool {
    name == "world" || (name.len() >= 4 && name.starts_with("__") && name.ends_with("__"))
}

// TinyXML collapses ordinary text before Param's space/tab/LF trim. CDATA
// skips that collapse, so VT/FF alone remain invalid numeric input.
fn param_text<'a>(node: Node<'a, '_>) -> Cow<'a, str> {
    let text = node.text().unwrap_or("");
    if node.text_is_cdata() {
        return Cow::Borrowed(text.trim_matches([' ', '\t', '\n']));
    }
    let text = text.trim_matches(legacy_whitespace);
    if text.contains("  ") || text.chars().any(|c| c != ' ' && legacy_whitespace(c)) {
        Cow::Owned(
            text.split(legacy_whitespace)
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join(" "),
        )
    } else {
        Cow::Borrowed(text)
    }
}
