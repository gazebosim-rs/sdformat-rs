// Portions ported from sdformat test/integration/nested_model.cc and model_dom.cc.
// Copyright Open Source Robotics Foundation. Apache-2.0, see LICENSE.
use sdformat::*;

fn load(xml: &str) -> Sdf {
    load_str(xml, "test.sdf", &NoResources).unwrap()
}
fn model(contents: &str) -> Sdf {
    load(&format!(
        "<sdf version='1.12'><model name='m'>{contents}</model></sdf>"
    ))
}
fn close(actual: Transform, expected: [f64; 6]) {
    let [x, y, z, r, p, h] = expected;
    for (a, b) in actual.translation.into_iter().zip([x, y, z]) {
        assert!((a - b).abs() < 1e-6, "{actual:?} != {expected:?}");
    }
    let q = glam::DQuat::from_euler(glam::EulerRot::ZYX, h, p, r);
    assert!(
        glam::DQuat::from_array(actual.rotation).abs_diff_eq(q, 1e-6)
            || glam::DQuat::from_array(actual.rotation).abs_diff_eq(-q, 1e-6)
    );
}

// All six original pose assertions, preserving gz-math's 1e-6 equality tolerance.
#[test]
fn nested_reference_pose_relative_to() {
    let sdf = load(include_str!(
        "fixtures/model_relative_to_nested_reference.sdf"
    ));
    let parent = &sdf.models[0];
    let half_pi = std::f64::consts::FRAC_PI_2;
    for (name, expected) in [
        ("M2", [1., 0., -2., 0., half_pi, 0.]),
        ("M3", [1., 1., -3., 0., half_pi, 0.]),
        ("M4", [2., 0., -4., 0., half_pi, 0.]),
        ("M5", [2., 0., -6., 0., half_pi, 0.]),
        ("M6", [1., 1., -6., half_pi, half_pi, 0.]),
        ("M7", [1., -6., -1., 0., 0., -half_pi]),
    ] {
        assert!(parent.models.iter().any(|m| m.name == name));
        close(
            sdf.pose_of(&format!("parent_model::{name}"), "parent_model")
                .unwrap(),
            expected,
        );
    }
}
#[test]
fn nested_reference_pose_relative_to_in_world() {
    let sdf = load(include_str!(
        "fixtures/world_relative_to_nested_reference.sdf"
    ));
    let world = &sdf.worlds[0];
    let h = std::f64::consts::FRAC_PI_2;
    for (name, expected) in [
        ("F2", [1., 0., -2., 0., h, 0.]),
        ("F3", [1., 1., -3., 0., h, 0.]),
        ("F4", [2., 0., -4., 0., h, 0.]),
        ("F5", [2., 0., -6., 0., h, 0.]),
        ("F6", [1., 1., -6., h, h, 0.]),
        ("F7", [1., -6., -1., 0., 0., -h]),
    ] {
        assert!(world.frames.iter().any(|f| f.name == name));
        close(
            sdf.pose_of(&format!("{}::{name}", world.name), &world.name)
                .unwrap(),
            expected,
        );
    }
}
#[test]
fn nested_explicit_canonical_link() {
    let sdf = load(include_str!("fixtures/nested_explicit_canonical_link.sdf"));
    let model = &sdf.models[0];
    assert_eq!("top", model.name);
    assert!(model.links.is_empty());
    assert!(model.joints.is_empty());
    assert_eq!(model.frames.len(), 1);
    assert_eq!(model.frames[0].name, "F");
    assert!(model.frames.get(1).is_none());
    assert_eq!(model.models.len(), 1);
    assert!(model.models.get(1).is_none());
    assert!(std::ptr::eq(
        model.models.iter().find(|m| m.name == "nested").unwrap(),
        &model.models[0]
    ));
    assert_eq!("nested::link2", model.canonical_link);
    assert_eq!("nested", model.models[0].name);
    assert_eq!("link2", model.models[0].links[1].name);
    assert_eq!("top::nested::link2", sdf.attached_body("top").unwrap());
    assert_eq!("top::nested::link2", sdf.attached_body("top::F").unwrap());
}
#[test]
fn nested_attachment_frames_and_joint() {
    let sdf = load(include_str!("fixtures/model_nested_frame_attached_to.sdf"));
    let root = "nested_model_frame_attached_to";
    for (frame, body) in [
        ("F0", "M1::L"),
        ("F1", "M1::L"),
        ("F2", "M1::L"),
        ("F3", "M1::M2::L"),
        ("F4", "M1::M2::L"),
    ] {
        assert_eq!(
            format!("{root}::{body}"),
            sdf.attached_body(&format!("{root}::{frame}")).unwrap()
        );
    }
}

