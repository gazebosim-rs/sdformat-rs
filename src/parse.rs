// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::graph::{Graphs, error, join};
use crate::types::normalized_axis_direction;
use crate::xml::{Document, Node, legacy_whitespace};
use crate::*;
use glam::{DQuat, DVec3, EulerRot};
use std::{
    borrow::Cow,
    collections::BTreeSet,
    sync::{Arc, OnceLock},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum LightContext {
    Standalone,
    Document,
}

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
    parser.allowed(root, &["world", "model", "include"], &["version"])?;
    for child in elements(root) {
        if child.has_tag_name("world") {
            worlds.push(parser.world(child)?);
        } else if child.has_tag_name("model") {
            models.push(parser.model(child, "::", None)?);
        } else {
            let (model, merge) = parser.include(child, "::")?;
            if merge {
                return Err(parser.err(
                    child,
                    ErrorCode::Unsupported,
                    "merge include requires a model or world parent",
                ));
            }
            models.push(model);
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
    for world in &worlds {
        validate_joints(&world.joints, &world.name, &world.name, "world", &graphs)?;
    }
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

pub(crate) fn load_sensor_element_into(
    sensor: &mut Sensor,
    xml: &str,
    uri: &str,
    resolver: &dyn ResourceResolver,
) -> Result<(), Diagnostic> {
    let parser = Parser {
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
    if !root.has_tag_name("sensor") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be sensor"));
    }
    if root.attribute("type") == Some("none") {
        // Native Sensor::Load updates these envelope values before reporting
        // ATTRIBUTE_INVALID for the special type-none element.
        sensor.name = parser.name(root)?;
        sensor.source = parser.source(root);
        sensor.frame_id = parser.optional_text(root, "frame_id")?.unwrap_or_default();
        sensor.update_rate = parser.scalar(root, "update_rate", sensor.update_rate)?;
        sensor.topic = parser
            .optional_text(root, "topic")?
            .filter(|topic| topic != "__default__")
            .unwrap_or_default();
        sensor.enable_metrics = parser.boolean(root, "enable_metrics", sensor.enable_metrics)?;
        return Err(parser.err(root, ErrorCode::InvalidValue, "sensor type none is invalid"));
    }
    *sensor = parser.sensor(root)?;
    Ok(())
}

pub(crate) fn load_light_element(xml: &str, uri: &str) -> Result<Light, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
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
    if !root.has_tag_name("light") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be light"));
    }
    parser.light(root)
}

pub(crate) fn load_frame_element(xml: &str, uri: &str) -> Result<Frame, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
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
    if !root.has_tag_name("frame") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be frame"));
    }
    parser.frame(root)
}

pub(crate) fn load_surface_element(xml: &str, uri: &str) -> Result<Surface, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
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
    if !root.has_tag_name("surface") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be surface"));
    }
    parser.surface(root)
}

pub(crate) fn load_mesh_element(xml: &str, uri: &str) -> Result<Mesh, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
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
    if !root.has_tag_name("mesh") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be mesh"));
    }
    parser.mesh(root)
}
pub(crate) fn load_heightmap_element(xml: &str, uri: &str) -> Result<Heightmap, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
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
    if !root.has_tag_name("heightmap") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be heightmap"));
    }
    parser.heightmap(root)
}
pub(crate) fn load_polyline_element(xml: &str, uri: &str) -> Result<Polyline, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
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
    if !root.has_tag_name("polyline") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be polyline"));
    }
    parser.polyline(root)
}
pub(crate) fn load_material_element(xml: &str, uri: &str) -> Result<Material, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
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
    if !root.has_tag_name("material") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be material"));
    }
    parser.material(root)
}
pub(crate) fn load_collision_element(xml: &str, uri: &str) -> Result<Collision, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
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
    if !root.has_tag_name("collision") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be collision"));
    }
    parser.collision(root)
}
pub(crate) fn load_visual_element(xml: &str, uri: &str) -> Result<Visual, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
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
    if !root.has_tag_name("visual") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be visual"));
    }
    parser.visual(root)
}
pub(crate) fn load_atmosphere_element(xml: &str, uri: &str) -> Result<Atmosphere, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
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
    if !root.has_tag_name("atmosphere") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be atmosphere"));
    }
    parser.atmosphere(root)
}
pub(crate) fn load_plugin_element(xml: &str, uri: &str) -> Result<Plugin, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
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
    if !root.has_tag_name("plugin") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be plugin"));
    }
    parser.plugin(root)
}

pub(crate) fn load_geometry_element(xml: &str, uri: &str) -> Result<Geometry, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
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
    if !root.has_tag_name("geometry") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be geometry"));
    }
    parser.geometry(root)
}

pub(crate) fn load_axis_element(xml: &str, uri: &str) -> Result<Axis, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
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
    if !root.has_tag_name("axis") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be axis"));
    }
    let mut axis = parser.axis(root, false, true)?;
    trim_axis_parameters(&mut axis);
    Ok(axis)
}

pub(crate) fn load_force_torque_element(xml: &str, uri: &str) -> Result<ForceTorque, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
        includes: vec![uri.into()],
    };
    let document = Document::parse(xml).map_err(|error| Diagnostic {
        code: error.code,
        source: Source {
            uri: uri.into(),
            line: error.pos().row,
            column: error.pos().col,
        },
        message: error.to_string(),
    })?;
    let root = document.root_element();
    if !root.has_tag_name("force_torque") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be force_torque"));
    }
    parser.force_torque_node(root)
}

pub(crate) fn load_imu_element(xml: &str, uri: &str) -> Result<Imu, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
        includes: vec![uri.into()],
    };
    let document = Document::parse(xml).map_err(|error| Diagnostic {
        code: error.code,
        source: Source {
            uri: uri.into(),
            line: error.pos().row,
            column: error.pos().col,
        },
        message: error.to_string(),
    })?;
    let root = document.root_element();
    if !root.has_tag_name("imu") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be imu"));
    }
    parser.imu_node(root)
}

pub(crate) fn load_altimeter_element(xml: &str, uri: &str) -> Result<Altimeter, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
        includes: vec![uri.into()],
    };
    let document = Document::parse(xml).map_err(|error| Diagnostic {
        code: error.code,
        source: Source {
            uri: uri.into(),
            line: error.pos().row,
            column: error.pos().col,
        },
        message: error.to_string(),
    })?;
    let root = document.root_element();
    if !root.has_tag_name("altimeter") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be altimeter"));
    }
    parser.altimeter_node(root)
}

pub(crate) fn load_contact_sensor_element(
    xml: &str,
    uri: &str,
) -> Result<ContactSensor, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
        includes: vec![uri.into()],
    };
    let document = Document::parse(xml).map_err(|error| Diagnostic {
        code: error.code,
        source: Source {
            uri: uri.into(),
            line: error.pos().row,
            column: error.pos().col,
        },
        message: error.to_string(),
    })?;
    let root = document.root_element();
    if !root.has_tag_name("contact") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be contact"));
    }
    parser.contact_node(root)
}

pub(crate) fn load_lidar_element(xml: &str, uri: &str) -> Result<Lidar, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
        includes: vec![uri.into()],
    };
    let document = Document::parse(xml).map_err(|error| Diagnostic {
        code: error.code,
        source: Source {
            uri: uri.into(),
            line: error.pos().row,
            column: error.pos().col,
        },
        message: error.to_string(),
    })?;
    let root = document.root_element();
    if !root.has_tag_name("lidar") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be lidar"));
    }
    parser.lidar_node(root)
}

pub(crate) fn load_air_speed_element(xml: &str, uri: &str) -> Result<AirSpeed, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
        includes: vec![uri.into()],
    };
    let document = Document::parse(xml).map_err(|error| Diagnostic {
        code: error.code,
        source: Source {
            uri: uri.into(),
            line: error.pos().row,
            column: error.pos().col,
        },
        message: error.to_string(),
    })?;
    let root = document.root_element();
    if !root.has_tag_name("air_speed") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be air_speed"));
    }
    parser.air_speed_node(root)
}

pub(crate) fn load_navsat_element(xml: &str, uri: &str) -> Result<NavSat, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
        includes: vec![uri.into()],
    };
    let document = Document::parse(xml).map_err(|error| Diagnostic {
        code: error.code,
        source: Source {
            uri: uri.into(),
            line: error.pos().row,
            column: error.pos().col,
        },
        message: error.to_string(),
    })?;
    let root = document.root_element();
    if !root.has_tag_name("navsat") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be navsat"));
    }
    parser.navsat_node(root)
}

pub(crate) fn load_magnetometer_element(xml: &str, uri: &str) -> Result<Magnetometer, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
        includes: vec![uri.into()],
    };
    let document = Document::parse(xml).map_err(|error| Diagnostic {
        code: error.code,
        source: Source {
            uri: uri.into(),
            line: error.pos().row,
            column: error.pos().col,
        },
        message: error.to_string(),
    })?;
    let root = document.root_element();
    if !root.has_tag_name("magnetometer") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be magnetometer"));
    }
    parser.magnetometer_node(root)
}

pub(crate) fn load_air_pressure_element(xml: &str, uri: &str) -> Result<AirPressure, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
        includes: vec![uri.into()],
    };
    let document = Document::parse(xml).map_err(|error| Diagnostic {
        code: error.code,
        source: Source {
            uri: uri.into(),
            line: error.pos().row,
            column: error.pos().col,
        },
        message: error.to_string(),
    })?;
    let root = document.root_element();
    if !root.has_tag_name("air_pressure") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be air_pressure"));
    }
    parser.air_pressure_node(root)
}

pub(crate) fn load_camera_element(xml: &str, uri: &str) -> Result<Camera, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
        includes: vec![uri.into()],
    };
    let document = Document::parse(xml).map_err(|error| Diagnostic {
        code: error.code,
        source: Source {
            uri: uri.into(),
            line: error.pos().row,
            column: error.pos().col,
        },
        message: error.to_string(),
    })?;
    let root = document.root_element();
    if !root.has_tag_name("camera") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be camera"));
    }
    let camera = parser.camera_node(root)?;
    crate::camera_xml::validate(&camera).map_err(|mut error| {
        error.source = parser.source(root);
        error
    })?;
    Ok(camera)
}

fn trim_axis_parameters(axis: &mut Axis) {
    // Native string Params trim padding; unattached names are not graph queries.
    axis.expressed_in = axis.expressed_in.trim_matches([' ', '\t', '\n']).into();
    if let Some(mimic) = &mut axis.mimic {
        mimic.joint = mimic.joint.trim_matches([' ', '\t', '\n']).into();
        mimic.axis = mimic.axis.trim_matches([' ', '\t', '\n']).into();
    }
}

pub(crate) fn load_joint_element(xml: &str, uri: &str) -> Result<Joint, Diagnostic> {
    let resolver = NoResources;
    let parser = Parser {
        uri,
        xml,
        resolver: &resolver,
        includes: vec![uri.into()],
    };
    let doc = Document::parse(xml).map_err(|error| Diagnostic {
        code: error.code,
        source: Source {
            uri: uri.into(),
            line: error.pos().row,
            column: error.pos().col,
        },
        message: error.to_string(),
    })?;
    let root = doc.root_element();
    if !root.has_tag_name("joint") {
        return Err(parser.err(root, ErrorCode::Xml, "root element must be joint"));
    }
    if child(root, "physics").is_some()
        || elements(root)
            .filter(|node| node.has_tag_name("sensor"))
            .any(|sensor| sensor.attribute("type").map(str::trim) != Some("force_torque"))
    {
        return Err(parser.err(
            root,
            ErrorCode::Unsupported,
            "standalone joint XML supports only force/torque sensors without physics payloads",
        ));
    }
    let mut joint = parser.joint(root, false, true)?;
    // Param text canonicalization is separate from resolving these references.
    joint.parent =
        param_text(child(root, "parent").expect("joint parser requires parent")).into_owned();
    joint.child =
        param_text(child(root, "child").expect("joint parser requires child")).into_owned();
    joint.pose.relative_to = joint
        .pose
        .relative_to
        .trim_matches([' ', '\t', '\n'])
        .into();
    for axis in [&mut joint.axis, &mut joint.axis2].into_iter().flatten() {
        trim_axis_parameters(axis);
    }
    crate::joint_xml::validate_boundary(&joint).map_err(|mut error| {
        error.source = parser.source(root);
        error
    })?;
    Ok(joint)
}

