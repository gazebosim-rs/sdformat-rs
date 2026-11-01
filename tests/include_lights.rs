/*
 * Copyright 2019 Open Source Robotics Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
use serde_json::Value;
use std::cell::RefCell;
#[path = "support/document_light_projection.rs"]
mod projection;
const ORIGINAL: &str = include_str!("fixtures/included_light_original.sdf");
struct Resolver<'a> {
    xml: &'a str,
    requests: RefCell<Vec<(String, String)>>,
}
impl ResourceResolver for Resolver<'_> {
    fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
        self.requests.borrow_mut().push((uri.into(), parent.into()));
        Ok(Resource {
            canonical_uri: "resources/light.sdf".into(),
            xml: self.xml.into(),
        })
    }
}
fn resolver(xml: &str) -> Resolver<'_> {
    Resolver {
        xml,
        requests: RefCell::new(Vec::new()),
    }
}
fn world(extra: &str) -> String {
    format!(
        "<sdf version='1.12'><world name='default'><include><uri>test_light</uri>{extra}</include></world></sdf>"
    )
}
fn close(actual: Transform, expected: &[&str]) {
    assert_eq!(expected.len(), 7);
    for (a, b) in actual
        .translation
        .into_iter()
        .chain(actual.rotation)
        .zip(expected)
    {
        let b = f64::from_bits(b.parse().unwrap());
        assert!((a - b).abs() <= 1e-12, "{a} != {b}");
    }
}
#[test]
fn native_include_values_overrides_sources_and_deferred_frames() {
    let rows: Value = serde_json::from_str(include_str!("include_lights.json")).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 23);
    for row in rows.as_array().unwrap() {
        let resolver = resolver(row["resource_xml"].as_str().unwrap());
        let result = load_str(row["xml"].as_str().unwrap(), "include-world.sdf", &resolver);
        assert!(!resolver.requests.borrow().is_empty(), "{}", row["id"]);
        assert!(
            resolver
                .requests
                .borrow()
                .iter()
                .all(|(uri, parent)| uri == "test_light" && parent == "include-world.sdf")
        );
        if let Some(error) = row["rust_error"].as_str() {
            assert_eq!(
                format!("{:?}", result.unwrap_err().code),
                error,
                "{}",
                row["id"]
            );
            continue;
        }
        assert_eq!(row["native"]["exit_code"], 0);
        let sdf = result.unwrap_or_else(|e| panic!("{} {e:?}", row["id"]));
        let world = &sdf.worlds[0];
        let native = row["native"]["stdout"].as_str().unwrap();
        assert!(native.lines().any(|l| l == "ERRORS 0"));
        let values: Vec<_> = native
            .lines()
            .filter(|line| line.starts_with("OK "))
            .collect();
        assert_eq!(world.lights.len(), values.len());
        assert_eq!(world.models.len(), 0);
        for (light, line) in world.lights.iter().zip(values) {
            let fields: Vec<_> = line.split_whitespace().collect();
            let actual = projection::projection(light);
            assert_eq!(actual.len(), 33);
            assert_eq!(fields.len(), 33);
            projection::compare(
                &actual,
                &fields.iter().map(|v| (*v).to_string()).collect::<Vec<_>>(),
            );
            close(light.pose.transform, &fields[3..10]);
            let source = light.source.as_ref().unwrap();
            assert_eq!(source.uri, "resources/light.sdf");
            let native_source = native
                .lines()
                .find(|line| line.starts_with("SOURCE "))
                .unwrap()
                .split_whitespace()
                .collect::<Vec<_>>();
            assert_eq!(source.line, native_source[2].parse::<u32>().unwrap());
            assert_eq!(source.column, if source.line == 3 { 3 } else { 21 });
            let resolved = sdf.resolve_pose(&light.pose, &world.name, &world.name, &world.name);
            if row["id"] == "resource_unknown_frame" {
                assert_eq!(resolved.unwrap_err().code, ErrorCode::UnknownReference);
                assert_eq!(
                    sdf.validate_non_frame_poses().unwrap_err().code,
                    ErrorCode::UnknownReference
                );
            } else {
                // Every successful semantic query is emitted in the same light order.
                let index = world
                    .lights
                    .iter()
                    .position(|l| std::ptr::eq(l, light))
                    .unwrap();
                let pose = native
                    .lines()
                    .filter(|line| line.starts_with("POSE world OK "))
                    .nth(index)
                    .unwrap();
                close(
                    resolved.unwrap(),
                    &pose.split_whitespace().skip(3).collect::<Vec<_>>(),
                );
                sdf.validate_non_frame_poses().unwrap();
            }
        }
    }
}
#[test]
fn original_includes_light_subtrees_retain_exact_assertions() {
    let resolver = resolver(ORIGINAL);
    let sdf = load_str(
        include_str!("fixtures/include_lights_original.sdf"),
        "includes.sdf#lights",
        &resolver,
    )
    .unwrap();
    assert_eq!(sdf.version, "1.8");
    let world = sdf.worlds.first().expect("original WorldByIndex0 presence");
    assert_eq!(world.lights.len(), 2);
    assert!(!world.lights.iter().any(|light| light.name.is_empty()));
    let light = world
        .lights
        .first()
        .expect("original LightByIndex0 presence");
    assert_eq!(light.name, "point_light");
    assert_eq!(light.light_type, LightType::Point);
    assert!(!light.cast_shadows);
    projection::pose(
        light.pose.transform,
        &[1_f64, 2., 10., 0., 0., 0., 1.].map(|v| v.to_bits().to_string()),
    );
    assert_eq!(light.pose.transform.translation, [1., 2., 10.]);
    assert_eq!(light.pose.transform.rotation, [0., 0., 0., 1.]);
    assert_eq!(light.pose.relative_to, "world");
    for (actual, expected) in [
        (light.attenuation_range, 123.5_f64),
        (light.linear_attenuation_factor, 1.),
        (light.constant_attenuation_factor, 0.),
        (light.quadratic_attenuation_factor, 20.2),
    ] {
        assert!(actual.to_bits().abs_diff(expected.to_bits()) <= 4);
    }
    let overridden = world
        .lights
        .get(1)
        .expect("original LightByIndex1 presence");
    assert_eq!(overridden.name, "override_light_name");
    assert_eq!(overridden.pose.transform.translation, [4., 5., 6.]);
    assert_eq!(overridden.pose.transform.rotation, [0., 0., 0., 1.]);
    assert_eq!(overridden.pose.relative_to, "");
    assert_eq!(resolver.requests.borrow().len(), 2);
    // Native Element pointers and OriginalVersion getters remain outside this owned value API.
    for light in &world.lights {
        assert_eq!(
            light.source,
            Some(Source {
                uri: "resources/light.sdf".into(),
                line: 3,
                column: 3
            })
        );
    }
    let mut clone = overridden.clone();
    clone.name = "changed".into();
    clone.intensity = 99.;
    assert_eq!(overridden.name, "override_light_name");
    assert_eq!(overridden.intensity, 1.);
}
#[test]
fn light_includes_are_world_only_and_keep_declaration_order() {
    let resolver = resolver(ORIGINAL);
    for xml in [
        "<sdf version='1.12'><include><uri>test_light</uri></include></sdf>",
        "<sdf version='1.12'><model name='robot'><link name='base'/><include><uri>test_light</uri></include></model></sdf>",
    ] {
        assert_eq!(
            load_str(xml, "include-world.sdf", &resolver)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
    let xml = world("<name>middle</name>")
        .replace("<include>", "<light name='first' type='point'/><include>")
        .replace("</include>", "</include><light name='last' type='spot'/>");
    let sdf = load_str(&xml, "include-world.sdf", &resolver).unwrap();
    assert_eq!(
        sdf.worlds[0]
            .lights
            .iter()
            .map(|v| v.name.as_str())
            .collect::<Vec<_>>(),
        ["first", "middle", "last"]
    );
    assert_eq!(
        sdf.worlds[0].lights[0].source.as_ref().unwrap().uri,
        "include-world.sdf"
    );
    assert_eq!(
        sdf.worlds[0].lights[1].source.as_ref().unwrap().uri,
        "resources/light.sdf"
    );
}
#[test]
fn light_include_errors_are_atomic_strict_and_source_located() {
    for extra in [
        "<unknown/>",
        "<name>__bad__</name>",
        "<pose rotation_format='bad'>0 0 0 0 0 0</pose>",
        "<pose relative_to='missing'/>",
    ] {
        let resolver = resolver(ORIGINAL);
        let result = load_str(&world(extra), "include-world.sdf", &resolver);
        if extra.contains("relative_to") {
            assert_eq!(
                result.unwrap().validate_non_frame_poses().unwrap_err().code,
                ErrorCode::UnknownReference
            );
        } else {
            assert!(result.is_err());
        }
    }
    for xml in [
        world("<name>duplicate</name>").replace(
            "</include>",
            "</include><light name='duplicate' type='point'/>",
        ),
        world("<name>duplicate</name>").replace(
            "</world>",
            "<include><uri>test_light</uri><name>duplicate</name></include></world>",
        ),
    ] {
        assert_eq!(
            load_str(&xml, "include-world.sdf", &resolver(ORIGINAL))
                .unwrap_err()
                .code,
            ErrorCode::DuplicateName
        );
    }
    for resource in [
        "<sdf version='1.12'><light name='lamp' type='point'><unknown/></light></sdf>",
        "<sdf version='1.12'><light name='lamp' type='point' bad='1'/></sdf>",
        "<sdf version='1.12'><light name='lamp' type='point'><intensity>nan</intensity></light></sdf>",
    ] {
        let resolver = resolver(resource);
        let error = load_str(&world(""), "include-world.sdf", &resolver).unwrap_err();
        assert_eq!(error.source.uri, "resources/light.sdf");
    }
    assert_eq!(
        load_str(&world(""), "include-world.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::Resource
    );
}