#[test]
fn complete_minimal_world() {
    let xml = r#"<sdf version="1.12"><world name="default"><gravity>0 0 -9.81</gravity>
    <model name="robot" canonical_link="base"><pose>1 2 3 0 0 0</pose>
    <plugin name="control" filename="control.so"><ns:config xmlns:ns="urn:test" ns:key="v">&amp;<child/></ns:config></plugin>
    <frame name="mount" attached_to="base"><pose>0 0 2 0 0 0</pose></frame>
    <link name="base"><pose>0 1 0 0 0 0</pose><plugin name="link" filename="l"/>
    <inertial><mass>2</mass><inertia><ixx>2</ixx><iyy>3</iyy><izz>4</izz><ixy>0.1</ixy><ixz>0.2</ixz><iyz>0.3</iyz></inertia><pose>0 0 0.5 0 0 0</pose></inertial>
    <collision name="box"><pose relative_to="mount">1 0 0 0 0 0</pose><geometry><box><size>1 2 3</size></box></geometry></collision>
    <sensor name="imu" type="imu"><pose relative_to="__model__">0 0 4 0 0 0</pose><always_on>true</always_on><update_rate>100</update_rate><topic>/imu</topic><imu><angular_velocity><x><noise type="gaussian"><stddev>0.01</stddev></noise></x></angular_velocity></imu></sensor></link>
    <link name="tip"/><joint name="hinge" type="revolute"><parent>base</parent><child>tip</child><pose>0 0 1 0 0 0</pose><axis><xyz expressed_in="mount">2 0 0</xyz><limit><lower>-1</lower><upper>2</upper><effort>5</effort><velocity>3</velocity></limit></axis></joint>
    </model><plugin name="world" filename="w"/></world></sdf>"#;
    let sdf = load(xml);
    let world = &sdf.worlds[0];
    assert_eq!(world.name, "default");
    assert_eq!(world.gravity, [0., 0., -9.81]);
    assert_eq!(world.plugins.len(), 1);
    let m = &world.models[0];
    let link = &m.links[0];
    assert!(!m.is_static);
    assert_eq!(link.inertial.mass, 2.);
    assert_eq!(link.inertial.inertia, [2., 3., 4., 0.1, 0.2, 0.3]);
    assert_eq!(
        link.collisions[0].geometry,
        Geometry::Box { size: [1., 2., 3.] }
    );
    close(
        sdf.pose_of("default::robot::mount", "default").unwrap(),
        [1., 3., 5., 0., 0., 0.],
    );
    close(
        sdf.resolve_pose(
            &link.collisions[0].pose,
            &m.scoped_name,
            &link.scoped_name,
            "default",
        )
        .unwrap(),
        [2., 3., 5., 0., 0., 0.],
    );
    close(
        sdf.resolve_pose(
            &link.sensors[0].pose,
            &m.scoped_name,
            &link.scoped_name,
            "default",
        )
        .unwrap(),
        [1., 2., 7., 0., 0., 0.],
    );
    assert_eq!(link.sensors[0].update_rate, 100.);
    assert!(link.sensors[0].always_on);
    assert_eq!(link.sensors[0].topic, "/imu");
    assert!(link.sensors[0].configuration_xml.contains("0.01"));
    assert_eq!(link.plugins[0].filename, "l");
    assert!(
        m.plugins[0]
            .xml
            .contains("xmlns:ns=\"urn:test\" ns:key=\"v\">&amp;")
    );
    let joint = &m.joints[0];
    assert_eq!(joint.kind, JointKind::Revolute);
    let axis = joint.axis.as_ref().unwrap();
    assert_eq!(axis.xyz, [1., 0., 0.]);
    assert_eq!(
        (axis.lower, axis.upper, axis.effort, axis.velocity),
        (-1., 2., 5., 3.)
    );
    close(
        sdf.pose_of("default::robot::hinge", "default::robot::tip")
            .unwrap(),
        [0., 0., 1., 0., 0., 0.],
    );
    assert_eq!(
        sdf.attached_body("default::robot").unwrap(),
        "default::robot::base"
    );
}
#[test]
fn shapes_defaults_and_pose_formats() {
    let sdf = model(
        r#"<static>1</static><link name="l">
    <collision name="s"><geometry><sphere/></geometry></collision>
    <collision name="c"><geometry><cylinder><radius>2</radius><length>3</length></cylinder></geometry></collision>
    <collision name="p"><geometry><plane><normal>0 0 2</normal><size>3 4</size></plane></geometry></collision>
    <collision name="b"><geometry><box/></geometry></collision>
    <sensor name="s" type="imu"/>
    </link><model name="nested"><pose degrees="true">0 1 0 0 0 90</pose><link name="l"><pose rotation_format="quat_xyzw">1 0 0 0 0 0 2</pose></link></model>"#,
    );
    let m = &sdf.models[0];
    let link = &m.links[0];
    assert_eq!(link.inertial.mass, 1.);
    assert_eq!(link.inertial.inertia, [1., 1., 1., 0., 0., 0.]);
    assert_eq!(link.collisions[0].geometry, Geometry::Sphere { radius: 1. });
    assert_eq!(
        link.collisions[1].geometry,
        Geometry::Cylinder {
            radius: 2.,
            length: 3.
        }
    );
    assert_eq!(
        link.collisions[2].geometry,
        Geometry::Plane {
            normal: [0., 0., 1.],
            size: [3., 4.]
        }
    );
    assert_eq!(link.collisions[3].geometry, Geometry::Box { size: [1.; 3] });
    assert_eq!(link.sensors[0].configuration_xml, "");
    assert_eq!(link.sensors[0].topic, "");
    assert!(!link.sensors[0].always_on);
    close(
        sdf.pose_of("m::nested::l", "::").unwrap(),
        [0., 2., 0., 0., 0., std::f64::consts::FRAC_PI_2],
    );
    assert_eq!(sdf.attached_body("m").unwrap(), "::");
    let sdf = load(
        "<sdf version='1.12'><world name='w'><model name='m'><static>true</static></model></world></sdf>",
    );
    assert_eq!(sdf.attached_body("w::m").unwrap(), "w");
    let sdf = model(
        "<link name='l'/><link name='r'/><joint name='j' type='prismatic'><parent>world</parent><child>r</child><axis><limit><effort>-1</effort><velocity>-1</velocity></limit></axis></joint>",
    );
    let axis = sdf.models[0].joints[0].axis.as_ref().unwrap();
    assert_eq!(axis.xyz, [0., 0., 1.]);
    assert!(axis.effort.is_infinite());
    assert!(axis.velocity.is_infinite());
    let sdf = model(
        "<link name='l'/><link name='r'/><joint name='j' type='continuous'><parent>l</parent><child>r</child><axis/></joint>",
    );
    assert_eq!(sdf.models[0].joints[0].kind, JointKind::Continuous);
    let sdf = load(
        "<sdf version='1.9'><world name='w'><model name='m'><link name='l'><collision name='p'><geometry><plane/></geometry></collision><collision name='c'><geometry><cylinder/></geometry></collision></link></model><frame name='f'/></world></sdf>",
    );
    assert_eq!(sdf.worlds[0].gravity, [0., 0., -9.8]);
    assert_eq!(sdf.attached_body("w::f").unwrap(), "w");
}