struct Parser<'a> {
    uri: &'a str,
    xml: &'a str,
    resolver: &'a dyn ResourceResolver,
    includes: Vec<String>,
}
impl Parser<'_> {
    fn source(&self, node: Node<'_, '_>) -> Source {
        let pos = node.position();
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
            Some("1.4" | "1.5" | "1.6" | "1.7" | "1.8" | "1.9" | "1.10" | "1.11" | "1.12")
        ) {
            return Err(self.err(root, ErrorCode::Version, "supported versions are SDF 1.4 through 1.12; earlier conversion is not implemented"));
        }
        Ok(())
    }
    fn allowed(
        &self,
        node: Node<'_, '_>,
        children: &[&str],
        attrs: &[&str],
    ) -> Result<(), Diagnostic> {
        self.allowed_repeated(node, children, attrs, &[])
    }
    fn allowed_repeated(
        &self,
        node: Node<'_, '_>,
        children: &[&str],
        attrs: &[&str],
        extra_repeated: &[&str],
    ) -> Result<(), Diagnostic> {
        let extra_repeated: BTreeSet<_> = extra_repeated.iter().copied().collect();
        for attr in node.attributes() {
            if !attrs.contains(&attr.name()) {
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
            // Additional names are admitted only when the caller lists them.
            let name = child.tag_name().name();
            if !(children.contains(&name) || extra_repeated.contains(name)) {
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
            "light",
            "plugin",
            "physics",
            "polyline",
            "point",
            "texture",
            "blend",
        ];
        let mut seen = BTreeSet::new();
        for child in elements(node) {
            let name = child.tag_name().name();
            if !repeated.contains(&name) && !extra_repeated.contains(name) && !seen.insert(name) {
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
                Some("1.4" | "1.5" | "1.6" | "1.7")
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
        let relative_to = if is_pre17(pose) {
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
            .map(|p| self.plugin(p))
            .collect()
    }
    fn plugin(&self, node: Node<'_, '_>) -> Result<Plugin, Diagnostic> {
        let filename = node
            .attribute("filename")
            .map(|value| value.trim_matches([' ', '\t', '\n']))
            .filter(|value| !value.is_empty())
            .ok_or_else(|| self.err(node, ErrorCode::Missing, "missing filename attribute"))?;
        Ok(Plugin {
            name: node
                .attribute("name")
                .unwrap_or("")
                .trim_matches([' ', '\t', '\n'])
                .into(),
            filename: filename.into(),
            xml: self.xml[node.range()].into(),
        })
    }

    fn world(&mut self, node: Node<'_, '_>) -> Result<World, Diagnostic> {
        let extension_names: Vec<_> = elements(node)
            .map(|member| member.tag_name().name())
            .filter(|name| name.contains(':') || !known_schema_name(name))
            .collect();
        self.allowed_repeated(
            node,
            &[
                "model",
                "light",
                "include",
                "frame",
                "joint",
                "gravity",
                "wind",
                "magnetic_field",
                "plugin",
                "physics",
                "atmosphere",
                "scene",
            ],
            &["name"],
            &extension_names,
        )?;
        let name = self.name(node)?;
        let gravity = child(node, "gravity").map_or(Ok([0., 0., -9.8]), |n| self.values(n))?;
        let wind_linear_velocity = child(node, "wind").map_or(Ok([0.; 3]), |wind| {
            self.allowed(wind, &["linear_velocity"], &[])?;
            child(wind, "linear_velocity").map_or(Ok([0.; 3]), |velocity| {
                self.allowed(velocity, &[], &[])?;
                if param_text(velocity).is_empty() {
                    Ok([0.; 3])
                } else {
                    self.values(velocity)
                }
            })
        })?;
        let magnetic_field = child(node, "magnetic_field")
            .map_or(Ok([5.5645e-6, 22.8758e-6, -42.3884e-6]), |n| self.values(n))?;
        let mut physics = elements(node)
            .filter(|n| n.has_tag_name("physics"))
            .map(|n| self.physics(n))
            .collect::<Result<Vec<_>, _>>()?;
        let mut names = BTreeSet::new();
        if physics.iter().any(|profile| !names.insert(&profile.name)) {
            return Err(self.err(
                node,
                ErrorCode::DuplicateName,
                "duplicate physics profile name",
            ));
        }
        if physics.is_empty() {
            physics.push(Physics {
                name: "default_physics".into(),
                ..Physics::default()
            });
        }
        let mut models = Vec::new();
        let mut lights = Vec::new();
        let mut frames = Vec::new();
        let mut frame_elements = Vec::new();
        let mut joints = Vec::new();
        let mut plugins = Vec::new();
        let mut extensions = Vec::new();
        let mut direct_extensions = Vec::new();
        let mut merged_includes = Vec::new();
        for member in elements(node) {
            match member.tag_name().name() {
                "model" | "include" => {
                    let (model, merge) = if member.has_tag_name("model") {
                        (self.model(member, &name, None)?, false)
                    } else {
                        self.include(member, &name)?
                    };
                    if merge {
                        let (imported, proxy, provenance) =
                            self.merge_model(model, &name, "world")?;
                        // World has no direct links. Fail atomically with the
                        // native first-error category, before building world graphs.
                        if !imported.links.is_empty() {
                            return Err(self.err(
                                member,
                                ErrorCode::MergeIncludeUnsupported,
                                "world merge include cannot contain direct links",
                            ));
                        }
                        frame_elements
                            .extend(std::iter::repeat_n(member, 1 + imported.frames.len()));
                        frames.push(proxy);
                        frames.extend(imported.frames);
                        joints.extend(imported.joints);
                        models.extend(imported.models);
                        plugins.extend(imported.plugins);
                        extensions.extend(
                            imported
                                .extensions
                                .into_iter()
                                .filter(|e| e.tag.contains(':')),
                        );
                        merged_includes.push(provenance);
                        merged_includes.extend(imported.merged_includes);
                    } else {
                        models.push(model);
                    }
                }
                "light" => lights.push(self.light_in(member, LightContext::Document)?),
                "frame" => {
                    frames.push(self.frame(member)?);
                    frame_elements.push(member);
                }
                "joint" => joints.push(self.joint(member, false, supports_axis_mimic(member))?),
                "plugin" => plugins.push(self.plugin(member)?),
                _ if extension_names.contains(&member.tag_name().name()) => {
                    direct_extensions.push(self.extension_metadata(member));
                }
                _ => {}
            }
        }
        // Native merge expansion precedes direct world metadata in the Element.
        extensions.extend(direct_extensions);
        if is_pre17(node) {
            // World reserves every model name before renaming legacy frames.
            let mut names = models.iter().map(|model| model.name.clone()).collect();
            let mut typed_names = BTreeSet::new();
            for (frame, element) in frames.iter_mut().zip(frame_elements) {
                frame.name =
                    self.legacy_name(element, "frame", &frame.name, &mut names, &mut typed_names)?;
            }
        }
        if lights
            .iter()
            .map(|v| &v.name)
            .collect::<BTreeSet<_>>()
            .len()
            != lights.len()
        {
            return Err(self.err(node, ErrorCode::DuplicateName, "duplicate light name"));
        }
        self.validate_axis_mimics(&name, &joints)?;
        Ok(World {
            scene: self.scene(node)?,
            name,
            gravity,
            wind_linear_velocity,
            magnetic_field,
            atmosphere: child(node, "atmosphere")
                .map_or(Ok(Atmosphere::default()), |n| self.atmosphere(n))?,
            physics,
            models,
            lights,
            joints,
            frames,
            merged_includes,
            plugins,
            extensions,
            source: self.source(node),
        })
    }
    fn scene(&self, world: Node<'_, '_>) -> Result<Scene, Diagnostic> {
        let Some(node) = child(world, "scene") else {
            return Ok(Scene::default());
        };
        self.allowed(
            node,
            &["ambient", "background", "grid", "shadows", "origin_visual"],
            &[],
        )?;
        // Required schema parameters may be omitted for defaults, but an
        // explicit empty value fails before native Scene::Load is reached.
        for name in ["ambient", "background", "shadows"] {
            if let Some(value) = child(node, name)
                && param_text(value).is_empty()
            {
                self.allowed(value, &[], &[])?;
                return Err(self.err(
                    value,
                    ErrorCode::InvalidValue,
                    format!("empty required scene {name} value"),
                ));
            }
        }
        let defaults = Scene::default();
        Ok(Scene {
            ambient: if child(node, "ambient").is_some() {
                self.color(node, "ambient")?
            } else {
                defaults.ambient
            },
            background: if child(node, "background").is_some() {
                self.color(node, "background")?
            } else {
                defaults.background
            },
            grid: self.boolean(node, "grid", defaults.grid)?,
            shadows: self.boolean(node, "shadows", defaults.shadows)?,
            origin_visual: self.boolean(node, "origin_visual", defaults.origin_visual)?,
            source: Some(self.source(node)),
        })
    }
    fn atmosphere(&self, node: Node<'_, '_>) -> Result<Atmosphere, Diagnostic> {
        self.allowed(
            node,
            &["temperature", "pressure", "temperature_gradient"],
            &["type"],
        )?;
        let kind = node
            .attribute("type")
            .map(|value| value.trim_matches([' ', '\t', '\n']))
            .filter(|value| !value.is_empty())
            .ok_or_else(|| self.err(node, ErrorCode::Missing, "atmosphere requires type"))?;
        if kind != "adiabatic" {
            return Err(self.err(node, ErrorCode::InvalidValue, "unknown atmosphere type"));
        }
        Ok(Atmosphere {
            atmosphere_type: AtmosphereType::Adiabatic,
            temperature_kelvin: self.scalar(node, "temperature", 288.15)?,
            temperature_gradient: self.scalar(node, "temperature_gradient", -0.0065)?,
            pressure: self.scalar(node, "pressure", 101325.)?,
            source: Some(self.source(node)),
            xml: Some(self.xml[node.range()].into()),
        })
    }
    fn physics(&self, node: Node<'_, '_>) -> Result<Physics, Diagnostic> {
        self.allowed(
            node,
            &[
                "max_step_size",
                "real_time_factor",
                "real_time_update_rate",
                "max_contacts",
            ],
            &["name", "default", "type"],
        )?;
        let name = node
            .attribute("name")
            .unwrap_or("")
            .trim_matches([' ', '\t', '\n']);
        let name = if name.is_empty() {
            "default_physics"
        } else {
            name
        };
        if reserved_name(name) || name.contains("::") {
            return Err(self.err(
                node,
                ErrorCode::InvalidValue,
                "reserved physics profile name",
            ));
        }
        let engine = node
            .attribute("type")
            .unwrap_or("")
            .trim_matches([' ', '\t', '\n']);
        if engine.is_empty() {
            return Err(self.err(node, ErrorCode::Missing, "physics requires type"));
        }
        let is_default = match node
            .attribute("default")
            .unwrap_or("")
            .trim_matches([' ', '\t', '\n'])
            .to_ascii_lowercase()
            .as_str()
        {
            "" | "false" | "0" => false,
            "true" | "1" => true,
            _ => {
                return Err(self.err(
                    node,
                    ErrorCode::InvalidValue,
                    "invalid default physics boolean",
                ));
            }
        };
        Ok(Physics {
            name: name.into(),
            is_default,
            engine_type: engine.into(),
            max_step_size: self.required_scalar(node, "max_step_size", 0.001)?,
            real_time_factor: self.required_scalar(node, "real_time_factor", 1.)?,
            real_time_update_rate: self.required_scalar(node, "real_time_update_rate", 1000.)?,
            max_contacts: self.signed_integer(node, "max_contacts", 20)?,
            source: Some(self.source(node)),
            xml: Some(self.xml[node.range()].into()),
        })
    }
    fn signed_integer(
        &self,
        node: Node<'_, '_>,
        name: &str,
        default: i32,
    ) -> Result<i32, Diagnostic> {
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
        // Param's stoi chooses hexadecimal only for a leading 0x before sign
        // handling; otherwise it accepts a decimal prefix and checks i32 range.
        let radix = if lower.starts_with("0x") { 16 } else { 10 };
        let text = text.trim_start_matches(legacy_whitespace);
        let (sign, digits) = match text.as_bytes().first() {
            Some(b'-') => (-1_i64, &text[1..]),
            Some(b'+') => (1, &text[1..]),
            _ => (1, text),
        };
        let digits = if radix == 16 && digits.as_bytes().get(2).is_some_and(u8::is_ascii_hexdigit) {
            &digits[2..]
        } else {
            digits
        };
        let count = digits
            .bytes()
            .take_while(|c| {
                if radix == 16 {
                    c.is_ascii_hexdigit()
                } else {
                    c.is_ascii_digit()
                }
            })
            .count();
        i64::from_str_radix(&digits[..count], radix)
            .ok()
            .and_then(|n| i32::try_from(n * sign).ok())
            .ok_or_else(|| self.err(value, ErrorCode::InvalidValue, "expected signed integer"))
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
        // The canonical model schema distinguishes unimplemented declarations
        // from unknown XML. Default native policy retains unknown children.
        // Never hide a known schema feature inside opaque extension metadata.
        let extension_names: Vec<_> = elements(node)
            .map(|member| member.tag_name().name())
            .filter(|name| {
                !matches!(
                    *name,
                    "pose"
                        | "static"
                        | "enable_wind"
                        | "link"
                        | "model"
                        | "include"
                        | "frame"
                        | "joint"
                        | "plugin"
                        | "self_collide"
                        | "allow_auto_disable"
                        | "gripper"
                        | "model_state"
                )
            })
            .collect();
        let mut children = vec![
            "pose",
            "static",
            "enable_wind",
            "link",
            "model",
            "include",
            "frame",
            "joint",
            "plugin",
        ];
        children.extend(extension_names.iter().copied());
        self.allowed_repeated(
            node,
            &children,
            &["name", "namespace", "canonical_link", "placement_frame"],
            &extension_names,
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
        let mut plugins = Vec::new();
        let mut extensions = Vec::new();
        let mut merged_includes = Vec::new();
        let mut element_children = Vec::new();
        let mut names = BTreeSet::new();
        let mut typed_names = BTreeSet::new();
        // Legacy collisions rename only the element loaded later. Future names
        // must not be reserved early; references retain their declared strings.
        for member in elements(node) {
            let mut element_child = ModelElementChild {
                tag: member.tag_name().name().into(),
                name_attribute: member.attribute("name").map(str::to_owned),
            };
            match member.tag_name().name() {
                "link" => {
                    let mut link = self.link(member, &scoped_name)?;
                    if is_pre17(node) {
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
                    let mut joint = self.joint(
                        member,
                        converted_14_model(node),
                        supports_axis_mimic(member),
                    )?;
                    if is_pre17(node) {
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
                    if is_pre17(node) {
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
                    let (nested, merge) = if member.has_tag_name("model") {
                        (self.model(member, &scoped_name, None)?, false)
                    } else {
                        self.include(member, &scoped_name)?
                    };
                    if merge {
                        let (imported, proxy, provenance) =
                            self.merge_model(nested, &scoped_name, "__model__")?;
                        if is_pre17(node) {
                            for (kind, name) in std::iter::once(("frame", &proxy.name))
                                .chain(imported.links.iter().map(|v| ("link", &v.name)))
                                .chain(imported.joints.iter().map(|v| ("joint", &v.name)))
                                .chain(imported.frames.iter().map(|v| ("frame", &v.name)))
                                .chain(imported.models.iter().map(|v| ("model", &v.name)))
                            {
                                if names.contains(name) {
                                    return Err(self.err(
                                        member,
                                        ErrorCode::Unsupported,
                                        "legacy merge name collisions are not implemented",
                                    ));
                                }
                                names.insert(name.clone());
                                typed_names.insert((kind, name.clone()));
                            }
                        }
                        append_merged_element_children(
                            &mut element_children,
                            &imported,
                            &proxy.name,
                        );
                        links.extend(imported.links);
                        joints.extend(imported.joints);
                        frames.push(proxy);
                        frames.extend(imported.frames);
                        models.extend(imported.models);
                        plugins.extend(imported.plugins);
                        // Native merge copies namespace-qualified extension tags,
                        // regardless of a name attribute. Other unknowns stay local.
                        extensions.extend(
                            imported
                                .extensions
                                .into_iter()
                                .filter(|e| e.tag.contains(':')),
                        );
                        merged_includes.push(provenance);
                        merged_includes.extend(imported.merged_includes);
                        continue;
                    }
                    if is_pre17(node) {
                        self.legacy_name(
                            member,
                            "model",
                            &nested.name,
                            &mut names,
                            &mut typed_names,
                        )?;
                    }
                    element_child.tag = "model".into();
                    element_child.name_attribute = Some(nested.name.clone());
                    models.push(nested);
                }
                "plugin" => plugins.push(self.plugin(member)?),
                _ if extension_names.contains(&member.tag_name().name()) => {
                    extensions.push(self.extension_metadata(member));
                }
                _ => {}
            }
            element_children.push(element_child);
        }
        self.validate_axis_mimics(&name, &joints)?;
        Ok(Model {
            name,
            scoped_name: scoped_name.clone(),
            raw_namespace: node
                .attribute("namespace")
                .map(|value| value.trim_matches([' ', '\t', '\n']))
                .filter(|value| !value.is_empty())
                .map(str::to_owned),
            pose,
            is_static: self.boolean(node, "static", false)?,
            enable_wind: self.boolean(node, "enable_wind", false)?,
            extensions,
            loaded_element: Some(Arc::new(ModelElement {
                children: element_children,
            })),
            canonical_link: node.attribute("canonical_link").unwrap_or("").into(),
            placement_frame: node.attribute("placement_frame").unwrap_or("").into(),
            include: None,
            merged_includes,
            links,
            joints,
            models,
            frames,
            plugins,
            source: self.source(node),
        })
    }
    fn extension_metadata(&self, node: Node<'_, '_>) -> ExtensionMetadata {
        ExtensionMetadata {
            tag: node.tag_name().name().into(),
            attributes: node
                .attributes()
                .map(|attribute| {
                    let name = attribute.name();
                    (
                        name.into(),
                        node.attribute(name)
                            .expect("attribute enumerated by this node")
                            .into(),
                    )
                })
                .collect(),
            xml: self.xml[node.range()].into(),
            source: self.source(node),
        }
    }
    fn include(&mut self, node: Node<'_, '_>, scope: &str) -> Result<(Model, bool), Diagnostic> {
        self.allowed(
            node,
            &[
                "uri",
                "name",
                "namespace",
                "pose",
                "static",
                "placement_frame",
                "plugin",
            ],
            &["merge"],
        )?;
        let merge = self.merge_attribute(node)?;
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
        // The resource is a standalone document before include overrides are
        // applied. Its root pose cannot acquire the including parent's scope.
        if !imported.pose(all[0])?.relative_to.is_empty() {
            return Err(imported.err(
                all[0],
                ErrorCode::InvalidValue,
                "top-level model pose relative_to must be empty",
            ));
        }
        let model = imported.model(all[0], scope, rename.as_deref())?;
        self.apply_include_overrides(node, model, uri, rename, merge)
    }
    // Applying overrides after recursive loading keeps their temporaries off
    // the stack while loading deeply nested resources.
    fn apply_include_overrides(
        &self,
        node: Node<'_, '_>,
        mut model: Model,
        uri: String,
        rename: Option<String>,
        merge: bool,
    ) -> Result<(Model, bool), Diagnostic> {
        // Native empty namespace overrides leave the resource's value intact.
        if let Some(namespace) = self
            .optional_text(node, "namespace")?
            .filter(|value| !value.is_empty())
        {
            model.raw_namespace = Some(namespace);
        }
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
        if let Some(element) = &mut model.loaded_element {
            let element = Arc::make_mut(element);
            if pose.is_some() {
                element.ensure_child("pose");
            }
            if is_static.is_some() {
                element.ensure_child("static");
            }
            element
                .children
                .extend(plugins.iter().map(|plugin| ModelElementChild {
                    tag: "plugin".into(),
                    name_attribute: Some(plugin.name.clone()),
                }));
        }
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
        Ok((model, merge))
    }
    fn merge_attribute(&self, node: Node<'_, '_>) -> Result<bool, Diagnostic> {
        let Some(raw) = node.attribute("merge") else {
            return Ok(false);
        };
        let trimmed = raw.trim_matches([' ', '\t', '\n']);
        if !matches!(
            trimmed.to_ascii_lowercase().as_str(),
            "" | "true" | "false" | "1" | "0"
        ) {
            return Err(self.err(node, ErrorCode::InvalidValue, "invalid merge boolean"));
        }
        // Schema validation trims the attribute, but TinyXML BoolAttribute uses
        // the original spelling: words with surrounding whitespace default false.
        Ok(raw.eq_ignore_ascii_case("true") || trimmed == "1")
    }
    fn merge_model(
        &self,
        mut model: Model,
        scope: &str,
        parent_frame: &str,
    ) -> Result<(Model, Frame, MergedInclude), Diagnostic> {
        let pose = std::mem::take(&mut model.pose);
        let placement = std::mem::take(&mut model.placement_frame);
        // Validate the imported scope before flattening. Otherwise an invalid
        // imported reference might accidentally resolve to a parent's entity.
        let graphs = Graphs::build(&[], std::slice::from_ref(&model), &model.source)?;
        validate_model(&model, &graphs)?;
        let mut transform = pose.transform;
        if !placement.is_empty() {
            let frame = graphs.validate_reference(&placement, &model.scoped_name, &model.source)?;
            let offset = graphs.pose_of(&frame, &model.scoped_name)?;
            transform = transform.compose(offset.inverse());
        }
        let proxy = format!("_merged__{}__model__", model.name);
        let include = model
            .include
            .take()
            .expect("merge_model receives an expanded include");
        let frame = Frame {
            name: proxy.clone(),
            attached_to: model.canonical_link_and_relative_name().1,
            pose: Pose {
                transform,
                relative_to: if pose.relative_to.is_empty() {
                    parent_frame.into()
                } else {
                    pose.relative_to
                },
            },
            source: include.source.clone(),
        };
        let provenance = MergedInclude {
            include,
            model_source: model.source.clone(),
            proxy_frame: proxy.clone(),
        };
        let rewrite = |value: &mut String, empty: bool| {
            if *value == "__model__" || (empty && value.is_empty()) {
                *value = proxy.clone();
            }
        };
        for link in &mut model.links {
            link.scoped_name = join(scope, &link.name);
            rewrite(&mut link.pose.relative_to, true);
        }
        for nested in &mut model.models {
            // Native merge creates a pose even when the nested model omitted it.
            if let Some(element) = &mut nested.loaded_element {
                Arc::make_mut(element).ensure_child("pose");
            }
            rewrite(&mut nested.pose.relative_to, true);
            rescope_model(nested, scope);
        }
        for frame in &mut model.frames {
            rewrite(&mut frame.attached_to, true);
            rewrite(&mut frame.pose.relative_to, false);
        }
        for joint in &mut model.joints {
            rewrite(&mut joint.pose.relative_to, false);
            rewrite(&mut joint.parent, false);
            rewrite(&mut joint.child, false);
            for axis in joint.axis.iter_mut().chain(&mut joint.axis2) {
                rewrite(&mut axis.expressed_in, false);
            }
        }
        Ok((model, frame, provenance))
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
                "light",
                "plugin",
                "enable_wind",
                "gravity",
                "kinematic",
                "self_collide",
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
        let lights = elements(node)
            .filter(|n| n.has_tag_name("light"))
            .map(|n| self.light_in(n, LightContext::Document))
            .collect::<Result<Vec<_>, _>>()?;
        if lights
            .iter()
            .map(|v| &v.name)
            .collect::<BTreeSet<_>>()
            .len()
            != lights.len()
        {
            return Err(self.err(node, ErrorCode::DuplicateName, "duplicate light name"));
        }
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
            lights,
            particle_emitters: Vec::new(),
            projectors: Vec::new(),
            sensors,
            properties: LinkProperties {
                enable_wind: self.boolean(node, "enable_wind", false)?,
                enable_gravity: self.boolean(node, "gravity", true)?,
                kinematic: self.boolean(node, "kinematic", false)?,
                self_collide: self.boolean(node, "self_collide", false)?,
            },
            plugins: self.plugins(node)?,
            source: self.source(node),
        })
    }
    fn light(&self, node: Node<'_, '_>) -> Result<Light, Diagnostic> {
        self.light_in(node, LightContext::Standalone)
    }
    fn light_color(
        &self,
        node: Node<'_, '_>,
        name: &str,
        schema_default: [f32; 4],
        context: LightContext,
    ) -> Result<[f32; 4], Diagnostic> {
        let value = self.color(node, name)?;
        if context == LightContext::Document
            && child(node, name).is_none_or(|n| param_text(n).is_empty())
        {
            Ok(schema_default)
        } else {
            Ok(value)
        }
    }
    fn light_required_scalar(
        &self,
        node: Node<'_, '_>,
        name: &str,
        default: f64,
        context: LightContext,
    ) -> Result<f64, Diagnostic> {
        let value = self.scalar(node, name, default)?;
        if context == LightContext::Document
            && let Some(n) = child(node, name).filter(|n| param_text(*n).is_empty())
        {
            return Err(self.err(
                n,
                ErrorCode::Missing,
                format!("light {name} requires a value"),
            ));
        }
        Ok(value)
    }
    fn light_in(&self, node: Node<'_, '_>, context: LightContext) -> Result<Light, Diagnostic> {
        self.allowed(
            node,
            &[
                "pose",
                "light_on",
                "visualize",
                "cast_shadows",
                "intensity",
                "diffuse",
                "specular",
                "attenuation",
                "direction",
                "spot",
            ],
            &["name", "type"],
        )?;
        let kind = node.attribute("type");
        if context == LightContext::Document && kind.is_none() {
            return Err(self.err(node, ErrorCode::Missing, "light type is required"));
        }
        let light_type = match kind.unwrap_or("point") {
            "point" => LightType::Point,
            "directional" => LightType::Directional,
            "spot" => LightType::Spot,
            _ => return Err(self.err(node, ErrorCode::InvalidValue, "invalid light type")),
        };
        let mut light = Light {
            source: Some(self.source(node)),
            name: self.name(node)?,
            light_type,
            pose: self.pose(node)?,
            light_on: self.boolean(node, "light_on", true)?,
            visualize: self.boolean(node, "visualize", true)?,
            cast_shadows: self.boolean(node, "cast_shadows", false)?,
            diffuse: self.light_color(node, "diffuse", [1.; 4], context)?,
            specular: self.light_color(node, "specular", [0.1, 0.1, 0.1, 1.], context)?,
            intensity: self.scalar(node, "intensity", 1.)?,
            ..Light::default()
        };
        if let Some(attenuation) = child(node, "attenuation") {
            self.allowed(
                attenuation,
                &["range", "linear", "constant", "quadratic"],
                &[],
            )?;
            if context == LightContext::Standalone && child(attenuation, "range").is_none() {
                return Err(self.err(
                    attenuation,
                    ErrorCode::Missing,
                    "attenuation requires range",
                ));
            }
            light.attenuation_range = self
                .light_required_scalar(attenuation, "range", 10., context)?
                .max(0.);
            light.set_linear_attenuation_factor(self.scalar(attenuation, "linear", 1.)?);
            light.set_constant_attenuation_factor(self.scalar(attenuation, "constant", 1.)?);
            light.set_quadratic_attenuation_factor(self.scalar(attenuation, "quadratic", 0.)?);
        }
        if let Some(direction) = child(node, "direction") {
            let value = self.values::<3>(direction)?;
            if context == LightContext::Standalone || light_type != LightType::Point {
                light.direction = value;
            }
        } else if context == LightContext::Standalone && light_type != LightType::Point {
            return Err(self.err(node, ErrorCode::Missing, "light direction is required"));
        }
        if let Some(spot) = child(node, "spot") {
            self.allowed(spot, &["inner_angle", "outer_angle", "falloff"], &[])?;
            let inner = self.light_required_scalar(spot, "inner_angle", 0., context)?;
            let outer = self.light_required_scalar(spot, "outer_angle", 0., context)?;
            let falloff = self.light_required_scalar(spot, "falloff", 0., context)?;
            if context == LightContext::Standalone || light_type == LightType::Spot {
                light.set_spot_inner_angle(inner);
                light.set_spot_outer_angle(outer);
                light.set_spot_falloff(falloff);
            }
        }
        if context == LightContext::Document {
            // Native std::max(0, value) selects +0; the linear/constant clamps retain -0.
            for value in [
                &mut light.attenuation_range,
                &mut light.quadratic_attenuation_factor,
                &mut light.spot_inner_angle,
                &mut light.spot_outer_angle,
                &mut light.spot_falloff,
            ] {
                if *value == 0. {
                    *value = 0.;
                }
            }
        }
        Ok(light)
    }
    fn inertial(&self, link: Node<'_, '_>) -> Result<Inertial, Diagnostic> {
        let Some(node) = child(link, "inertial") else {
            return Ok(Inertial::default());
        };
        self.allowed(node, &["mass", "inertia", "pose", "fluid_added_mass"], &[])?;
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
        if let Some(fluid) = child(node, "fluid_added_mass") {
            let names = [
                "xx", "xy", "xz", "xp", "xq", "xr", "yy", "yz", "yp", "yq", "yr", "zz", "zp", "zq",
                "zr", "pp", "pq", "pr", "qq", "qr", "rr",
            ];
            self.allowed(fluid, &names, &[])?;
            let mut matrix = [[0.; 6]; 6];
            let upper = (0..6).flat_map(|row| (row..6).map(move |column| (row, column)));
            for ((row, column), name) in upper.zip(names) {
                let value = self.scalar(fluid, name, 0.)?;
                matrix[row][column] = value;
                matrix[column][row] = value;
            }
            result.fluid_added_mass = Some(Box::new(matrix));
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
        self.allowed(node, &["pose", "geometry", "surface", "density"], &["name"])?;
        let geometry =
            child(node, "geometry").map_or(Ok(Geometry::default()), |node| self.geometry(node))?;
        Ok(Collision {
            density: child(node, "density")
                .map(|_| self.scalar(node, "density", Collision::DENSITY_DEFAULT))
                .transpose()?,
            name: self.name(node)?,
            pose: self.pose(node)?,
            geometry,
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
                "empty",
                "box",
                "sphere",
                "cylinder",
                "capsule",
                "cone",
                "ellipsoid",
                "mesh",
                "plane",
                "heightmap",
                "polyline",
            ],
            &[],
        )?;
        let shapes = elements(geometry).collect::<Vec<_>>();
        if shapes.is_empty() {
            return Ok(Geometry::Empty);
        }
        let polylines = !shapes.is_empty() && shapes.iter().all(|n| n.has_tag_name("polyline"));
        if shapes.len() != 1 && !polylines {
            return Err(self.err(
                geometry,
                ErrorCode::InvalidValue,
                "geometry requires exactly one shape",
            ));
        }
        let shape = shapes[0];
        let geometry = match shape.tag_name().name() {
            "empty" => {
                self.allowed(shape, &[], &[])?;
                Geometry::Empty
            }
            "box" => {
                self.allowed(shape, &["size"], &[])?;
                let size = child(shape, "size").map_or(Ok([1.; 3]), |n| self.values(n))?;
                Geometry::Box {
                    size: if size.iter().all(|value| *value > 0.) {
                        size
                    } else {
                        [1.; 3]
                    },
                }
            }
            "sphere" => {
                self.allowed(shape, &["radius"], &[])?;
                let radius = self.required_scalar(shape, "radius", 1.)?;
                Geometry::Sphere {
                    radius: if radius > 0. { radius } else { 1. },
                }
            }
            "cylinder" => {
                self.allowed(shape, &["radius", "length"], &[])?;
                let radius = self.required_scalar(shape, "radius", 1.)?;
                let length = self.required_scalar(shape, "length", 1.)?;
                Geometry::Cylinder {
                    radius: if radius > 0. { radius } else { 0.5 },
                    length: if length > 0. { length } else { 1. },
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
            "heightmap" => Geometry::Heightmap(Box::new(self.heightmap(shape)?)),
            "polyline" => Geometry::Polyline(
                shapes
                    .into_iter()
                    .map(|n| self.polyline(n))
                    .collect::<Result<_, _>>()?,
            ),
            "mesh" => Geometry::Mesh(self.mesh(shape)?),
            _ => {
                self.allowed(shape, &["normal", "size"], &[])?;
                let normal = child(shape, "normal").map_or(Ok([0., 0., 1.]), |n| self.values(n))?;
                let vector = DVec3::from_array(normal);
                let length = vector.length();
                // Native Vector3::Normalize leaves vectors within 1e-6 of zero
                // unchanged; overflowed finite lengths divide to zero.
                let normal = if length > 1e-6 {
                    (vector / length).to_array()
                } else {
                    normal
                };
                let size = child(shape, "size").map_or(Ok([1.; 2]), |n| self.values(n))?;
                Geometry::Plane {
                    normal,
                    size: if size.iter().all(|value| *value > 0.) {
                        size
                    } else {
                        [1.; 2]
                    },
                }
            }
        };
        // Each primitive branch applies its native fallback to nonpositive
        // dimensions. Mesh scale is deliberately unconstrained by that policy.
        Ok(geometry)
    }
    fn heightmap(&self, node: Node<'_, '_>) -> Result<Heightmap, Diagnostic> {
        self.allowed(
            node,
            &[
                "uri",
                "size",
                "pos",
                "use_terrain_paging",
                "sampling",
                "texture",
                "blend",
            ],
            &[],
        )?;
        let mut textures = Vec::new();
        let mut blends = Vec::new();
        for item in elements(node) {
            if item.has_tag_name("texture") {
                self.allowed(item, &["size", "diffuse", "normal"], &[])?;
                textures.push(HeightmapTexture {
                    size: self.required_scalar(item, "size", 10.)?,
                    diffuse: self.schema_string(item, "diffuse", "__default__")?,
                    normal: self.schema_string(item, "normal", "__default__")?,
                });
            } else if item.has_tag_name("blend") {
                self.allowed(item, &["min_height", "fade_dist"], &[])?;
                blends.push(HeightmapBlend {
                    min_height: self.required_scalar(item, "min_height", 0.)?,
                    fade_distance: self.required_scalar(item, "fade_dist", 0.)?,
                });
            }
        }
        Ok(Heightmap {
            uri: self.schema_string(node, "uri", "__default__")?,
            size: child(node, "size").map_or(Ok([1.; 3]), |n| self.geometry_values(n, [1.; 3]))?,
            position: child(node, "pos")
                .map_or(Ok([0.; 3]), |n| self.geometry_values(n, [0.; 3]))?,
            use_terrain_paging: self.boolean(node, "use_terrain_paging", false)?,
            sampling: self.unsigned(node, "sampling", 1)?,
            textures,
            blends,
            source: self.source(node),
        })
    }
    fn polyline(&self, node: Node<'_, '_>) -> Result<Polyline, Diagnostic> {
        self.allowed(node, &["height", "point"], &[])?;
        let mut points = elements(node)
            .filter(|n| n.has_tag_name("point"))
            .map(|n| self.geometry_values(n, [0.; 2]))
            .collect::<Result<Vec<_>, _>>()?;
        // Native schema parsing inserts the required point when none is present.
        if points.is_empty() {
            points.push([0.; 2]);
        }
        Ok(Polyline {
            height: self.required_scalar(node, "height", 1.)?,
            points,
            source: self.source(node),
        })
    }
    fn schema_string(
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
    fn geometry_values<const N: usize>(
        &self,
        node: Node<'_, '_>,
        default: [f64; N],
    ) -> Result<[f64; N], Diagnostic> {
        self.allowed(node, &[], &[])?;
        if param_text(node).is_empty() {
            Ok(default)
        } else {
            self.values(node)
        }
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
    fn metadata_scalar(
        &self,
        node: Node<'_, '_>,
        name: &str,
        default: f64,
        required: bool,
    ) -> Result<f64, Diagnostic> {
        let Some(value) = child(node, name) else {
            return Ok(default);
        };
        self.allowed(value, &[], &[])?;
        let text = param_text(value);
        if text.is_empty() {
            return if required {
                Err(self.err(value, ErrorCode::Missing, format!("missing {name}")))
            } else {
                Ok(default)
            };
        }
        // These metadata fields retain native Param doubles, including NaN.
        let special = match text.to_ascii_lowercase().as_str() {
            "inf" | "infinity" | "+inf" | "+infinity" => Some(f64::INFINITY),
            "-inf" | "-infinity" => Some(f64::NEG_INFINITY),
            "nan" | "+nan" => Some(f64::NAN),
            "-nan" => Some(-f64::NAN),
            _ => None,
        };
        special.map_or_else(|| self.number(value, default), Ok)
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
        self.unsigned_text(value, &text)
    }
    fn unsigned_text(&self, node: Node<'_, '_>, text: &str) -> Result<u32, Diagnostic> {
        let lower = text.to_ascii_lowercase();
        let text = match lower.as_str() {
            "true" => "1",
            "false" => "0",
            _ => text,
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
            .map_err(|_| self.err(node, ErrorCode::InvalidValue, "expected unsigned integer"))?;
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
        self.allowed(
            node,
            &[
                "pose",
                "geometry",
                "material",
                "plugin",
                "cast_shadows",
                "transparency",
                "visibility_flags",
                "laser_retro",
            ],
            &["name"],
        )?;
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
            material: child(node, "material")
                .map(|n| self.material(n).map(Box::new))
                .transpose()?,
            properties: VisualProperties {
                cast_shadows: self.boolean(node, "cast_shadows", true)?,
                transparency: if let Some(value) = child(node, "transparency") {
                    crate::number::stored_double_to_float(self.scalar(node, "transparency", 0.)?)
                        .map_err(|code| self.err(value, code, "transparency conversion failed"))?
                } else {
                    0.
                },
                visibility_flags: self.unsigned(node, "visibility_flags", u32::MAX)?,
                laser_retro: child(node, "laser_retro")
                    .map(|_| self.scalar(node, "laser_retro", 0.))
                    .transpose()?,
            },
            plugins: self.plugins(node)?,
            source: self.source(node),
        })
    }
    fn material(&self, node: Node<'_, '_>) -> Result<Material, Diagnostic> {
        self.allowed(
            node,
            &[
                "ambient",
                "diffuse",
                "specular",
                "emissive",
                "shininess",
                "render_order",
                "lighting",
                "double_sided",
                "script",
                "shader",
                "pbr",
            ],
            &[],
        )?;
        let script = child(node, "script")
            .map(|script| {
                self.allowed_repeated(script, &["uri", "name"], &[], &["uri"])?;
                let mut uris = Vec::new();
                for uri in elements(script).filter(|n| n.has_tag_name("uri")) {
                    self.allowed(uri, &[], &[])?;
                    let mut text = param_text(uri).into_owned();
                    if text.is_empty() {
                        text = "__default__".into();
                    }
                    uris.push(text);
                }
                if uris.first().is_none_or(|uri| uri == "__default__") {
                    return Err(self.err(
                        script,
                        ErrorCode::Missing,
                        "material script requires uri",
                    ));
                }
                let name = self
                    .optional_text(script, "name")?
                    .filter(|name| !name.is_empty() && name != "__default__")
                    .ok_or_else(|| {
                        self.err(script, ErrorCode::Missing, "material script requires name")
                    })?;
                Ok(MaterialScript {
                    uris,
                    name,
                    source: self.source(script),
                })
            })
            .transpose()?;
        let (shader, normal_map) = if let Some(shader) = child(node, "shader") {
            self.allowed(shader, &["normal_map"], &["type"])?;
            let value = shader
                .attribute("type")
                .ok_or_else(|| self.err(shader, ErrorCode::Missing, "shader requires type"))?;
            let kind = match value.trim_matches([' ', '\t', '\n']) {
                "pixel" => ShaderType::Pixel,
                "vertex" => ShaderType::Vertex,
                "normal_map_objectspace" | "normal_map_object_space" => {
                    ShaderType::NormalMapObjectSpace
                }
                "normal_map_tangentspace" | "normal_map_tangent_space" => {
                    ShaderType::NormalMapTangentSpace
                }
                _ => {
                    return Err(self.err(
                        shader,
                        ErrorCode::InvalidValue,
                        "unsupported shader type",
                    ));
                }
            };
            let mut map = self
                .optional_text(shader, "normal_map")?
                .unwrap_or_default();
            if map == "__default__" {
                map.clear();
            }
            if matches!(
                kind,
                ShaderType::NormalMapObjectSpace | ShaderType::NormalMapTangentSpace
            ) && map.is_empty()
            {
                return Err(self.err(
                    shader,
                    ErrorCode::Missing,
                    "normal map shader requires normal_map",
                ));
            }
            (kind, map)
        } else {
            (ShaderType::Pixel, String::new())
        };
        Ok(Material {
            ambient: self.color(node, "ambient")?,
            diffuse: self.color(node, "diffuse")?,
            specular: self.color(node, "specular")?,
            emissive: self.color(node, "emissive")?,
            shininess: self.scalar(node, "shininess", 0.)?,
            render_order: self.float(node, "render_order", 0.)?,
            lighting: self.boolean(node, "lighting", true)?,
            double_sided: self.boolean(node, "double_sided", false)?,
            shader,
            normal_map,
            script,
            pbr: child(node, "pbr")
                .map(|node| self.pbr(node).map(Box::new))
                .transpose()?,
            source: self.source(node),
        })
    }
    fn pbr(&self, node: Node<'_, '_>) -> Result<PbrMaterial, Diagnostic> {
        self.allowed_repeated(node, &["metal", "specular"], &[], &["metal", "specular"])?;
        let mut pbr = PbrMaterial {
            metal: None,
            specular: None,
            source: self.source(node),
        };
        for workflow in elements(node) {
            let kind = if workflow.has_tag_name("metal") {
                PbrWorkflowKind::Metal
            } else {
                PbrWorkflowKind::Specular
            };
            let value = self.pbr_workflow(workflow, kind)?;
            match kind {
                PbrWorkflowKind::Metal => pbr.metal = Some(value),
                PbrWorkflowKind::Specular => pbr.specular = Some(value),
            }
        }
        Ok(pbr)
    }
    fn pbr_workflow(
        &self,
        node: Node<'_, '_>,
        kind: PbrWorkflowKind,
    ) -> Result<PbrWorkflow, Diagnostic> {
        let mut children = vec![
            "albedo_map",
            "normal_map",
            "environment_map",
            "ambient_occlusion_map",
            "emissive_map",
            "light_map",
        ];
        children.extend(match kind {
            PbrWorkflowKind::Metal => {
                ["roughness_map", "metalness_map", "roughness", "metalness"].as_slice()
            }
            PbrWorkflowKind::Specular => {
                ["specular_map", "glossiness_map", "glossiness"].as_slice()
            }
        });
        self.allowed(node, &children, &[])?;
        let (normal_map, normal_map_type) = if let Some(map) = child(node, "normal_map") {
            self.allowed(map, &[], &["type"])?;
            (
                param_text(map).into_owned(),
                map.attribute("type").map(str::to_owned),
            )
        } else {
            (String::new(), None)
        };
        let (light_map, light_map_uv_set) = if let Some(map) = child(node, "light_map") {
            self.allowed(map, &[], &["uv_set"])?;
            (
                param_text(map).into_owned(),
                map.attribute("uv_set")
                    .map(|text| {
                        let text = text.trim_matches([' ', '\t', '\n']);
                        if text.is_empty() {
                            Ok(0)
                        } else {
                            self.unsigned_text(map, text)
                        }
                    })
                    .transpose()?
                    .unwrap_or(0),
            )
        } else {
            (String::new(), 0)
        };
        let metal = kind == PbrWorkflowKind::Metal;
        Ok(PbrWorkflow {
            kind,
            albedo_map: self.optional_text(node, "albedo_map")?.unwrap_or_default(),
            normal_map,
            normal_map_type,
            environment_map: self
                .optional_text(node, "environment_map")?
                .unwrap_or_default(),
            ambient_occlusion_map: self
                .optional_text(node, "ambient_occlusion_map")?
                .unwrap_or_default(),
            emissive_map: self
                .optional_text(node, "emissive_map")?
                .unwrap_or_default(),
            light_map,
            light_map_uv_set,
            roughness_map: if metal {
                self.optional_text(node, "roughness_map")?
                    .unwrap_or_default()
            } else {
                String::new()
            },
            metalness_map: if metal {
                self.optional_text(node, "metalness_map")?
                    .unwrap_or_default()
            } else {
                String::new()
            },
            specular_map: if !metal {
                self.optional_text(node, "specular_map")?
                    .unwrap_or_default()
            } else {
                String::new()
            },
            glossiness_map: if !metal {
                self.optional_text(node, "glossiness_map")?
                    .unwrap_or_default()
            } else {
                String::new()
            },
            roughness: if metal {
                self.pbr_scalar(node, "roughness", 0.5)?
            } else {
                0.5
            },
            metalness: if metal {
                self.pbr_scalar(node, "metalness", 0.5)?
            } else {
                0.5
            },
            glossiness: if !metal {
                self.pbr_scalar(node, "glossiness", 0.)?
            } else {
                0.
            },
            source: self.source(node),
        })
    }
    fn pbr_scalar(&self, node: Node<'_, '_>, name: &str, default: f64) -> Result<f64, Diagnostic> {
        // Native workflow getters default zero when a Param conversion fails.
        match self.metadata_scalar(node, name, default, false) {
            Err(error) if error.code == ErrorCode::InvalidValue => Ok(0.),
            result => result,
        }
    }
    fn float(&self, node: Node<'_, '_>, name: &str, default: f32) -> Result<f32, Diagnostic> {
        let Some(value) = child(node, name) else {
            return Ok(default);
        };
        self.allowed(value, &[], &[])?;
        let text = param_text(value);
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
        crate::number::float(text).map_err(|code| self.err(value, code, "float conversion failed"))
    }
    fn color(&self, node: Node<'_, '_>, name: &str) -> Result<[f32; 4], Diagnostic> {
        let Some(value) = child(node, name) else {
            return Ok([0., 0., 0., 1.]);
        };
        self.allowed(value, &[], &[])?;
        let text = param_text(value);
        if text.is_empty() {
            return Ok([0., 0., 0., 1.]);
        }
        let mut result = [0., 0., 0., 1.];
        let tokens: Vec<_> = text
            .split(legacy_whitespace)
            .filter(|s| !s.is_empty())
            .collect();
        if !matches!(tokens.len(), 3 | 4) {
            return Err(self.err(
                value,
                ErrorCode::InvalidValue,
                "color requires three or four values",
            ));
        }
        for (slot, token) in result.iter_mut().zip(tokens) {
            let number = crate::number::float(token)
                .map_err(|code| self.err(value, code, "color conversion failed"))?;
            if !number.is_nan() && !(0.0..=1.0).contains(&number) {
                return Err(self.err(
                    value,
                    ErrorCode::InvalidValue,
                    "color component must be between zero and one",
                ));
            }
            // Native Color construction clamps NaN to zero after the range check.
            *slot = if number.is_nan() { 0. } else { number };
        }
        Ok(result)
    }
    fn joint(
        &self,
        node: Node<'_, '_>,
        model_frame_default: bool,
        allow_mimic: bool,
    ) -> Result<Joint, Diagnostic> {
        self.allowed(
            node,
            &[
                "parent",
                "child",
                "pose",
                "axis",
                "axis2",
                "sensor",
                "physics",
                "thread_pitch",
                "screw_thread_pitch",
            ],
            &["name", "type"],
        )?;
        let kind = match self.attr(node, "type")?.as_str() {
            "fixed" => JointKind::Fixed,
            "ball" => JointKind::Ball,
            "gearbox" => JointKind::Gearbox,
            "screw" => JointKind::Screw,
            "revolute" => JointKind::Revolute,
            "prismatic" => JointKind::Prismatic,
            "continuous" => JointKind::Continuous,
            "universal" => JointKind::Universal,
            "revolute2" => JointKind::Revolute2,
            _ => return Err(self.err(node, ErrorCode::Unsupported, "unsupported joint type")),
        };
        let axis = child(node, "axis")
            .map(|n| self.axis(n, model_frame_default, allow_mimic))
            .transpose()?;
        let axis2 = child(node, "axis2")
            .map(|n| self.axis(n, model_frame_default, allow_mimic))
            .transpose()?;
        let screw_thread_pitch = if child(node, "screw_thread_pitch").is_some() {
            self.scalar(node, "screw_thread_pitch", 1.)?
        } else if child(node, "thread_pitch").is_some() {
            -2. * std::f64::consts::PI / self.scalar(node, "thread_pitch", 1.)?
        } else {
            1.
        };
        let physics_xml = child(node, "physics").map(|n| self.xml[n.range()].into());
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
            screw_thread_pitch,
            physics_xml,
            sensors: {
                let sensors = elements(node)
                    .filter(|value| value.has_tag_name("sensor"))
                    .map(|value| self.sensor(value))
                    .collect::<Result<Vec<_>, _>>()?;
                if sensors
                    .iter()
                    .map(|sensor| &sensor.name)
                    .collect::<BTreeSet<_>>()
                    .len()
                    != sensors.len()
                {
                    return Err(self.err(
                        node,
                        ErrorCode::DuplicateName,
                        "duplicate joint sensor name",
                    ));
                }
                sensors
            },
            source: self.source(node),
        })
    }
    fn axis(
        &self,
        axis: Node<'_, '_>,
        model_frame_default: bool,
        allow_mimic: bool,
    ) -> Result<Axis, Diagnostic> {
        self.allowed(
            axis,
            if is_pre17(axis) {
                &[
                    "xyz",
                    "limit",
                    "dynamics",
                    "use_parent_model_frame",
                    // Native retains this legacy unknown child with a warning;
                    // JointAxis DOM does not expose its value.
                    "initial_position",
                ]
            } else if allow_mimic {
                &["xyz", "limit", "dynamics", "mimic"]
            } else {
                &["xyz", "limit", "dynamics"]
            },
            &[],
        )?;
        // Upstream's conversion table recognizes exactly these four spellings;
        // every other value is removed without being interpreted as a boolean.
        let parent_model = is_pre17(axis)
            && child(axis, "use_parent_model_frame").map_or(model_frame_default, |n| {
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
        let xyz = normalized_axis_direction(xyz)
            .map_err(|_| self.err(axis, ErrorCode::InvalidValue, "invalid joint axis"))?;
        let mut result = Axis {
            xyz,
            expressed_in,
            // Native Root loads the converted current schema's unbounded defaults.
            lower: f64::NEG_INFINITY,
            upper: f64::INFINITY,
            effort: f64::INFINITY,
            velocity: f64::INFINITY,
            dynamics: if let Some(node) = child(axis, "dynamics") {
                self.allowed(
                    node,
                    &[
                        "damping",
                        "friction",
                        "spring_reference",
                        "spring_stiffness",
                    ],
                    &[],
                )?;
                AxisDynamics {
                    damping: self.scalar(node, "damping", 0.)?,
                    friction: self.scalar(node, "friction", 0.)?,
                    spring_reference: self.required_scalar(node, "spring_reference", 0.)?,
                    spring_stiffness: self.required_scalar(node, "spring_stiffness", 0.)?,
                    source: Some(self.source(node)),
                    xml: Some(self.xml[node.range()].into()),
                }
            } else {
                AxisDynamics::default()
            },
            stiffness: 1e8,
            dissipation: 1.,
            mimic: child(axis, "mimic")
                .map(|node| self.axis_mimic(node))
                .transpose()?,
        };
        if let Some(limit) = child(axis, "limit") {
            self.allowed(
                limit,
                &[
                    "lower",
                    "upper",
                    "effort",
                    "velocity",
                    "stiffness",
                    "dissipation",
                ],
                &[],
            )?;
            result.lower = self.metadata_scalar(limit, "lower", result.lower, true)?;
            result.upper = self.metadata_scalar(limit, "upper", result.upper, true)?;
            result.effort = self.metadata_scalar(limit, "effort", result.effort, false)?;
            result.velocity = self.metadata_scalar(limit, "velocity", result.velocity, false)?;
            result.stiffness = self.scalar(limit, "stiffness", result.stiffness)?;
            result.dissipation = self.scalar(limit, "dissipation", result.dissipation)?;
            if result.effort < 0. {
                result.effort = f64::INFINITY;
            }
            if result.velocity < 0. {
                result.velocity = f64::INFINITY;
            }
        }
        Ok(result)
    }
    fn axis_mimic(&self, node: Node<'_, '_>) -> Result<AxisMimic, Diagnostic> {
        // Native loading retains known values while unknown mimic content is
        // advisory; its warning side effect is outside this owned DOM subset.
        Ok(AxisMimic {
            joint: self.attr(node, "joint")?,
            axis: node.attribute("axis").unwrap_or("axis").into(),
            multiplier: self.scalar(node, "multiplier", 1.)?,
            offset: self.scalar(node, "offset", 0.)?,
            reference: self.scalar(node, "reference", 0.)?,
        })
    }
    fn validate_axis_mimics(&self, model: &str, joints: &[Joint]) -> Result<(), Diagnostic> {
        for joint in joints {
            for (follower_name, follower) in [("axis", joint.axis(0)), ("axis2", joint.axis(1))] {
                let Some(mimic) = follower.and_then(|axis| axis.mimic.as_ref()) else {
                    continue;
                };
                let invalid =
                    |message| error(ErrorCode::JointAxisMimicInvalid, &joint.source, message);
                if mimic.joint == joint.name && mimic.axis == follower_name {
                    return Err(invalid(format!(
                        "Axis with name [{follower_name}] in joint with name [{}] cannot mimic itself.",
                        joint.name
                    )));
                }
                if !matches!(mimic.axis.as_str(), "axis" | "axis2") {
                    return Err(invalid(format!(
                        "Axis with name [{follower_name}] in joint with name [{}] specified an invalid leader axis name [{}].",
                        joint.name, mimic.axis
                    )));
                }
                let Some(leader) = joints
                    .iter()
                    .find(|candidate| candidate.name == mimic.joint)
                else {
                    return Err(invalid(format!(
                        "A joint with name[{}] specified by an axis mimic in joint with name[{}] not found in model with name[{model}].",
                        mimic.joint, joint.name
                    )));
                };
                let leader_index = usize::from(mimic.axis == "axis2");
                if leader.axis(leader_index).is_none() {
                    return Err(invalid(format!(
                        "Axis with name [{follower_name}] in joint with name [{}] specified a leader axis name [{}] that is not found in the leader joint with name [{}].",
                        joint.name, mimic.axis, mimic.joint
                    )));
                }
            }
        }
        Ok(())
    }
    fn imu(&self, sensor: Node<'_, '_>) -> Result<Imu, Diagnostic> {
        let Some(node) = child(sensor, "imu") else {
            return Ok(Imu::default());
        };
        self.imu_node(node)
    }
    fn imu_node(&self, node: Node<'_, '_>) -> Result<Imu, Diagnostic> {
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
            source: Some(self.source(node)),
            enable_orientation: self.boolean(node, "enable_orientation", true)?,
            ..Imu::default()
        };
        imu.angular_velocity = self.noise_vector(node, "angular_velocity")?;
        imu.linear_acceleration = self.noise_vector(node, "linear_acceleration")?;
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
    fn noise_vector(&self, node: Node<'_, '_>, name: &str) -> Result<[Noise; 3], Diagnostic> {
        match child(node, name) {
            Some(vector) => self.noise_vector_node(vector),
            None => Ok(std::array::from_fn(|_| Noise::default())),
        }
    }
    fn noise_vector_node(&self, vector: Node<'_, '_>) -> Result<[Noise; 3], Diagnostic> {
        let mut noises = std::array::from_fn(|_| Noise::default());
        self.allowed(vector, &["x", "y", "z"], &[])?;
        for (index, component) in ["x", "y", "z"].iter().enumerate() {
            if let Some(axis) = child(vector, component) {
                self.allowed(axis, &["noise"], &[])?;
                if let Some(noise) = child(axis, "noise") {
                    noises[index] = self.noise(noise)?;
                }
            }
        }
        Ok(noises)
    }
    fn force_torque(&self, sensor: Node<'_, '_>) -> Result<ForceTorque, Diagnostic> {
        let Some(node) = child(sensor, "force_torque") else {
            return Ok(ForceTorque::default());
        };
        self.force_torque_node(node)
    }
    fn force_torque_node(&self, node: Node<'_, '_>) -> Result<ForceTorque, Diagnostic> {
        self.allowed(
            node,
            &["frame", "measure_direction", "force", "torque"],
            &[],
        )?;
        let frame = self
            .optional_text(node, "frame")?
            .filter(|value| !value.is_empty());
        let frame = match frame.as_deref().unwrap_or("child") {
            "parent" => ForceTorqueFrame::Parent,
            "child" => ForceTorqueFrame::Child,
            "sensor" => ForceTorqueFrame::Sensor,
            _ => {
                return Err(self.err(
                    child(node, "frame").unwrap(),
                    ErrorCode::InvalidValue,
                    "invalid force/torque frame",
                ));
            }
        };
        let direction = self
            .optional_text(node, "measure_direction")?
            .filter(|value| !value.is_empty());
        let measure_direction = match direction.as_deref().unwrap_or("child_to_parent") {
            "parent_to_child" => ForceTorqueMeasureDirection::ParentToChild,
            "child_to_parent" => ForceTorqueMeasureDirection::ChildToParent,
            _ => {
                return Err(self.err(
                    child(node, "measure_direction").unwrap(),
                    ErrorCode::InvalidValue,
                    "invalid force/torque measure direction",
                ));
            }
        };
        Ok(ForceTorque {
            frame,
            measure_direction,
            force: self.noise_vector(node, "force")?,
            torque: self.noise_vector(node, "torque")?,
            source: Some(self.source(node)),
        })
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
            .map(|value| value.trim_matches([' ', '\t', '\n']))
            .filter(|value| !value.is_empty())
            .ok_or_else(|| self.err(node, ErrorCode::Missing, "missing noise type attribute"))?
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
    fn contact_string(&self, value: Node<'_, '_>) -> Result<String, Diagnostic> {
        self.allowed(value, &[], &[])?;
        let text = param_text(value);
        if text.is_empty() {
            return Err(self.err(value, ErrorCode::Missing, "empty required contact value"));
        }
        Ok(text.into_owned())
    }
    fn contact(&self, sensor: Node<'_, '_>) -> Result<ContactSensor, Diagnostic> {
        let Some(node) = child(sensor, "contact") else {
            return Ok(ContactSensor::default());
        };
        self.contact_node(node)
    }
    fn contact_node(&self, node: Node<'_, '_>) -> Result<ContactSensor, Diagnostic> {
        self.allowed(node, &["collision", "topic"], &[])?;
        let mut collisions = elements(node)
            .filter(|value| value.has_tag_name("collision"))
            .map(|value| self.contact_string(value))
            .collect::<Result<Vec<_>, _>>()?;
        if collisions.is_empty() {
            collisions.push("__default__".into());
        }
        let topic = child(node, "topic").map_or_else(
            || Ok("__default_topic__".into()),
            |value| self.contact_string(value),
        )?;
        Ok(ContactSensor {
            collisions,
            topic,
            source: Some(self.source(node)),
        })
    }
    fn altimeter_noise(&self, altimeter: Node<'_, '_>, name: &str) -> Result<Noise, Diagnostic> {
        let Some(component) = child(altimeter, name) else {
            return Ok(Noise::default());
        };
        self.allowed(component, &["noise"], &[])?;
        child(component, "noise")
            .map(|noise| self.noise(noise))
            .transpose()
            .map(|noise| noise.unwrap_or_default())
    }
    fn altimeter(&self, sensor: Node<'_, '_>) -> Result<Altimeter, Diagnostic> {
        let Some(node) = child(sensor, "altimeter") else {
            return Ok(Altimeter::default());
        };
        self.altimeter_node(node)
    }
    fn altimeter_node(&self, node: Node<'_, '_>) -> Result<Altimeter, Diagnostic> {
        self.allowed(node, &["vertical_position", "vertical_velocity"], &[])?;
        Ok(Altimeter {
            vertical_position: self.altimeter_noise(node, "vertical_position")?,
            vertical_velocity: self.altimeter_noise(node, "vertical_velocity")?,
            source: Some(self.source(node)),
        })
    }
    fn air_pressure(&self, sensor: Node<'_, '_>) -> Result<AirPressure, Diagnostic> {
        let Some(node) = child(sensor, "air_pressure") else {
            return Ok(AirPressure::default());
        };
        self.air_pressure_node(node)
    }
    fn air_pressure_node(&self, node: Node<'_, '_>) -> Result<AirPressure, Diagnostic> {
        self.allowed(node, &["pressure", "reference_altitude"], &[])?;
        let pressure = child(node, "pressure")
            .map(|component| {
                self.allowed(component, &["noise"], &[])?;
                child(component, "noise")
                    .map(|noise| self.noise(noise))
                    .transpose()
                    .map(|noise| noise.unwrap_or_default())
            })
            .transpose()?
            .unwrap_or_default();
        Ok(AirPressure {
            pressure,
            reference_altitude: self.scalar(node, "reference_altitude", 0.)?,
            source: Some(self.source(node)),
        })
    }
    fn air_speed(&self, sensor: Node<'_, '_>) -> Result<AirSpeed, Diagnostic> {
        let Some(node) = child(sensor, "air_speed") else {
            return Ok(AirSpeed::default());
        };
        self.air_speed_node(node)
    }
    fn air_speed_node(&self, node: Node<'_, '_>) -> Result<AirSpeed, Diagnostic> {
        self.allowed(node, &["pressure"], &[])?;
        let pressure = child(node, "pressure")
            .map(|component| {
                self.allowed(component, &["noise"], &[])?;
                child(component, "noise")
                    .map(|noise| self.noise(noise))
                    .transpose()
                    .map(|noise| noise.unwrap_or_default())
            })
            .transpose()?
            .unwrap_or_default();
        Ok(AirSpeed {
            pressure,
            source: Some(self.source(node)),
        })
    }
    fn navsat_noise(
        &self,
        navsat: Node<'_, '_>,
        sensing: &str,
        axis: &str,
    ) -> Result<Noise, Diagnostic> {
        let Some(group) = child(navsat, sensing) else {
            return Ok(Noise::default());
        };
        self.allowed(group, &["horizontal", "vertical"], &[])?;
        let Some(component) = child(group, axis) else {
            return Ok(Noise::default());
        };
        self.allowed(component, &["noise"], &[])?;
        child(component, "noise")
            .map(|noise| self.noise(noise))
            .transpose()
            .map(|noise| noise.unwrap_or_default())
    }
    fn navsat(&self, sensor: Node<'_, '_>, tag: &str) -> Result<NavSat, Diagnostic> {
        let Some(node) = child(sensor, tag) else {
            return Ok(NavSat::default());
        };
        self.navsat_node(node)
    }
    fn navsat_node(&self, node: Node<'_, '_>) -> Result<NavSat, Diagnostic> {
        self.allowed(node, &["position_sensing", "velocity_sensing"], &[])?;
        Ok(NavSat {
            horizontal_position: self.navsat_noise(node, "position_sensing", "horizontal")?,
            vertical_position: self.navsat_noise(node, "position_sensing", "vertical")?,
            horizontal_velocity: self.navsat_noise(node, "velocity_sensing", "horizontal")?,
            vertical_velocity: self.navsat_noise(node, "velocity_sensing", "vertical")?,
            source: Some(self.source(node)),
        })
    }
    fn lidar_scan(&self, node: Node<'_, '_>, default: LidarScan) -> Result<LidarScan, Diagnostic> {
        self.allowed(
            node,
            &["samples", "resolution", "min_angle", "max_angle"],
            &[],
        )?;
        Ok(LidarScan {
            samples: self.unsigned(node, "samples", default.samples)?,
            resolution: self.scalar(node, "resolution", default.resolution)?,
            min_angle: self.scalar(node, "min_angle", default.min_angle)?,
            max_angle: self.scalar(node, "max_angle", default.max_angle)?,
        })
    }
    fn lidar_noise(&self, node: Node<'_, '_>) -> Result<Noise, Diagnostic> {
        self.allowed(node, &["type", "mean", "stddev"], &[])?;
        let kind = match self
            .optional_text(node, "type")?
            .unwrap_or_else(|| "gaussian".into())
            .trim_matches([' ', '\t', '\n'])
            .to_ascii_lowercase()
            .as_str()
        {
            "none" => NoiseKind::None,
            "gaussian" => NoiseKind::Gaussian,
            "gaussian_quantized" => NoiseKind::GaussianQuantized,
            _ => return Err(self.err(node, ErrorCode::InvalidValue, "invalid lidar noise type")),
        };
        let noise = Noise {
            kind,
            mean: self.scalar(node, "mean", 0.)?,
            stddev: self.scalar(node, "stddev", 0.)?,
            ..Noise::default()
        };
        if noise.stddev < 0. {
            return Err(self.err(node, ErrorCode::InvalidValue, "negative lidar noise stddev"));
        }
        Ok(noise)
    }
    fn lidar(&self, sensor: Node<'_, '_>, tag: &str) -> Result<Lidar, Diagnostic> {
        let Some(node) = child(sensor, tag) else {
            return Ok(Lidar::default());
        };
        self.lidar_node(node)
    }
    fn lidar_node(&self, node: Node<'_, '_>) -> Result<Lidar, Diagnostic> {
        self.allowed(node, &["scan", "range", "noise", "visibility_mask"], &[])?;
        let scan = child(node, "scan")
            .ok_or_else(|| self.err(node, ErrorCode::Missing, "missing lidar scan"))?;
        self.allowed(scan, &["horizontal", "vertical"], &[])?;
        let horizontal = child(scan, "horizontal")
            .ok_or_else(|| self.err(scan, ErrorCode::Missing, "missing horizontal lidar scan"))?;
        let range = child(node, "range")
            .ok_or_else(|| self.err(node, ErrorCode::Missing, "missing lidar range"))?;
        self.allowed(range, &["min", "max", "resolution"], &[])?;
        let default = Lidar::default();
        Ok(Lidar {
            horizontal: self.lidar_scan(horizontal, default.horizontal)?,
            vertical: child(scan, "vertical")
                .map(|vertical| self.lidar_scan(vertical, default.vertical))
                .transpose()?
                .unwrap_or(default.vertical),
            range_min: self.scalar(range, "min", default.range_min)?,
            range_max: self.scalar(range, "max", default.range_max)?,
            range_resolution: self.scalar(range, "resolution", default.range_resolution)?,
            noise: child(node, "noise")
                .map(|noise| self.lidar_noise(noise))
                .transpose()?
                .unwrap_or_default(),
            visibility_mask: self.unsigned(node, "visibility_mask", default.visibility_mask)?,
            source: Some(self.source(node)),
        })
    }
    fn camera_noise(&self, node: Node<'_, '_>) -> Result<Noise, Diagnostic> {
        self.allowed(node, &["type", "mean", "stddev"], &[])?;
        let kind = match self
            .optional_text(node, "type")?
            .unwrap_or_else(|| "gaussian".into())
            .as_str()
        {
            "none" => NoiseKind::None,
            "gaussian" => NoiseKind::Gaussian,
            "gaussian_quantized" => NoiseKind::GaussianQuantized,
            _ => return Err(self.err(node, ErrorCode::InvalidValue, "invalid camera noise type")),
        };
        Ok(Noise {
            kind,
            mean: self.scalar(node, "mean", 0.)?,
            stddev: self.scalar(node, "stddev", 0.)?,
            ..Noise::default()
        })
    }
    fn camera(&self, sensor: Node<'_, '_>) -> Result<Camera, Diagnostic> {
        let Some(node) = child(sensor, "camera") else {
            return Ok(Camera::default());
        };
        self.camera_node(node)
    }
    fn camera_node(&self, node: Node<'_, '_>) -> Result<Camera, Diagnostic> {
        self.allowed(
            node,
            &[
                "pose",
                "camera_info_topic",
                "horizontal_fov",
                "triggered",
                "trigger_topic",
                "image",
                "clip",
                "save",
                "noise",
                "distortion",
                "lens",
                "visibility_mask",
                "segmentation_type",
                "box_type",
            ],
            &["name"],
        )?;
        let mut camera = Camera {
            name: node
                .attribute("name")
                .map(|name| name.trim_matches([' ', '\t', '\n']))
                .filter(|name| !name.is_empty())
                .unwrap_or("__default__")
                .into(),
            pose: self.pose(node)?,
            source: Some(self.source(node)),
            ..Camera::default()
        };
        camera.camera_info_topic = self
            .optional_text(node, "camera_info_topic")?
            .unwrap_or_default();
        camera.horizontal_fov = self.scalar(node, "horizontal_fov", camera.horizontal_fov)?;
        camera.triggered = self.boolean(node, "triggered", camera.triggered)?;
        camera.trigger_topic = self
            .optional_text(node, "trigger_topic")?
            .unwrap_or_default();
        if let Some(value) = self.optional_text(node, "segmentation_type")? {
            camera.segmentation_type = value;
            camera.has_segmentation_type = true;
        }
        if let Some(value) = self.optional_text(node, "box_type")? {
            camera.bounding_box_type = value;
            camera.has_bounding_box_type = true;
        }
        let image = child(node, "image")
            .ok_or_else(|| self.err(node, ErrorCode::Missing, "camera image is missing"))?;
        self.allowed(image, &["width", "height", "format", "anti_aliasing"], &[])?;
        camera.image_width = self.unsigned(image, "width", camera.image_width)?;
        camera.image_height = self.unsigned(image, "height", camera.image_height)?;
        camera.anti_aliasing = self.unsigned(image, "anti_aliasing", camera.anti_aliasing)?;
        if let Some(format) = self.optional_text(image, "format")? {
            camera.pixel_format = match format.as_str() {
                "R8G8B8" | "RGB_INT8" => CameraPixelFormat::RgbInt8,
                "L8" | "L_INT8" => CameraPixelFormat::LInt8,
                _ => {
                    return Err(self.err(
                        image,
                        ErrorCode::InvalidValue,
                        "invalid camera pixel format",
                    ));
                }
            };
        }
        let clip = child(node, "clip")
            .ok_or_else(|| self.err(node, ErrorCode::Missing, "camera clip is missing"))?;
        self.allowed(clip, &["near", "far"], &[])?;
        camera.near_clip = self.scalar(clip, "near", camera.near_clip)?;
        camera.far_clip = self.scalar(clip, "far", camera.far_clip)?;
        if let Some(save) = child(node, "save") {
            self.allowed(save, &["path"], &["enabled"])?;
            camera.save_frames = match save.attribute("enabled").unwrap_or("false") {
                "true" | "1" => true,
                "false" | "0" => false,
                _ => {
                    return Err(self.err(
                        save,
                        ErrorCode::InvalidValue,
                        "invalid camera save enabled",
                    ));
                }
            };
            let path = self.optional_text(save, "path")?.unwrap_or_default();
            camera.save_frames_path = if camera.save_frames {
                path
            } else {
                String::new()
            };
        }
        if let Some(noise) = child(node, "noise") {
            camera.image_noise = self.camera_noise(noise)?;
        }
        if let Some(distortion) = child(node, "distortion") {
            self.allowed(distortion, &["k1", "k2", "k3", "p1", "p2", "center"], &[])?;
            camera.distortion.k1 = self.scalar(distortion, "k1", camera.distortion.k1)?;
            camera.distortion.k2 = self.scalar(distortion, "k2", camera.distortion.k2)?;
            camera.distortion.k3 = self.scalar(distortion, "k3", camera.distortion.k3)?;
            camera.distortion.p1 = self.scalar(distortion, "p1", camera.distortion.p1)?;
            camera.distortion.p2 = self.scalar(distortion, "p2", camera.distortion.p2)?;
            if let Some(center) = child(distortion, "center") {
                camera.distortion.center = self.values::<2>(center)?;
            }
        }
        camera.lens.has_intrinsics =
            child(node, "lens").is_some_and(|lens| child(lens, "intrinsics").is_some());
        camera.lens.has_projection =
            child(node, "lens").is_some_and(|lens| child(lens, "projection").is_some());
        if let Some(lens) = child(node, "lens") {
            self.allowed(
                lens,
                &[
                    "type",
                    "scale_to_hfov",
                    "custom_function",
                    "cutoff_angle",
                    "env_texture_size",
                    "intrinsics",
                    "projection",
                ],
                &[],
            )?;
            camera.lens.kind = self
                .optional_text(lens, "type")?
                .unwrap_or(camera.lens.kind);
            camera.lens.scale_to_hfov =
                self.boolean(lens, "scale_to_hfov", camera.lens.scale_to_hfov)?;
            if let Some(custom) = child(lens, "custom_function") {
                self.allowed(custom, &["c1", "c2", "c3", "f", "fun"], &[])?;
                camera.lens.c1 = self.scalar(custom, "c1", camera.lens.c1)?;
                camera.lens.c2 = self.scalar(custom, "c2", camera.lens.c2)?;
                camera.lens.c3 = self.scalar(custom, "c3", camera.lens.c3)?;
                camera.lens.focal_length = self.scalar(custom, "f", camera.lens.focal_length)?;
                camera.lens.function = self
                    .optional_text(custom, "fun")?
                    .unwrap_or(camera.lens.function);
            }
            // A present lens uses its schema cutoff; no lens keeps constructor PI/2.
            camera.lens.cutoff_angle =
                self.scalar(lens, "cutoff_angle", crate::camera::SCHEMA_LENS_CUTOFF)?;
            camera.lens.environment_texture_size = self.unsigned(
                lens,
                "env_texture_size",
                camera.lens.environment_texture_size,
            )?;
            if let Some(intrinsics) = child(lens, "intrinsics") {
                self.allowed(intrinsics, &["fx", "fy", "cx", "cy", "s"], &[])?;
                camera.lens.intrinsics.fx =
                    self.scalar(intrinsics, "fx", camera.lens.intrinsics.fx)?;
                camera.lens.intrinsics.fy =
                    self.scalar(intrinsics, "fy", camera.lens.intrinsics.fy)?;
                camera.lens.intrinsics.cx =
                    self.scalar(intrinsics, "cx", camera.lens.intrinsics.cx)?;
                camera.lens.intrinsics.cy =
                    self.scalar(intrinsics, "cy", camera.lens.intrinsics.cy)?;
                camera.lens.intrinsics.skew =
                    self.scalar(intrinsics, "s", camera.lens.intrinsics.skew)?;
            }
            if let Some(projection) = child(lens, "projection") {
                self.allowed(
                    projection,
                    &["p_fx", "p_fy", "p_cx", "p_cy", "tx", "ty"],
                    &[],
                )?;
                camera.lens.projection.fx =
                    self.scalar(projection, "p_fx", camera.lens.projection.fx)?;
                camera.lens.projection.fy =
                    self.scalar(projection, "p_fy", camera.lens.projection.fy)?;
                camera.lens.projection.cx =
                    self.scalar(projection, "p_cx", camera.lens.projection.cx)?;
                camera.lens.projection.cy =
                    self.scalar(projection, "p_cy", camera.lens.projection.cy)?;
                camera.lens.projection.tx =
                    self.scalar(projection, "tx", camera.lens.projection.tx)?;
                camera.lens.projection.ty =
                    self.scalar(projection, "ty", camera.lens.projection.ty)?;
            }
        }
        camera.visibility_mask = self.unsigned(node, "visibility_mask", camera.visibility_mask)?;
        Ok(camera)
    }
    fn magnetometer(&self, sensor: Node<'_, '_>) -> Result<Magnetometer, Diagnostic> {
        let Some(node) = child(sensor, "magnetometer") else {
            return Ok(Magnetometer::default());
        };
        self.magnetometer_node(node)
    }
    fn magnetometer_node(&self, node: Node<'_, '_>) -> Result<Magnetometer, Diagnostic> {
        Ok(Magnetometer {
            noise: self.noise_vector_node(node)?,
            source: Some(self.source(node)),
        })
    }
    fn sensor(&self, node: Node<'_, '_>) -> Result<Sensor, Diagnostic> {
        let kind = self
            .attr(node, "type")?
            .trim_matches([' ', '\t', '\n'])
            .to_owned();
        let custom_type = (kind == "custom")
            .then(|| {
                node.attribute("gz:type")
                    .map(|value| value.trim_matches([' ', '\t', '\n']).to_owned())
            })
            .flatten();
        let custom_element = if kind == "custom" {
            elements(node).find(|child| child.tag_name().namespace().is_some())
        } else {
            None
        };
        let mut allowed_children = vec![
            "pose",
            "update_rate",
            "always_on",
            "alwaysOn",
            "visualize",
            "enable_metrics",
            "topic",
            "frame_id",
            "gz_frame_id",
            "plugin",
            "imu",
            "contact",
            "force_torque",
            "magnetometer",
            "altimeter",
            "air_pressure",
            "navsat",
            "gps",
            "air_speed",
            "ray",
            "lidar",
            "camera",
        ];
        // Unknown direct children of custom sensors are native extension data.
        // Keep each original subtree instead of assigning schema or runtime meaning.
        let custom_fields: BTreeSet<_> = elements(node)
            .filter(|child| {
                kind == "custom" && !allowed_children.contains(&child.tag_name().name())
            })
            .map(|child| child.tag_name().name())
            .collect();
        if let Some(extension) = custom_element {
            allowed_children.push(extension.tag_name().name());
        }
        // The unknown legacy spelling is copied as opaque XML, never a boolean alias.
        let attrs: &[&str] = if kind == "custom" {
            &["name", "type", "gz:type"]
        } else {
            &["name", "type"]
        };
        let mut repeated = vec!["alwaysOn"];
        repeated.extend(custom_fields.iter().copied());
        self.allowed_repeated(node, &allowed_children, attrs, &repeated)?;
        if !matches!(
            kind.as_str(),
            "imu"
                | "contact"
                | "force_torque"
                | "magnetometer"
                | "altimeter"
                | "air_pressure"
                | "navsat"
                | "gps"
                | "air_speed"
                | "ray"
                | "lidar"
                | "gpu_ray"
                | "gpu_lidar"
                | "camera"
                | "depth"
                | "depth_camera"
                | "rgbd"
                | "rgbd_camera"
                | "segmentation"
                | "segmentation_camera"
                | "boundingbox"
                | "boundingbox_camera"
                | "thermal"
                | "thermal_camera"
                | "wideanglecamera"
                | "wide_angle_camera"
                | "logical_camera"
                | "multicamera"
                | "rfid"
                | "rfidtag"
                | "sonar"
                | "wireless_receiver"
                | "wireless_transmitter"
                | "custom"
        ) {
            return Err(self.err(
                node,
                ErrorCode::Unsupported,
                "only typed sensor configurations and native-known envelope-only sensor types are supported",
            ));
        }
        // Native Sensor::Load selects <navsat> before legacy <gps> for either type.
        let config_tag = if matches!(kind.as_str(), "navsat" | "gps") {
            if child(node, "navsat").is_some() {
                "navsat"
            } else {
                "gps"
            }
        } else if matches!(kind.as_str(), "ray" | "lidar" | "gpu_ray" | "gpu_lidar") {
            if child(node, "lidar").is_some() {
                "lidar"
            } else {
                "ray"
            }
        } else if matches!(
            kind.as_str(),
            "depth"
                | "depth_camera"
                | "rgbd"
                | "rgbd_camera"
                | "segmentation"
                | "segmentation_camera"
                | "boundingbox"
                | "boundingbox_camera"
                | "thermal"
                | "thermal_camera"
                | "wideanglecamera"
                | "wide_angle_camera"
        ) {
            "camera"
        } else {
            kind.as_str()
        };
        if [
            "imu",
            "contact",
            "force_torque",
            "magnetometer",
            "altimeter",
            "air_pressure",
            "navsat",
            "gps",
            "air_speed",
            "ray",
            "lidar",
            "camera",
        ]
        .iter()
        .any(|name| {
            child(node, name).is_some()
                && *name != config_tag
                && !(matches!(kind.as_str(), "navsat" | "gps") && matches!(*name, "navsat" | "gps"))
                && !(matches!(kind.as_str(), "ray" | "lidar" | "gpu_ray" | "gpu_lidar")
                    && matches!(*name, "ray" | "lidar"))
        }) {
            return Err(self.err(
                node,
                ErrorCode::Unsupported,
                "mismatched sensor configuration",
            ));
        }
        let update_rate = self.scalar(node, "update_rate", 0.)?;
        // Keep the existing IMU policy; contact retains native unthrottled metadata.
        if kind == "imu" && update_rate < 0. {
            return Err(self.err(node, ErrorCode::InvalidValue, "negative sensor update rate"));
        }
        // Preserve the existing IMU envelope's first-error order before configuration.
        let name = self.name(node)?;
        let pose = self.pose(node)?;
        let always_on = self.boolean(node, "always_on", false)?;
        let enable_metrics = self.boolean(node, "enable_metrics", false)?;
        let config = match kind.as_str() {
            "imu" => SensorConfig::Imu(Box::new(self.imu(node)?)),
            "contact" => SensorConfig::Contact(self.contact(node)?),
            "force_torque" => SensorConfig::ForceTorque(Box::new(self.force_torque(node)?)),
            "magnetometer" => SensorConfig::Magnetometer(Box::new(self.magnetometer(node)?)),
            "altimeter" => SensorConfig::Altimeter(Box::new(self.altimeter(node)?)),
            "air_pressure" => SensorConfig::AirPressure(Box::new(self.air_pressure(node)?)),
            "navsat" | "gps" => SensorConfig::NavSat(Box::new(self.navsat(node, config_tag)?)),
            "air_speed" => SensorConfig::AirSpeed(Box::new(self.air_speed(node)?)),
            "ray" | "lidar" | "gpu_ray" | "gpu_lidar" => {
                SensorConfig::Lidar(Box::new(self.lidar(node, config_tag)?))
            }
            "camera"
            | "depth"
            | "depth_camera"
            | "rgbd"
            | "rgbd_camera"
            | "segmentation"
            | "segmentation_camera"
            | "boundingbox"
            | "boundingbox_camera"
            | "thermal"
            | "thermal_camera"
            | "wideanglecamera"
            | "wide_angle_camera" => SensorConfig::Camera(Box::new(self.camera(node)?)),
            "logical_camera"
            | "multicamera"
            | "rfid"
            | "rfidtag"
            | "sonar"
            | "wireless_receiver"
            | "wireless_transmitter" => SensorConfig::Unconfigured,
            "custom" => SensorConfig::Custom(CustomSensor {
                custom_type,
                xml: self.xml[node.range()].to_owned(),
                element_name: custom_element
                    .map(|extension| extension.tag_name().name().to_owned())
                    .unwrap_or_default(),
                parameters: custom_element
                    .map(|extension| {
                        elements(extension)
                            .map(|item| {
                                (
                                    item.tag_name().name().to_owned(),
                                    item.text().unwrap_or("").trim().to_owned(),
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
            }),
            _ => unreachable!("sensor type checked above"),
        };
        let configuration_xml = if kind == "custom" {
            custom_element.map_or(String::new(), |n| self.xml[n.range()].into())
        } else {
            child(node, config_tag).map_or(String::new(), |n| self.xml[n.range()].into())
        };
        Ok(Sensor {
            name,
            kind,
            pose,
            update_rate,
            always_on,
            visualize: self.boolean(node, "visualize", false)?,
            extensions: elements(node)
                .filter(|child| {
                    child.tag_name().name() == "alwaysOn"
                        || custom_fields.contains(child.tag_name().name())
                })
                .map(|child| self.extension_metadata(child))
                .collect(),
            enable_metrics,
            config,
            plugins: self.plugins(node)?,
            topic: self
                .optional_text(node, "topic")?
                .filter(|topic| topic != "__default__")
                .unwrap_or_default(),
            frame_id: self.optional_text(node, "frame_id")?.unwrap_or_default(),
            gz_frame_id: self.optional_text(node, "gz_frame_id")?,
            configuration_xml,
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
// The 1.4 -> 1.5 table adds this default only to direct root/world model
// joints, not nested models. Imported documents use their own XML root scope.
fn converted_14_model(model: Node<'_, '_>) -> bool {
    let root = model.document().root_element();
    root.attribute("version") == Some("1.4")
        && elements(root).any(|node| {
            if node.has_tag_name("world") {
                elements(node)
                    .any(|child| child.has_tag_name("model") && child.range() == model.range())
            } else {
                node.has_tag_name("model") && node.range() == model.range()
            }
        })
}
// Names from pinned schema declarations and conversion paths remain reserved,
// even when they are not valid direct children of this particular world.
fn known_schema_name(name: &str) -> bool {
    static NAMES: OnceLock<BTreeSet<&str>> = OnceLock::new();
    let names = NAMES.get_or_init(|| {
        include_str!("../data/world_known_schema_names.txt")
            .lines()
            .filter(|entry| !entry.starts_with('#'))
            .collect()
    });
    names.contains(name)
        || (name.bytes().any(|byte| byte.is_ascii_uppercase())
            && names.contains(name.to_ascii_lowercase().as_str()))
}

fn is_pre17(node: Node<'_, '_>) -> bool {
    matches!(
        node.document().root_element().attribute("version"),
        Some("1.4" | "1.5" | "1.6")
    )
}
fn supports_axis_mimic(node: Node<'_, '_>) -> bool {
    matches!(
        node.document().root_element().attribute("version"),
        Some("1.11" | "1.12")
    )
}
fn pose_attributes(node: Node<'_, '_>) -> &'static [&'static str] {
    if is_pre17(node) {
        &["frame", "relative_to", "degrees", "rotation_format"]
    } else {
        &["relative_to", "degrees", "rotation_format"]
    }
}
fn validate_model(model: &Model, graphs: &Graphs) -> Result<(), Diagnostic> {
    validate_joints(
        &model.joints,
        &model.scoped_name,
        &model.name,
        "model",
        graphs,
    )?;
    for nested in &model.models {
        validate_model(nested, graphs)?;
    }
    Ok(())
}
fn validate_joints(
    joints: &[Joint],
    scope: &str,
    name: &str,
    container: &str,
    graphs: &Graphs,
) -> Result<(), Diagnostic> {
    for joint in joints {
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
            if !axis.expressed_in.is_empty()
                && graphs
                    .validate_reference(&axis.expressed_in, scope, &joint.source)
                    .is_err()
            {
                return Err(error(
                    ErrorCode::JointAxisExpressedInInvalid,
                    &joint.source,
                    format!(
                        "axis xyz expressed-in frame with name[{}] specified by joint with name[{}] not found in {container} with name[{}].",
                        axis.expressed_in, joint.name, name
                    ),
                ));
            }
        }
    }
    Ok(())
}

// Keep snapshot expansion temporaries off the recursive model parser's stack.
fn append_merged_element_children(
    children: &mut Vec<ModelElementChild>,
    imported: &Model,
    proxy_name: &str,
) {
    children.push(ModelElementChild {
        tag: "frame".into(),
        name_attribute: Some(proxy_name.into()),
    });
    if let Some(element) = imported.element() {
        children.extend(
            element
                .children()
                .iter()
                .filter(|child| {
                    matches!(
                        child.tag(),
                        "link" | "model" | "joint" | "frame" | "gripper" | "plugin"
                    ) || child.tag().contains(':')
                })
                .cloned(),
        );
    }
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

fn rescope_model(model: &mut Model, scope: &str) {
    model.scoped_name = join(scope, &model.name);
    for link in &mut model.links {
        link.scoped_name = join(&model.scoped_name, &link.name);
    }
    for nested in &mut model.models {
        rescope_model(nested, &model.scoped_name);
    }
}
