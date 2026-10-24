// Exact upstream assertion port: FrameSemantics.resolveAgainstOwnFrameIsExact,
// sdformat/src/FrameSemantics_TEST.cc, OSRF, Apache-2.0.
use sdformat::*;
#[test]
fn resolve_against_own_frame_is_exact() {
    let sdf = load_str(
        include_str!("fixtures/joint_axis_in_rotated_frame.sdf"),
        "joint_axis_in_rotated_frame.sdf",
        &NoResources,
    )
    .unwrap();
    let model = &sdf.models[0];
    for local in ["__model__", "chassis", "wheel", "wheel_joint"] {
        let frame = if local == "__model__" {
            model.scoped_name.clone()
        } else {
            format!("{}::{local}", model.scoped_name)
        };
        let pose = sdf.pose_of(&frame, &frame).unwrap();
        assert_eq!(pose.translation, [0.; 3], "{local}");
        assert_eq!(pose.rotation, [0., 0., 0., 1.], "{local}");
    }
    for name in ["wheel_joint", "wheel_joint_exact"] {
        let joint = model
            .joints
            .iter()
            .find(|joint| joint.name == name)
            .unwrap();
        let axis = joint.axis.as_ref().unwrap();
        let frame = format!("{}::{name}", model.scoped_name);
        assert_eq!(
            sdf.resolve_axis(axis, &frame, &frame).unwrap(),
            [0., 0., 1.],
            "{name}"
        );
    }
}
#[test]
fn small_relative_offsets_survive_large_common_world_origins() {
    let xml = "<sdf version='1.12'><model name='m'><pose>1e16 -1e16 1e16 0.1 0.2 0.3</pose><link name='l'><pose>0.125 0 0 0 0 0</pose></link><frame name='f' attached_to='l'><pose>0 0.25 0 0 0 0</pose></frame></model></sdf>";
    let sdf = load_str(xml, "precision", &NoResources).unwrap();
    assert_eq!(
        sdf.pose_of("m::l", "m").unwrap().translation,
        [0.125, 0., 0.]
    );
    assert_eq!(
        sdf.pose_of("m::f", "m").unwrap().translation,
        [0.125, 0.25, 0.]
    );
    assert_eq!(
        sdf.pose_of("m::f", "m::l").unwrap().translation,
        [0., 0.25, 0.]
    );
    assert_eq!(
        sdf.pose_of("m::l", "m::f").unwrap().translation,
        [0., -0.25, 0.]
    );
    assert_eq!(sdf.pose_of("m::l", "m::l").unwrap(), Transform::default());
    let pose = Pose {
        transform: Transform {
            translation: [0.01, 0.02, 0.03],
            rotation: [0., 0., 0., -1.],
        },
        relative_to: String::new(),
    };
    assert_eq!(
        sdf.resolve_pose(&pose, "m", "m::l", "m::l").unwrap(),
        pose.transform
    );
    assert_eq!(
        sdf.pose_of("missing", "missing").unwrap_err().code,
        ErrorCode::UnknownReference
    );
}
#[test]
fn sibling_and_root_queries_use_the_right_common_ancestor() {
    let xml = "<sdf version='1.12'><world name='w'><model name='m'><pose>10 0 0 0 0 0</pose><link name='l'><pose>0.125 0 0 0 0 0</pose></link><link name='r'><pose>0 0.25 0 0 0 0</pose></link></model></world></sdf>";
    let sdf = load_str(xml, "siblings", &NoResources).unwrap();
    assert_eq!(
        sdf.pose_of("w::m::l", "w::m::r").unwrap().translation,
        [0.125, -0.25, 0.]
    );
    assert_eq!(
        sdf.pose_of("w::m::l", "::").unwrap().translation,
        [10.125, 0., 0.]
    );
    assert_eq!(
        sdf.pose_of("::", "w::m::l").unwrap().translation,
        [-10.125, 0., 0.]
    );
    assert_eq!(
        sdf.pose_of("w::m::l", "w").unwrap().translation,
        [10.125, 0., 0.]
    );
}