struct MemoryResolver;
impl ResourceResolver for MemoryResolver {
    fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
        assert!(!parent.is_empty());
        let xml = match uri {
            "inner" => {
                "<sdf version='1.12'><model name='original'><pose>10 0 0 0 0 0</pose><link name='l'/><include><uri>leaf</uri></include></model></sdf>"
            }
            "leaf" => "<sdf version='1.12'><model name='leaf'><link name='l'/></model></sdf>",
            "cycle" => {
                "<sdf version='1.12'><model name='cycle'><include><uri>cycle</uri></include></model></sdf>"
            }
            "badxml" => "<sdf>",
            "badroot" => "<model/>",
            "badchild" => "<sdf version='1.12'><world name='w'/></sdf>",
            "multiple" => "<sdf version='1.12'><model name='a'/><model name='b'/></sdf>",
            "empty" => "<sdf version='1.12'/>",
            _ => return Err("not found".into()),
        };
        Ok(Resource {
            canonical_uri: format!("memory:{uri}"),
            xml: xml.into(),
        })
    }
}
#[test]
fn injected_includes_override_pose_name_and_static_with_provenance() {
    let sdf=load_str("<sdf version='1.12'><world name='w'><include><uri>inner</uri><name>renamed</name><pose>0 2 0 0 0 0</pose><static>false</static></include></world></sdf>","memory:root",&MemoryResolver).unwrap();
    let m = &sdf.worlds[0].models[0];
    assert_eq!(m.name, "renamed");
    assert_eq!(m.source.uri, "memory:inner");
    assert_eq!(m.links[0].source.uri, "memory:inner");
    assert_eq!(m.models[0].source.uri, "memory:leaf");
    close(
        sdf.pose_of("w::renamed::leaf::l", "w").unwrap(),
        [0., 2., 0., 0., 0., 0.],
    );
    assert_eq!(sdf.attached_body("w::renamed").unwrap(), "w::renamed::l");
    let sdf = load_str(
        "<sdf version='1.12'><model name='outer'><include><uri>leaf</uri></include></model></sdf>",
        "memory:root",
        &MemoryResolver,
    )
    .unwrap();
    assert_eq!(sdf.attached_body("outer").unwrap(), "outer::leaf::l");
    for (uri, code) in [
        ("cycle", ErrorCode::Cycle),
        ("badxml", ErrorCode::Xml),
        ("badroot", ErrorCode::Xml),
        ("badchild", ErrorCode::Unsupported),
        ("multiple", ErrorCode::InvalidValue),
        ("empty", ErrorCode::InvalidValue),
        ("missing", ErrorCode::Resource),
    ] {
        let result = load_str(
            &format!(
                "<sdf version='1.12'><model name='m'><include><uri>{uri}</uri></include></model></sdf>"
            ),
            "memory:root",
            &MemoryResolver,
        );
        assert_eq!(result.unwrap_err().code, code, "{uri}");
    }
}
#[test]
fn diagnostic_contract_and_query_errors() {
    let error=load_str("<sdf version='1.12'>\n<model name='m'><link name='l'/><frame name='f' attached_to='missing'/></model></sdf>","broken.sdf",&NoResources).unwrap_err();
    assert_eq!(error.code, ErrorCode::UnknownReference);
    assert_eq!(error.source.line, 2);
    assert!(error.to_string().contains("broken.sdf:2:"));
    assert!(std::error::Error::source(&error).is_none());
    let sdf = model("<link name='l'/>");
    assert_eq!(
        sdf.pose_of("missing", "m").unwrap_err().code,
        ErrorCode::UnknownReference
    );
    assert_eq!(
        sdf.pose_of("m", "missing").unwrap_err().code,
        ErrorCode::UnknownReference
    );
    assert_eq!(
        sdf.attached_body("missing").unwrap_err().code,
        ErrorCode::UnknownReference
    );
    let pose = Pose {
        relative_to: "missing".into(),
        ..Pose::default()
    };
    assert_eq!(
        sdf.resolve_pose(&pose, "m", "m::l", "::").unwrap_err().code,
        ErrorCode::UnknownReference
    );
}

