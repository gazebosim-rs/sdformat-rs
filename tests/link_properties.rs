// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{LinkProperties, NoResources, load_str};
use serde_json::Value;

#[test]
fn native_link_properties_defaults_and_boolean_boundaries() {
    let rows: Vec<Value> = serde_json::from_str(include_str!("link_properties.json")).unwrap();
    for row in rows {
        let result = load_str(row["xml"].as_str().unwrap(), "link.sdf", &NoResources);
        let tokens = row["tokens"].as_array().unwrap();
        assert_eq!(
            result.is_ok(),
            tokens[0] == "0",
            "{}: {result:?}",
            row["id"]
        );
        if let Ok(sdf) = result {
            let links = &sdf.models[0].links;
            assert_eq!(
                links.len(),
                tokens[1].as_str().unwrap().parse::<usize>().unwrap()
            );
            for (index, link) in links.iter().enumerate() {
                for (offset, value) in [
                    link.properties.enable_wind,
                    link.properties.enable_gravity,
                    link.properties.kinematic,
                    link.properties.self_collide,
                ]
                .into_iter()
                .enumerate()
                {
                    assert_eq!(
                        value,
                        tokens[2 + index * 4 + offset] == "1",
                        "{} flag {offset}",
                        row["id"]
                    );
                }
            }
            let mut next = 2 + links.len() * 4;
            if tokens.get(next).is_some_and(|token| token == "A") {
                next += 1;
                let joint = &sdf.models[0].joints[0];
                for index in [0, 1, 2, usize::MAX] {
                    let axis = joint.axis(index);
                    assert_eq!(axis.is_some(), tokens[next] == "1");
                    let aliases_second = match (axis, joint.axis2.as_ref()) {
                        (None, None) => true,
                        (Some(a), Some(b)) => std::ptr::eq(a, b),
                        _ => false,
                    };
                    assert_eq!(aliases_second, tokens[next + 1] == "1");
                    next += 2;
                }
            }
            assert_eq!(next, tokens.len());
        }
    }
}

#[test]
fn link_properties_are_owned_values_with_explicit_native_defaults() {
    let owned = {
        let sdf = load_str("<sdf version='1.12'><model name='m'><link name='l'><gravity>false</gravity><kinematic>true</kinematic><enable_wind>true</enable_wind><self_collide>true</self_collide></link></model></sdf>", "link.sdf", &NoResources).unwrap();
        sdf.models[0].links[0].properties
    };
    assert_eq!(
        owned,
        LinkProperties {
            enable_wind: true,
            enable_gravity: false,
            kinematic: true,
            self_collide: true
        }
    );
    assert_eq!(
        LinkProperties::default(),
        LinkProperties {
            enable_wind: false,
            enable_gravity: true,
            kinematic: false,
            self_collide: false
        }
    );
}

#[test]
fn joint_axis_index_clamps_positive_indices_to_the_stored_second_axis() {
    let sdf = load_str("<sdf version='1.12'><model name='m'><link name='a'/><link name='b'/><joint name='j' type='universal'><parent>a</parent><child>b</child><axis><xyz>1 0 0</xyz></axis><axis2><xyz>0 1 0</xyz></axis2></joint></model></sdf>", "axes", &NoResources).unwrap();
    let joint = &sdf.models[0].joints[0];
    assert!(std::ptr::eq(
        joint.axis(0).unwrap(),
        joint.axis.as_ref().unwrap()
    ));
    assert!(std::ptr::eq(
        joint.axis(1).unwrap(),
        joint.axis2.as_ref().unwrap()
    ));
    assert!(std::ptr::eq(
        joint.axis(2).unwrap(),
        joint.axis2.as_ref().unwrap()
    ));
    assert!(std::ptr::eq(
        joint.axis(usize::MAX).unwrap(),
        joint.axis2.as_ref().unwrap()
    ));
}
