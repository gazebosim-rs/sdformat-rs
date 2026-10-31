// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Bounded URDF admission and conversion; resource and runtime work stay external.
use crate::xml::{Document, Node};
use crate::{Diagnostic, ErrorCode, NoResources, Sdf, Source};
use quick_xml::escape::escape;
use std::collections::BTreeSet;

/// Converted SDF and distinct immutable input/conversion snapshots.
#[derive(Debug)]
pub struct UrdfImport {
    pub sdf: Sdf,
    original_source: Source,
    original_xml: String,
    converted_source: Source,
    converted_xml: String,
}
impl UrdfImport {
    pub fn original_source(&self) -> &Source {
        &self.original_source
    }
    pub fn original_xml(&self) -> &str {
        &self.original_xml
    }
    pub fn converted_source(&self) -> &Source {
        &self.converted_source
    }
    pub fn converted_xml(&self) -> &str {
        &self.converted_xml
    }
}
fn source(node: Node<'_, '_>, uri: &str) -> Source {
    let position = node.position();
    Source {
        uri: uri.into(),
        line: position.row,
        column: position.col,
    }
}
fn error(node: Node<'_, '_>, uri: &str, code: ErrorCode, message: &str) -> Diagnostic {
    Diagnostic {
        code,
        source: source(node, uri),
        message: message.into(),
    }
}
fn child<'a, 'i>(node: Node<'a, 'i>, tag: &str) -> Option<Node<'a, 'i>> {
    node.children().find(|n| n.has_tag_name(tag))
}
fn allowed(
    node: Node<'_, '_>,
    uri: &str,
    tags: &[&str],
    attrs: &[&str],
    repeated: &[&str],
) -> Result<(), Diagnostic> {
    for attr in node.attributes() {
        if !attrs.contains(&attr.name()) {
            return Err(error(
                node,
                uri,
                ErrorCode::Unsupported,
                "unsupported URDF attribute",
            ));
        }
    }
    let mut seen = BTreeSet::new();
    for value in node.children() {
        let name = value.tag_name().name();
        if !tags.contains(&name) {
            return Err(error(
                value,
                uri,
                ErrorCode::Unsupported,
                "unsupported URDF element",
            ));
        }
        if !repeated.contains(&name) && !seen.insert(name) {
            return Err(error(
                value,
                uri,
                ErrorCode::InvalidValue,
                "duplicate URDF element",
            ));
        }
    }
    Ok(())
}
fn required<'a>(node: Node<'a, '_>, uri: &str, name: &str) -> Result<&'a str, Diagnostic> {
    node.attribute(name)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            error(
                node,
                uri,
                ErrorCode::Missing,
                "required URDF attribute is missing",
            )
        })
}
fn numeric(
    node: Node<'_, '_>,
    uri: &str,
    attr: &str,
    count: usize,
) -> Result<Vec<f64>, Diagnostic> {
    let raw = required(node, uri, attr)?;
    // urdf-rs filters malformed vector tokens; do not let it discard input.
    // Native urdfdom splits vectors on spaces and requires whole scalar tokens.
    let words: Vec<_> = if count == 1 {
        vec![raw]
    } else {
        raw.split(' ').filter(|s| !s.is_empty()).collect()
    };
    let values = words
        .iter()
        .map(|s| {
            s.trim_start_matches(|c: char| c.is_ascii_whitespace())
                .parse::<f64>()
        })
        .collect::<Result<Vec<_>, _>>();
    match values {
        Ok(values) if values.len() == count && values.iter().all(|v| v.is_finite()) => Ok(values),
        _ => Err(error(
            node,
            uri,
            ErrorCode::InvalidValue,
            "invalid finite URDF numeric attribute",
        )),
    }
}
fn origin(node: Node<'_, '_>, uri: &str) -> Result<(), Diagnostic> {
    allowed(node, uri, &[], &["xyz", "rpy"], &[])?;
    for attr in ["xyz", "rpy"] {
        if node.attribute(attr).is_some() {
            numeric(node, uri, attr, 3)?;
        }
    }
    Ok(())
}
fn geometry(node: Node<'_, '_>, uri: &str) -> Result<(), Diagnostic> {
    allowed(node, uri, &["sphere", "box", "cylinder"], &[], &[])?;
    let shapes: Vec<_> = node.children().collect();
    if shapes.len() != 1 {
        return Err(error(
            node,
            uri,
            ErrorCode::InvalidValue,
            "URDF geometry requires one shape",
        ));
    }
    let shape = shapes[0];
    let attrs: &[&str] = match shape.tag_name().name() {
        "sphere" => &["radius"],
        "box" => &["size"],
        _ => &["radius", "length"],
    };
    allowed(shape, uri, &[], attrs, &[])?;
    for attr in attrs {
        if numeric(shape, uri, attr, if *attr == "size" { 3 } else { 1 })?
            .iter()
            .any(|v| *v <= 0.)
        {
            return Err(error(
                shape,
                uri,
                ErrorCode::InvalidValue,
                "URDF dimensions must be positive",
            ));
        }
    }
    Ok(())
}
fn view(node: Node<'_, '_>, uri: &str) -> Result<(), Diagnostic> {
    allowed(node, uri, &["origin", "geometry"], &["name"], &[])?;
    if let Some(value) = child(node, "origin") {
        origin(value, uri)?;
    }
    let shape = child(node, "geometry").ok_or_else(|| {
        error(
            node,
            uri,
            ErrorCode::Missing,
            "URDF visual/collision geometry is missing",
        )
    })?;
    geometry(shape, uri)
}
#[derive(Default)]
struct Extensions {
    material: Option<String>,
    mu1: Option<f64>,
    mu2: Option<f64>,
}
fn admit(root: Node<'_, '_>, uri: &str) -> Result<Extensions, Diagnostic> {
    if !root.has_tag_name("robot") {
        return Err(error(root, uri, ErrorCode::Xml, "URDF root must be robot"));
    }
    allowed(
        root,
        uri,
        &["link", "gazebo"],
        &["name", "version"],
        &["link", "gazebo"],
    )?;
    required(root, uri, "name")?;
    if root.attribute("version").is_some_and(|v| v != "1.0") {
        return Err(error(
            root,
            uri,
            ErrorCode::Version,
            "only URDF 1.0 is supported",
        ));
    }
    let links: Vec<_> = root.children().filter(|n| n.has_tag_name("link")).collect();
    if links.len() != 1 {
        return Err(error(
            root,
            uri,
            ErrorCode::Unsupported,
            "URDF import requires exactly one link",
        ));
    }
    let link = links[0];
    allowed(
        link,
        uri,
        &["inertial", "visual", "collision"],
        &["name"],
        &["visual", "collision"],
    )?;
    let name = required(link, uri, "name")?;
    if name == "world" {
        return Err(error(
            link,
            uri,
            ErrorCode::Unsupported,
            "URDF world dummy links are unsupported",
        ));
    }
    let inertial = child(link, "inertial")
        .ok_or_else(|| error(link, uri, ErrorCode::Missing, "URDF inertial is required"))?;
    allowed(inertial, uri, &["origin", "mass", "inertia"], &[], &[])?;
    if let Some(value) = child(inertial, "origin") {
        origin(value, uri)?;
    }
    let mass = child(inertial, "mass")
        .ok_or_else(|| error(inertial, uri, ErrorCode::Missing, "URDF mass is required"))?;
    allowed(mass, uri, &[], &["value"], &[])?;
    numeric(mass, uri, "value", 1)?;
    let inertia = child(inertial, "inertia").ok_or_else(|| {
        error(
            inertial,
            uri,
            ErrorCode::Missing,
            "URDF inertia is required",
        )
    })?;
    let moments = ["ixx", "ixy", "ixz", "iyy", "iyz", "izz"];
    allowed(inertia, uri, &[], &moments, &[])?;
    for attr in moments {
        numeric(inertia, uri, attr, 1)?;
    }
    for tag in ["visual", "collision"] {
        let values: Vec<_> = link.children().filter(|n| n.has_tag_name(tag)).collect();
        if values.len() > 1 {
            return Err(error(
                link,
                uri,
                ErrorCode::Unsupported,
                "multiple URDF visual/collision declarations are unsupported",
            ));
        }
        for value in values {
            view(value, uri)?;
        }
    }
    let gazebo: Vec<_> = root
        .children()
        .filter(|n| n.has_tag_name("gazebo"))
        .collect();
    if gazebo.len() > 1 {
        return Err(error(
            root,
            uri,
            ErrorCode::Unsupported,
            "multiple Gazebo extension blocks are unsupported",
        ));
    }
    let mut extensions = Extensions::default();
    for node in gazebo {
        allowed(node, uri, &["material", "mu1", "mu2"], &["reference"], &[])?;
        if required(node, uri, "reference")? != name {
            return Err(error(
                node,
                uri,
                ErrorCode::UnknownReference,
                "Gazebo reference must name the single link",
            ));
        }
        for value in node.children() {
            allowed(value, uri, &[], &[], &[])?;
            let text = value.text().unwrap_or("").trim_matches([' ', '\t', '\n']);
            if text.is_empty() {
                return Err(error(
                    value,
                    uri,
                    ErrorCode::Missing,
                    "Gazebo extension value is missing",
                ));
            }
            if value.has_tag_name("material") {
                extensions.material = Some(text.into());
            } else {
                let number = crate::number::scalar_float(text)
                    .map_err(|code| error(value, uri, code, "invalid Gazebo friction scalar"))?;
                if number < 0. {
                    return Err(error(
                        value,
                        uri,
                        ErrorCode::InvalidValue,
                        "Gazebo friction must be nonnegative",
                    ));
                }
                if value.has_tag_name("mu1") {
                    extensions.mu1 = Some(number);
                } else {
                    extensions.mu2 = Some(number);
                }
            }
        }
    }
    if extensions.material.is_some() && child(link, "visual").is_none()
        || (extensions.mu1.is_some() || extensions.mu2.is_some())
            && child(link, "collision").is_none()
    {
        return Err(error(
            root,
            uri,
            ErrorCode::Unsupported,
            "Gazebo metadata requires its visual/collision target",
        ));
    }
    Ok(extensions)
}
fn decimal(value: f64) -> String {
    // Native URDF conversion suppresses negative zero in generated scalars.
    if value == 0. {
        "0".into()
    } else {
        value.to_string()
    }
}
fn element(name: &str, text: &str) -> String {
    format!("<{name}>{}</{name}>", escape(text))
}
fn pose(value: &urdf_rs::Pose) -> String {
    element(
        "pose",
        &value
            .xyz
            .iter()
            .chain(value.rpy.iter())
            .map(|v| decimal(*v))
            .collect::<Vec<_>>()
            .join(" "),
    )
}
fn shape(value: &urdf_rs::Geometry) -> Option<String> {
    let xml = match value {
        urdf_rs::Geometry::Sphere { radius } => {
            format!("<sphere>{}</sphere>", element("radius", &decimal(*radius)))
        }
        urdf_rs::Geometry::Box { size } => format!(
            "<box>{}</box>",
            element(
                "size",
                &size
                    .iter()
                    .map(|v| decimal(*v))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        ),
        urdf_rs::Geometry::Cylinder { radius, length } => format!(
            "<cylinder>{}{}</cylinder>",
            element("radius", &decimal(*radius)),
            element("length", &decimal(*length))
        ),
        urdf_rs::Geometry::Mesh { .. } | urdf_rs::Geometry::Capsule { .. } => return None,
    };
    Some(format!("<geometry>{xml}</geometry>"))
}
fn name(link: &str, explicit: Option<&str>, suffix: &str) -> String {
    let base = explicit.filter(|s| !s.is_empty()).unwrap_or(link);
    let value = format!("{base}_{suffix}");
    if value.starts_with(link) {
        value
    } else {
        format!("{link}_fixed_joint_lump__{value}")
    }
}
fn convert(robot: &urdf_rs::Robot, extensions: &Extensions) -> Option<String> {
    if robot.links.len() != 1 || !robot.joints.is_empty() || !robot.materials.is_empty() {
        return None;
    }
    let link = robot.links.first()?;
    let inertial = &link.inertial;
    let moments = &inertial.inertia;
    let mut xml = format!(
        "<sdf version=\"1.12\"><model name=\"{}\"><link name=\"{}\"><inertial>{}{}<inertia>",
        escape(&robot.name),
        escape(&link.name),
        pose(&inertial.origin),
        element("mass", &decimal(inertial.mass.value))
    );
    for (tag, value) in [
        ("ixx", moments.ixx),
        ("iyy", moments.iyy),
        ("izz", moments.izz),
        ("ixy", moments.ixy),
        ("ixz", moments.ixz),
        ("iyz", moments.iyz),
    ] {
        xml += &element(tag, &decimal(value));
    }
    xml += "</inertia></inertial>";
    for value in &link.collision {
        xml += &format!(
            "<collision name=\"{}\">{}{}",
            escape(name(&link.name, value.name.as_deref(), "collision")),
            pose(&value.origin),
            shape(&value.geometry)?
        );
        if extensions.mu1.is_some() || extensions.mu2.is_some() {
            xml += "<surface><friction><ode>";
            for (tag, value) in [("mu", extensions.mu1), ("mu2", extensions.mu2)] {
                if let Some(value) = value {
                    xml += &element(tag, &decimal(value));
                }
            }
            xml += "</ode></friction></surface>";
        }
        xml += "</collision>";
    }
    for value in &link.visual {
        xml += &format!(
            "<visual name=\"{}\">{}{}",
            escape(name(&link.name, value.name.as_deref(), "visual")),
            pose(&value.origin),
            shape(&value.geometry)?
        );
        if let Some(material) = &extensions.material {
            xml += &format!(
                "<material><script>{}{}</script></material>",
                element("name", material),
                element("uri", "file://media/materials/scripts/gazebo.material")
            );
        }
        xml += "</visual>";
    }
    xml += "</link></model></sdf>";
    Some(xml)
}
/// Import one bounded URDF 1.0 link without IO, joints or runtime activation.
/// The SDF view uses an explicit generated URI; original bytes remain separate.
pub fn load_urdf_str(xml: &str, uri: &str) -> Result<UrdfImport, Diagnostic> {
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
    let extensions = admit(root, uri)?;
    let original_source = source(root, uri);
    let robot = urdf_rs::read_from_string(xml).map_err(|e| Diagnostic {
        code: ErrorCode::InvalidValue,
        source: original_source.clone(),
        message: format!("URDF parsing failed: {e}"),
    })?;
    let converted_xml = convert(&robot, &extensions).ok_or_else(|| {
        error(
            root,
            uri,
            ErrorCode::Unsupported,
            "unadmitted parsed URDF structure",
        )
    })?;
    let converted_uri = format!("generated:urdf-to-sdf:{uri}");
    let sdf =
        crate::load_str(&converted_xml, &converted_uri, &NoResources).map_err(|e| Diagnostic {
            source: original_source.clone(),
            message: format!("URDF-to-SDF validation failed: {}", e.message),
            ..e
        })?;
    Ok(UrdfImport {
        sdf,
        original_source,
        original_xml: xml.into(),
        converted_source: Source {
            uri: converted_uri,
            line: 1,
            column: 1,
        },
        converted_xml,
    })
}