#[test]
fn invalid_documents_fail_atomically() {
    for (xml, code) in [
        ("<sdf>", ErrorCode::Xml),
        ("<model/>", ErrorCode::Xml),
        ("<sdf/>", ErrorCode::Version),
        ("<sdf version='1.7'/>", ErrorCode::Version),
        ("<sdf version='1.12'/>", ErrorCode::Missing),
        (
            "<sdf version='1.12'><world name='w'/><model name='m'/></sdf>",
            ErrorCode::InvalidValue,
        ),
        (
            "<sdf version='1.12'><world name='w'/><world name='w'/></sdf>",
            ErrorCode::DuplicateName,
        ),
        (
            "<sdf version='1.12'><world name='w'><gravity>0 NaN 0</gravity></world></sdf>",
            ErrorCode::InvalidValue,
        ),
    ] {
        assert_eq!(
            load_str(xml, "test.sdf", &NoResources).unwrap_err().code,
            code,
            "{xml}"
        );
    }
    for (content, code) in [
        ("", ErrorCode::Missing),
        ("<link/>", ErrorCode::Missing),
        ("<link name=''/>", ErrorCode::Missing),
        ("<link name='world'/>", ErrorCode::InvalidValue),
        ("<link name='__model__'/>", ErrorCode::InvalidValue),
        ("<link name='a::b'/>", ErrorCode::InvalidValue),
        ("<link name=' l'/>", ErrorCode::InvalidValue),
        ("<link name='l' foo='1'/>", ErrorCode::Unsupported),
        (
            "<link name='l' xmlns:x='urn:x' x:foo='1'/>",
            ErrorCode::Unsupported,
        ),
        (
            "<link name='l'><visual name='v'/></link>",
            ErrorCode::Unsupported,
        ),
        (
            "<static>true</static><static>false</static>",
            ErrorCode::InvalidValue,
        ),
        ("<static>maybe</static>", ErrorCode::InvalidValue),
        ("<link name='l'/><link name='l'/>", ErrorCode::DuplicateName),
        (
            "<link name='l'/><frame name='l'/>",
            ErrorCode::DuplicateName,
        ),
        (
            "<link name='l'><pose relative_to='no'>0 0 0 0 0 0</pose></link>",
            ErrorCode::UnknownReference,
        ),
        (
            "<link name='l'/><frame name='a' attached_to='b'/><frame name='b' attached_to='a'/>",
            ErrorCode::Cycle,
        ),
        (
            "<link name='l'/><frame name='a'><pose relative_to='b'/></frame><frame name='b'><pose relative_to='a'/></frame>",
            ErrorCode::Cycle,
        ),
        (
            "<link name='l'><pose>1 nope 3 0 0 0</pose></link>",
            ErrorCode::InvalidValue,
        ),
        (
            "<link name='l'><pose>0 0 0</pose></link>",
            ErrorCode::InvalidValue,
        ),
        (
            "<link name='l'><pose>0 0 NaN 0 0 0</pose></link>",
            ErrorCode::InvalidValue,
        ),
        (
            "<link name='l'><pose degrees='maybe'>0 0 0 0 0 0</pose></link>",
            ErrorCode::InvalidValue,
        ),
        (
            "<link name='l'><pose rotation_format='bad'>0 0 0 0 0 0</pose></link>",
            ErrorCode::InvalidValue,
        ),
        (
            "<link name='l'><pose degrees='true' rotation_format='quat_xyzw'>0 0 0 0 0 0 1</pose></link>",
            ErrorCode::InvalidValue,
        ),
        (
            "<link name='l'><pose rotation_format='quat_xyzw'>0 0 0 0 0 0 0</pose></link>",
            ErrorCode::InvalidValue,
        ),
        (
            "<link name='l'><inertial><mass>0</mass></inertial></link>",
            ErrorCode::InvalidValue,
        ),
        (
            "<link name='l'><inertial><mass>NaN</mass></inertial></link>",
            ErrorCode::InvalidValue,
        ),
        (
            "<link name='l'><inertial><mass>bad</mass></inertial></link>",
            ErrorCode::InvalidValue,
        ),
        (
            "<link name='l'><inertial><inertia><ixx>-1</ixx></inertia></inertial></link>",
            ErrorCode::InvalidValue,
        ),
        (
            "<link name='l'><inertial><inertia><ixx>4</ixx></inertia></inertial></link>",
            ErrorCode::InvalidValue,
        ),
        (
            "<link name='l'><inertial><inertia><ixy>2</ixy></inertia></inertial></link>",
            ErrorCode::InvalidValue,
        ),
        (
            "<link name='l'><collision name='c'/></link>",
            ErrorCode::Missing,
        ),
        (
            "<link name='l'><collision name='c'><geometry/></collision></link>",
            ErrorCode::InvalidValue,
        ),
        (
            "<link name='l'><collision name='c'><geometry><sphere/><box/></geometry></collision></link>",
            ErrorCode::InvalidValue,
        ),
        (
            "<link name='l'><collision name='c'><geometry><sphere><radius>-1</radius></sphere></geometry></collision></link>",
            ErrorCode::InvalidValue,
        ),
        (
            "<link name='l'><collision name='c'><geometry><plane><normal>0 0 0</normal></plane></geometry></collision></link>",
            ErrorCode::InvalidValue,
        ),
        (
            "<link name='l'><collision name='c'><geometry><sphere/></geometry></collision><collision name='c'><geometry><box/></geometry></collision></link>",
            ErrorCode::DuplicateName,
        ),
        (
            "<link name='l'><sensor name='s' type='imu'/><sensor name='s' type='imu'/></link>",
            ErrorCode::DuplicateName,
        ),
        (
            "<link name='l'><sensor name='s' type='camera'/></link>",
            ErrorCode::Unsupported,
        ),
        (
            "<link name='l'><sensor name='s' type='imu'><update_rate>-1</update_rate></sensor></link>",
            ErrorCode::InvalidValue,
        ),
        (
            "<link name='l'/><joint name='j' type='ball'><parent>l</parent><child>l</child></joint>",
            ErrorCode::Unsupported,
        ),
        (
            "<link name='l'/><joint name='j' type='revolute'><parent>l</parent><child>l</child></joint>",
            ErrorCode::Missing,
        ),
        (
            "<link name='l'/><joint name='j' type='fixed'><child>l</child></joint>",
            ErrorCode::Missing,
        ),
        (
            "<link name='l'/><joint name='j' type='fixed'><parent>l</parent><child>world</child></joint>",
            ErrorCode::InvalidValue,
        ),
        (
            "<link name='l'/><joint name='j' type='fixed'><parent>l</parent><child>l</child></joint>",
            ErrorCode::InvalidValue,
        ),
        (
            "<link name='l'/><joint name='j' type='fixed'><parent>no</parent><child>l</child></joint>",
            ErrorCode::UnknownReference,
        ),
        (
            "<link name='l'/><joint name='j' type='revolute'><parent>world</parent><child>l</child><axis><xyz>0 0 0</xyz></axis></joint>",
            ErrorCode::InvalidValue,
        ),
        (
            "<link name='l'/><joint name='j' type='revolute'><parent>world</parent><child>l</child><axis><xyz expressed_in='no'>1 0 0</xyz></axis></joint>",
            ErrorCode::UnknownReference,
        ),
        (
            "<link name='l'/><joint name='j' type='revolute'><parent>world</parent><child>l</child><axis><limit><lower>2</lower><upper>1</upper></limit></axis></joint>",
            ErrorCode::InvalidValue,
        ),
        (
            "<include><uri>file://missing</uri></include>",
            ErrorCode::Resource,
        ),
        (
            "<include merge='true'><uri>missing</uri></include>",
            ErrorCode::Unsupported,
        ),
    ] {
        let xml = format!("<sdf version='1.12'><model name='m'>{content}</model></sdf>");
        assert_eq!(
            load_str(&xml, "test.sdf", &NoResources).unwrap_err().code,
            code,
            "{content}"
        );
    }
}

