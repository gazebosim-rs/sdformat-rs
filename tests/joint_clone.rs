// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{JointKind, NoResources, load_str};

#[test]
fn retained_joint_configuration_outlives_dom_and_clones_without_aliasing() {
    let xml = "<sdf version='1.12'><model name='m'><link name='a'/><link name='b'/><joint name='j' type='universal'><parent>a</parent><child>b</child><pose relative_to='a'>1 2 3 0 0 0</pose><axis><xyz expressed_in='a'>0 1 0</xyz><limit><lower>2</lower><upper>1</upper><effort>4</effort><velocity>5</velocity><stiffness>6</stiffness><dissipation>7</dissipation></limit><dynamics><damping>8</damping><friction>9</friction><spring_reference>10</spring_reference><spring_stiffness>11</spring_stiffness></dynamics></axis><axis2><xyz>1 0 0</xyz></axis2></joint></model></sdf>";
    let retained = {
        let sdf = load_str(xml, "joint-source", &NoResources).unwrap();
        sdf.models[0].joints[0].clone()
    };
    assert_eq!(retained.kind, JointKind::Universal);
    assert_eq!(
        (&*retained.name, &*retained.parent, &*retained.child),
        ("j", "a", "b")
    );
    assert_eq!(retained.pose.relative_to, "a");
    assert_eq!(retained.pose.transform.translation, [1., 2., 3.]);
    assert_eq!(retained.source.uri, "joint-source");
    let axis = retained.axis(0).unwrap();
    assert_eq!(axis.xyz, [0., 1., 0.]);
    assert_eq!(axis.expressed_in, "a");
    assert_eq!(
        (axis.lower, axis.upper, axis.effort, axis.velocity),
        (2., 1., 4., 5.)
    );
    assert_eq!((axis.stiffness, axis.dissipation), (6., 7.));
    assert_eq!(
        (
            axis.dynamics.damping,
            axis.dynamics.friction,
            axis.dynamics.spring_reference,
            axis.dynamics.spring_stiffness
        ),
        (8., 9., 10., 11.)
    );
    let mut copy = retained.clone();
    copy.name.clear();
    copy.parent.clear();
    copy.child.clear();
    copy.pose.relative_to.clear();
    copy.source.uri.clear();
    copy.axis.as_mut().unwrap().expressed_in.clear();
    copy.axis
        .as_mut()
        .unwrap()
        .dynamics
        .xml
        .as_mut()
        .unwrap()
        .clear();
    copy.axis
        .as_mut()
        .unwrap()
        .dynamics
        .source
        .as_mut()
        .unwrap()
        .uri
        .clear();
    copy.axis2.as_mut().unwrap().xyz = [0., 0., 1.];
    assert_eq!(retained.name, "j");
    assert_eq!(retained.parent, "a");
    assert_eq!(retained.child, "b");
    assert_eq!(retained.pose.relative_to, "a");
    assert_eq!(retained.source.uri, "joint-source");
    assert_eq!(retained.axis(0).unwrap().expressed_in, "a");
    assert_eq!(
        retained
            .axis(0)
            .unwrap()
            .dynamics
            .source
            .as_ref()
            .unwrap()
            .uri,
        "joint-source"
    );
    assert!(
        retained
            .axis(0)
            .unwrap()
            .dynamics
            .xml
            .as_ref()
            .unwrap()
            .contains("<damping>8</damping>")
    );
    assert_eq!(retained.axis(1).unwrap().xyz, [1., 0., 0.]);
}
