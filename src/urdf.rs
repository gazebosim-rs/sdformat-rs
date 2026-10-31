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
    admit_link(link, uri)?;
    admit_extensions(root, link, uri)
}
fn admit_link(link: Node<'_, '_>, uri: &str) -> Result<(), Diagnostic> {
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
    Ok(())
}
fn admit_extensions(
    root: Node<'_, '_>,
    link: Node<'_, '_>,
    uri: &str,
) -> Result<Extensions, Diagnostic> {
    let name = required(link, uri, "name")?;
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
    for (tag, applied) in [
        ("visual", extensions.material.is_some()),
        (
            "collision",
            extensions.mu1.is_some() || extensions.mu2.is_some(),
        ),
    ] {
        if applied
            && child(link, tag).is_some_and(|view| {
                self::name(name, view.attribute("name"), tag).contains("_fixed_joint_lump__")
            })
        {
            // Native skips ordinary link extensions on lump-prefixed view
            // names, including prefixes introduced without any fixed joint.
            return Err(error(
                child(root, "gazebo").unwrap(),
                uri,
                ErrorCode::Unsupported,
                "Gazebo metadata cannot target a native lump-prefixed view name",
            ));
        }
    }
    Ok(extensions)
}
struct Articulation {
    parent: String,
    child: String,
    extension_link: String,
    limit: bool,
}
fn admit_articulation(
    root: Node<'_, '_>,
    uri: &str,
) -> Result<(Articulation, Extensions), Diagnostic> {
    allowed(
        root,
        uri,
        &["link", "joint", "gazebo"],
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
    for link in &links {
        admit_link(*link, uri)?;
    }
    let names = [
        required(links[0], uri, "name")?,
        required(links[1], uri, "name")?,
    ];
    if names[0] == names[1] {
        return Err(error(
            links[1],
            uri,
            ErrorCode::InvalidValue,
            "duplicate URDF link name",
        ));
    }
    let joint = child(root, "joint").unwrap();
    allowed(
        joint,
        uri,
        &["parent", "child", "origin", "axis", "limit", "dynamics"],
        &["name", "type"],
        &[],
    )?;
    required(joint, uri, "name")?;
    if required(joint, uri, "type")? != "continuous" {
        return Err(error(
            joint,
            uri,
            ErrorCode::Unsupported,
            "only one continuous URDF joint is supported",
        ));
    }
    let mut endpoints = Vec::new();
    for tag in ["parent", "child"] {
        let node = child(joint, tag).ok_or_else(|| {
            error(
                joint,
                uri,
                ErrorCode::Missing,
                "URDF joint endpoint is required",
            )
        })?;
        allowed(node, uri, &[], &["link"], &[])?;
        let name = required(node, uri, "link")?;
        if !names.contains(&name) {
            return Err(error(
                node,
                uri,
                ErrorCode::UnknownReference,
                "URDF joint endpoint must name a declared link",
            ));
        }
        endpoints.push(name);
    }
    if endpoints[0] == endpoints[1] {
        return Err(error(
            joint,
            uri,
            ErrorCode::InvalidValue,
            "URDF joint endpoints must be different links",
        ));
    }
    if let Some(node) = child(joint, "origin") {
        origin(node, uri)?;
    }
    if let Some(node) = child(joint, "axis") {
        allowed(node, uri, &[], &["xyz"], &[])?;
        let values = numeric(node, uri, "xyz", 3)?;
        let length = glam::DVec3::from_array([values[0], values[1], values[2]]).length();
        // Native Normalize leaves a tiny boundary interval unnormalized. Keep
        // it outside this subset rather than change the shared SDF math policy.
        if !length.is_finite() || length <= 1e-6 {
            return Err(error(
                node,
                uri,
                ErrorCode::InvalidValue,
                "URDF axis norm must be finite and greater than 1e-6",
            ));
        }
    }
    let limit = child(joint, "limit");
    if let Some(node) = limit {
        allowed(node, uri, &[], &["effort", "velocity"], &[])?;
        numeric(node, uri, "effort", 1)?;
        numeric(node, uri, "velocity", 1)?;
    }
    if let Some(node) = child(joint, "dynamics") {
        allowed(node, uri, &[], &["damping", "friction"], &[])?;
        if node.attribute("damping").is_none() && node.attribute("friction").is_none() {
            return Err(error(
                node,
                uri,
                ErrorCode::Missing,
                "URDF dynamics requires damping or friction",
            ));
        }
        for attr in ["damping", "friction"] {
            if node.attribute(attr).is_some() {
                numeric(node, uri, attr, 1)?;
            }
        }
    }
    let target = if let Some(gazebo) = child(root, "gazebo") {
        let name = required(gazebo, uri, "reference")?;
        links
            .iter()
            .copied()
            .find(|link| link.attribute("name") == Some(name))
            .ok_or_else(|| {
                error(
                    gazebo,
                    uri,
                    ErrorCode::UnknownReference,
                    "Gazebo reference must name one of the two links",
                )
            })?
    } else {
        links[0]
    };
    let extensions = admit_extensions(root, target, uri)?;
    Ok((
        Articulation {
            parent: endpoints[0].into(),
            child: endpoints[1].into(),
            extension_link: required(target, uri, "name")?.into(),
            limit: limit.is_some(),
        },
        extensions,
    ))
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
fn coordinates(value: &urdf_rs::Pose) -> String {
    value
        .xyz
        .iter()
        .chain(value.rpy.iter())
        .map(|v| decimal(*v))
        .collect::<Vec<_>>()
        .join(" ")
}
fn pose(value: &urdf_rs::Pose) -> String {
    element("pose", &coordinates(value))
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
fn link_xml(
    link: &urdf_rs::Link,
    extensions: &Extensions,
    parent_joint: Option<&str>,
) -> Option<String> {
    let inertial = &link.inertial;
    let moments = &inertial.inertia;
    let mut xml = format!("<link name=\"{}\">", escape(&link.name));
    if let Some(joint) = parent_joint {
        xml += &format!("<pose relative_to=\"{}\">0 0 0 0 0 0</pose>", escape(joint));
    }
    xml += &format!(
        "<inertial>{}{}<inertia>",
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
    xml += "</link>";
    Some(xml)
}
fn convert(
    robot: &urdf_rs::Robot,
    extensions: &Extensions,
    articulation: Option<&Articulation>,
) -> Option<String> {
    if !robot.materials.is_empty() {
        return None;
    }
    let mut xml = format!(
        "<sdf version=\"1.12\"><model name=\"{}\">",
        escape(&robot.name)
    );
    if let Some(admitted) = articulation {
        if robot.links.len() != 2 || robot.joints.len() != 1 {
            return None;
        }
        let joint = robot.joints.first()?;
        if joint.joint_type != urdf_rs::JointType::Continuous
            || joint.parent.link != admitted.parent
            || joint.child.link != admitted.child
        {
            return None;
        }
        let empty = Extensions::default();
        let extension = |name: &str| {
            if name == admitted.extension_link {
                extensions
            } else {
                &empty
            }
        };
        let parent = robot
            .links
            .iter()
            .find(|link| link.name == admitted.parent)?;
        let child = robot
            .links
            .iter()
            .find(|link| link.name == admitted.child)?;
        xml += &link_xml(parent, extension(&parent.name), None)?;
        xml += &format!(
            "<joint name=\"{}\" type=\"revolute\"><pose relative_to=\"{}\">{}</pose>{}{}<axis>{}<limit>",
            escape(&joint.name),
            escape(&joint.parent.link),
            coordinates(&joint.origin),
            element("parent", &joint.parent.link),
            element("child", &joint.child.link),
            element(
                "xyz",
                &joint
                    .axis
                    .xyz
                    .iter()
                    .map(|v| decimal(*v))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        );
        // urdf-rs has zero-valued limit defaults even when no URDF limit exists.
        if admitted.limit {
            xml += &element("effort", &decimal(joint.limit.effort));
            xml += &element("velocity", &decimal(joint.limit.velocity));
        }
        xml += "</limit><dynamics>";
        if let Some(dynamics) = &joint.dynamics {
            xml += &element("damping", &decimal(dynamics.damping));
            xml += &element("friction", &decimal(dynamics.friction));
        }
        xml += "</dynamics></axis></joint>";
        xml += &link_xml(child, extension(&child.name), Some(&joint.name))?;
    } else {
        if robot.links.len() != 1 || !robot.joints.is_empty() {
            return None;
        }
        xml += &link_xml(robot.links.first()?, extensions, None)?;
    }
    xml += "</model></sdf>";
    Some(xml)
}
/// Import one bounded URDF link or a two-link continuous-joint model without IO.
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
    let pair = root.has_tag_name("robot")
        && root.children().filter(|n| n.has_tag_name("link")).count() == 2
        && root.children().filter(|n| n.has_tag_name("joint")).count() == 1;
    let (articulation, extensions) = if pair {
        let (joint, extensions) = admit_articulation(root, uri)?;
        (Some(joint), extensions)
    } else {
        (None, admit(root, uri)?)
    };
    let original_source = source(root, uri);
    let robot = urdf_rs::read_from_string(xml).map_err(|e| Diagnostic {
        code: ErrorCode::InvalidValue,
        source: original_source.clone(),
        message: format!("URDF parsing failed: {e}"),
    })?;
    let converted_xml = convert(&robot, &extensions, articulation.as_ref()).ok_or_else(|| {
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