#[test]
fn attachment_cycles_are_independent_of_pose_cycles() {
    let xml = "<sdf version='1.12'><model name='m'><link name='l'/><frame name='a' attached_to='b'><pose relative_to='__model__'/></frame><frame name='b' attached_to='a'><pose relative_to='__model__'/></frame></model></sdf>";
    assert_eq!(
        load_str(xml, "test", &NoResources).unwrap_err().code,
        ErrorCode::Cycle
    );
    let sdf = model("<link name='l'/>");
    assert_eq!(
        sdf.resolve_pose(&Pose::default(), "m", "m::l", "unknown")
            .unwrap_err()
            .code,
        ErrorCode::UnknownReference
    );
    // Model frames in different scopes cannot become this model's canonical link.
    let xml = "<sdf version='1.12'><model name='m' canonical_link='__model__'><link name='l'/></model></sdf>";
    assert_eq!(
        load_str(xml, "test", &NoResources).unwrap_err().code,
        ErrorCode::Cycle
    );
}
#[test]
fn include_failures_and_depth_are_bounded() {
    for name in ["bad::name", "__reserved", "world"] {
        let xml = format!(
            "<sdf version='1.12'><model name='m'><include><uri>leaf</uri><name>{name}</name></include></model></sdf>"
        );
        assert_eq!(
            load_str(&xml, "root", &MemoryResolver).unwrap_err().code,
            ErrorCode::InvalidValue
        );
    }
    struct Deep;
    impl ResourceResolver for Deep {
        fn resolve(&self, uri: &str, _: &str) -> Result<Resource, String> {
            let n = uri.parse::<u32>().unwrap();
            Ok(Resource {
                canonical_uri: uri.into(),
                xml: format!(
                    "<sdf version='1.12'><model name='m{n}'><include><uri>{}</uri></include></model></sdf>",
                    n + 1
                ),
            })
        }
    }
    let xml = "<sdf version='1.12'><model name='m'><include><uri>0</uri></include></model></sdf>";
    assert_eq!(
        load_str(xml, "root", &Deep).unwrap_err().code,
        ErrorCode::Resource
    );
    let sdf = model("<link name='l'><inertial><mass/></inertial></link>");
    assert_eq!(sdf.models[0].links[0].inertial.mass, 1.);
}

#[test]
fn axis_resolution_rotates_vectors_and_resolves_joint_bodies() {
    let sdf = model(
        "<link name='l'/><link name='r'/><frame name='f' attached_to='l'><pose>0 0 0 0 0 1.5707963267948966</pose></frame><joint name='j' type='revolute'><parent>world</parent><child>r</child><axis><xyz expressed_in='f'>1 0 0</xyz></axis></joint>",
    );
    let axis = sdf.models[0].joints[0].axis.as_ref().unwrap();
    let v = sdf.resolve_axis(axis, "m::j", "m").unwrap();
    assert!(v[0].abs() < 1e-12);
    assert!((v[1] - 1.).abs() < 1e-12);
    assert_eq!(v[2], 0.);
    assert_eq!(sdf.resolve_name("m", "__model__").unwrap(), "m");
    assert_eq!(sdf.resolve_name("m", "r").unwrap(), "m::r");
    assert_eq!(sdf.resolve_name("m", "world").unwrap(), "::");
    let axis = Axis {
        xyz: [0., 1., 0.],
        expressed_in: String::new(),
        lower: 0.,
        upper: 1.,
        effort: 1.,
        velocity: 1.,
    };
    assert_eq!(sdf.resolve_axis(&axis, "m::j", "m").unwrap(), [0., 1., 0.]);
    assert_eq!(
        sdf.resolve_axis(&axis, "missing", "m").unwrap_err().code,
        ErrorCode::UnknownReference
    );
    let axis = Axis {
        expressed_in: "missing".into(),
        ..axis
    };
    assert_eq!(
        sdf.resolve_axis(&axis, "m::j", "m").unwrap_err().code,
        ErrorCode::UnknownReference
    );
    for xml in [
        "<sdf version='1.12'><model name='m'><link name='l'/><frame name='f' attached_to='l'/><joint name='j' type='fixed'><parent>l</parent><child>f</child></joint></model></sdf>",
        "<sdf version='1.12'><world name='w'><frame name='f'/><model name='m'><link name='l'/><joint name='j' type='fixed'><parent>l</parent><child>world</child></joint></model></world></sdf>",
    ] {
        assert_eq!(
            load_str(xml, "test", &NoResources).unwrap_err().code,
            ErrorCode::InvalidValue
        );
    }
}

#[test]
fn canonical_frame_alias_and_external_numeric_invariants() {
    // The pinned C++ Root accepts a frame alias as canonical_link.
    let sdf = load(
        "<sdf version='1.12'><model name='m' canonical_link='f'><link name='l'/><frame name='f' attached_to='l'/></model></sdf>",
    );
    assert_eq!(sdf.attached_body("m").unwrap(), "m::l");
    for transform in [
        Transform {
            translation: [f64::INFINITY, 0., 0.],
            ..Transform::default()
        },
        Transform {
            rotation: [0.; 4],
            ..Transform::default()
        },
        Transform {
            rotation: [f64::NAN, 0., 0., 1.],
            ..Transform::default()
        },
    ] {
        let pose = Pose {
            transform,
            relative_to: String::new(),
        };
        assert_eq!(
            sdf.resolve_pose(&pose, "m", "m::l", "::").unwrap_err().code,
            ErrorCode::InvalidValue
        );
    }
    let xml = "<sdf version='1.12'><model name='m'><pose>1e308 0 0 0 0 0</pose><link name='l'><pose>1e308 0 0 0 0 0</pose></link></model></sdf>";
    assert_eq!(
        load_str(xml, "overflow", &NoResources).unwrap_err().code,
        ErrorCode::InvalidValue
    );
}

#[test]
fn pinned_cpp_differential_acceptance_corpus() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("differential.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let result = load_str(
            case["xml"].as_str().unwrap(),
            case["id"].as_str().unwrap(),
            &NoResources,
        );
        assert_eq!(
            result.is_ok(),
            case["accepted"].as_bool().unwrap(),
            "{case}"
        );
        if let Some(expected) = case.get("link_mass") {
            assert_eq!(
                result.as_ref().unwrap().models[0].links[0].inertial.mass,
                expected.as_f64().unwrap(),
                "{case}"
            );
        }
        if let Some(expected) = case.get("model_name") {
            let model = &result.as_ref().unwrap().models[0];
            assert_eq!(model.name, expected.as_str().unwrap());
            assert_eq!(model.plugins[0].name, case["plugin_name"].as_str().unwrap());
            assert_eq!(
                model.plugins[0].filename,
                case["plugin_filename"].as_str().unwrap()
            );
        }
        if let Some(expected) = case.get("link_pose") {
            let sdf = result.as_ref().unwrap();
            let model = &sdf.models[0];
            close(
                sdf.pose_of(&model.links[0].scoped_name, &model.scoped_name)
                    .unwrap(),
                std::array::from_fn(|i| expected[i].as_f64().unwrap()),
            );
        }
        if let Some(expected) = case.get("model_pose") {
            let sdf = result.unwrap();
            let model = if sdf.models.is_empty() {
                &sdf.worlds[0].models[0]
            } else {
                &sdf.models[0]
            };
            let values = std::array::from_fn(|i| expected[i].as_f64().unwrap());
            close(sdf.pose_of(&model.scoped_name, "::").unwrap(), values);
        }
    }
}

#[test]
fn nested_scalar_schema_and_model_depth_are_validated() {
    for contents in [
        "<link name='l'><collision name='c'><geometry><box><size><unexpected/></size></box></geometry></collision></link>",
        "<link name='l'/><joint name='j' type='fixed'><parent attribute='x'>world</parent><child>l</child></joint>",
    ] {
        let xml = format!("<sdf version='1.12'><model name='m'>{contents}</model></sdf>");
        assert_eq!(
            load_str(&xml, "nested-scalar", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
    let xml = format!(
        "<sdf version='1.12'>{}<link name='l'/>{}</sdf>",
        "<model name='m'>".repeat(66),
        "</model>".repeat(66)
    );
    assert_eq!(
        load_str(&xml, "deep", &NoResources).unwrap_err().code,
        ErrorCode::Resource
    );
    let xml = "<sdf version='1.12'><model name='m'><link name='l'><collision name='c'><geometry><plane><normal>1e308 0 0</normal></plane></geometry></collision></link></model></sdf>";
    assert_eq!(
        load_str(xml, "large-normal", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::InvalidValue
    );
}

#[test]
fn local_schema_references_cannot_escape_their_model_scope() {
    for reference in ["world", "m", "m::r", "::"] {
        let xml = format!(
            "<sdf version='1.12'><world name='w'><model name='m'><link name='l'><pose relative_to='{reference}'/></link><link name='r'/></model></world></sdf>"
        );
        assert_eq!(
            load_str(&xml, "local", &NoResources).unwrap_err().code,
            ErrorCode::UnknownReference,
            "{reference}"
        );
    }
    for (body, code) in [
        (
            "<model name='m'><link name='l'/><frame name='f' attached_to='world'/></model>",
            ErrorCode::UnknownReference,
        ),
        (
            "<model name='m' canonical_link='world'><link name='l'/></model>",
            ErrorCode::UnknownReference,
        ),
        (
            "<model name='m'><pose relative_to='world'/><link name='l'/></model>",
            ErrorCode::InvalidValue,
        ),
        (
            "<model name='m'><link name='l'/><model name='n'><pose relative_to='world'/><link name='l'/></model></model>",
            ErrorCode::UnknownReference,
        ),
        (
            "<world name='w'><frame name='f'><pose relative_to='__model__'/></frame></world>",
            ErrorCode::UnknownReference,
        ),
        (
            "<model name='m'><link name='l'/></model><model name='n'><link name='l'/></model>",
            ErrorCode::InvalidValue,
        ),
        (
            "<model name='m'><link name='l'><collision name='c'><pose relative_to='world'/><geometry><box/></geometry></collision></link></model>",
            ErrorCode::UnknownReference,
        ),
        (
            "<model name='m'><link name='l'/><joint name='j' type='revolute'><parent>world</parent><child>l</child><axis><xyz expressed_in='world'>1 0 0</xyz></axis></joint></model>",
            ErrorCode::UnknownReference,
        ),
    ] {
        assert_eq!(
            load_str(
                &format!("<sdf version='1.12'>{body}</sdf>"),
                "scope",
                &NoResources
            )
            .unwrap_err()
            .code,
            code,
            "{body}"
        );
    }
    let sdf = load(
        "<sdf version='1.12'><world name='w'><frame name='f' attached_to='world'/><model name='m'><pose relative_to='world'/><link name='l'><pose relative_to='__model__'/></link></model></world></sdf>",
    );
    assert_eq!(sdf.attached_body("w::f").unwrap(), "w");
    close(sdf.pose_of("w::m::l", "w").unwrap(), [0.; 6]);
    let sdf = load(
        "<sdf version='1.12'><model name='m' canonical_link='missing'><static>true</static><link name='l'/></model></sdf>",
    );
    assert_eq!(sdf.attached_body("m").unwrap(), "::");
    assert_eq!(
        sdf.resolve_pose(&Pose::default(), "m", "missing", "::")
            .unwrap_err()
            .code,
        ErrorCode::UnknownReference
    );
    let axis = Axis {
        xyz: [1., 0., 0.],
        expressed_in: "world".into(),
        lower: 0.,
        upper: 1.,
        effort: 0.,
        velocity: 0.,
    };
    assert_eq!(
        sdf.resolve_axis(&axis, "m::joint", "::").unwrap_err().code,
        ErrorCode::UnknownReference
    );
}

#[test]
fn relative_query_overflow_reports_an_error() {
    let sdf = load(
        "<sdf version='1.12'><world name='w'><model name='a'><pose>1e308 0 0 0 0 0</pose><link name='l'/></model><model name='b'><pose>-1e308 0 0 0 0 0</pose><link name='l'/></model></world></sdf>",
    );
    assert_eq!(
        sdf.pose_of("w::a", "w::b").unwrap_err().code,
        ErrorCode::InvalidValue
    );
    let pose = Pose {
        transform: Transform {
            translation: [1e308, 0., 0.],
            ..Transform::default()
        },
        relative_to: String::new(),
    };
    assert_eq!(
        sdf.resolve_pose(&pose, "w::a", "w::a::l", "w")
            .unwrap_err()
            .code,
        ErrorCode::InvalidValue
    );
}
