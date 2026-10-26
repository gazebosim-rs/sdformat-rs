// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Supplemental native projections; only root_include_original.rs ports an original.
use sdformat::{ErrorCode, Geometry, Model, Resource, ResourceResolver, Sdf, Transform, load_str};
use serde_json::{Value, json};
use std::cell::Cell;

struct Resolver<'a> {
    xml: &'a str,
    calls: Cell<usize>,
}
impl ResourceResolver for Resolver<'_> {
    fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
        assert_eq!(uri, "model://root-include");
        assert_eq!(parent, "root-include.sdf");
        self.calls.set(self.calls.get() + 1);
        Ok(Resource {
            canonical_uri: "resolved://root-include/model.sdf".into(),
            xml: self.xml.into(),
        })
    }
}
fn corpus() -> Value {
    serde_json::from_str(include_str!("root_include_cases.json")).unwrap()
}
fn pose(actual: Transform, expected: &Value) {
    let expected = expected.as_array().unwrap();
    assert_eq!(expected.len(), 7);
    for (a, e) in actual
        .translation
        .into_iter()
        .chain(actual.rotation)
        .zip(expected)
    {
        assert!((a - e.as_f64().unwrap()).abs() <= 1e-12, "{a} != {e}");
    }
}
fn model(sdf: &Sdf, actual: &Model, native: &Value) {
    assert_eq!(actual.name, native["name"]);
    assert_eq!(json!(actual.raw_namespace), native["raw"]);
    assert_eq!(json!(actual.namespace()), native["resolved"]);
    assert_eq!(actual.is_static, native["static"]);
    assert_eq!(actual.canonical_link, native["canonical"]);
    assert_eq!(actual.placement_frame, native["placement"]);
    assert_eq!(actual.pose.relative_to, native["relative"]);
    pose(actual.pose.transform, &native["pose"]);
    assert_eq!(native["pose_errors"], json!([]));
    pose(
        sdf.pose_of(&actual.scoped_name, "::").unwrap(),
        &native["resolved_pose"],
    );
    assert_eq!(
        actual.links.len(),
        native["links"].as_u64().unwrap() as usize
    );
    assert_eq!(
        json!(actual.frames.iter().map(|f| &f.name).collect::<Vec<_>>()),
        native["frames"]
    );
    assert_eq!(
        json!(
            actual
                .plugins
                .iter()
                .map(|p| json!({"name":p.name,"filename":p.filename}))
                .collect::<Vec<_>>()
        ),
        native["plugins"]
    );
    assert_eq!(
        actual.models.len(),
        native["models"].as_array().unwrap().len()
    );
    for (a, n) in actual
        .models
        .iter()
        .zip(native["models"].as_array().unwrap())
    {
        model(sdf, a, n);
    }
    for (name, expected) in actual
        .links
        .iter()
        .map(|l| (&l.scoped_name, &l.name))
        .zip(native["links_data"].as_array().unwrap())
        .chain(
            actual
                .frames
                .iter()
                .map(|f| (&f.name, &f.name))
                .zip(native["frames_data"].as_array().unwrap()),
        )
    {
        let (scoped, local) = name;
        assert_eq!(local, expected["name"].as_str().unwrap());
        let scoped = if scoped.contains("::") {
            scoped.clone()
        } else {
            format!("{}::{scoped}", actual.scoped_name)
        };
        let semantic = &expected["semantic"];
        assert_eq!(semantic["default"]["errors"], json!([]));
        assert_eq!(semantic["model"]["errors"], json!([]));
        pose(
            sdf.pose_of(&scoped, &actual.scoped_name).unwrap(),
            &semantic["default"]["pose"],
        );
        pose(
            sdf.pose_of(&scoped, &actual.scoped_name).unwrap(),
            &semantic["model"]["pose"],
        );
        // Native child lookup of __root__ fails. Composition through the model
        // is tested separately; it does not claim that native lookup succeeds.
        assert_eq!(semantic["root"]["errors"], json!([26]));
        assert_eq!(semantic["absolute_root"]["errors"], json!([26]));
    }
}
#[test]
fn all_native_root_include_cases_and_explicit_subset_boundary() {
    let cases = corpus();
    assert_eq!(cases.as_array().unwrap().len(), 34);
    let mut counts = [0; 3];
    for case in cases.as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let resolver = Resolver {
            xml: case["resource"].as_str().unwrap(),
            calls: Cell::new(0),
        };
        let xml = case["xml"]
            .as_str()
            .unwrap()
            .replace("@RESOURCE@", "model://root-include");
        let result = load_str(&xml, "root-include.sdf", &resolver);
        match case["disposition"].as_str().unwrap() {
            "supported" => {
                counts[0] += 1;
                assert_eq!(case["native"]["errors"], json!([]));
                let sdf = result.unwrap_or_else(|e| panic!("{id}: {e}"));
                assert!(sdf.worlds.is_empty());
                assert_eq!(sdf.models.len(), 1);
                let actual = &sdf.models[0];
                model(&sdf, actual, &case["native"]["root_model"]);
                assert_eq!(actual.source.uri, "resolved://root-include/model.sdf");
                let include = actual.include.as_ref().unwrap();
                assert_eq!(include.uri, "model://root-include");
                assert_eq!(include.source.uri, "root-include.sdf");
                assert!(xml.contains(&include.xml));
                assert_eq!(resolver.calls.get(), 1);
            }
            "native_rejection" => {
                counts[1] += 1;
                assert!(!case["native"]["errors"].as_array().unwrap().is_empty());
                let error = result.unwrap_err();
                assert_eq!(
                    error.code,
                    if id == "root-merge-true" {
                        ErrorCode::Unsupported
                    } else {
                        ErrorCode::InvalidValue
                    }
                );
            }
            "known_mixed_root_subset_rejection" => {
                counts[2] += 1;
                // Native accepts both worlds and a model; this established Rust
                // subset remains rejected and is not counted as a parity pass.
                assert_eq!(case["native"]["errors"], json!([]));
                assert_eq!(result.unwrap_err().code, ErrorCode::InvalidValue);
            }
            _ => panic!("unclassified row {id}"),
        }
    }
    assert_eq!(counts, [28, 5, 1]);
}
#[test]
fn nonzero_child_poses_compose_through_the_root_model_edge() {
    let cases = corpus();
    for case in cases
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["id"].as_str().unwrap().starts_with("offset-link-"))
    {
        let resolver = Resolver {
            xml: case["resource"].as_str().unwrap(),
            calls: Cell::new(0),
        };
        let xml = case["xml"]
            .as_str()
            .unwrap()
            .replace("@RESOURCE@", "model://root-include");
        let sdf = load_str(&xml, "root-include.sdf", &resolver).unwrap();
        assert_eq!(sdf.pose_of("m", "::").unwrap().translation, [1., 2., 3.]);
        assert_eq!(sdf.pose_of("m::l", "m").unwrap().translation, [4., 5., 6.]);
        assert_eq!(sdf.pose_of("m::f", "m").unwrap().translation, [5., 5., 6.]);
        // These global positions are inferred by composing the independently
        // observed native model and local child poses, not native __root__ lookup.
        assert_eq!(sdf.pose_of("m::l", "::").unwrap().translation, [5., 7., 9.]);
        assert_eq!(sdf.pose_of("m::f", "::").unwrap().translation, [6., 7., 9.]);
    }
}
#[test]
fn exact_mesh_fixture_retains_submeshes_scales_and_resource_origin() {
    let resolver = Resolver {
        xml: include_str!("fixtures/root_include_test_model.sdf"),
        calls: Cell::new(0),
    };
    let sdf = load_str(
        "<sdf version='1.12'><include><uri>model://root-include</uri></include></sdf>",
        "root-include.sdf",
        &resolver,
    )
    .unwrap();
    let link = &sdf.models[0].links[0];
    assert_eq!(link.name, "link");
    assert_eq!(link.collisions[0].name, "mesh_col");
    assert_eq!(link.visuals[0].name, "mesh_vis");
    for (geometry, submesh, center, scale) in [
        (
            &link.collisions[0].geometry,
            "my_submesh",
            true,
            [0.1, 0.2, 0.3],
        ),
        (
            link.visuals[0].geometry.as_ref().unwrap(),
            "another_submesh",
            false,
            [1.2, 2.3, 3.4],
        ),
    ] {
        let Geometry::Mesh(mesh) = geometry else {
            panic!("expected mesh")
        };
        assert_eq!(mesh.uri, "meshes/mesh.dae");
        assert_eq!(mesh.submesh, submesh);
        assert_eq!(mesh.center_submesh, center);
        assert_eq!(mesh.scale, scale);
    }
}
#[test]
fn root_merge_still_resolves_and_validates_the_resource_before_context_rejection() {
    struct Missing;
    impl ResourceResolver for Missing {
        fn resolve(&self, _: &str, _: &str) -> Result<Resource, String> {
            Err("missing root resource".into())
        }
    }
    let xml = "<sdf version='1.12'><include merge='true'><uri>missing</uri></include></sdf>";
    let error = load_str(xml, "root-include.sdf", &Missing).unwrap_err();
    assert_eq!(error.code, ErrorCode::Resource);
    assert!(error.message.contains("missing root resource"));
}

#[test]
fn nonzero_root_placement_matches_native_model_pose_and_child_composition() {
    let cases = corpus();
    let mut seen = 0;
    for case in cases.as_array().unwrap().iter().filter(|case| {
        case["id"]
            .as_str()
            .unwrap()
            .starts_with("nonzero-root-placement-")
    }) {
        seen += 1;
        let resolver = Resolver {
            xml: case["resource"].as_str().unwrap(),
            calls: Cell::new(0),
        };
        let xml = case["xml"]
            .as_str()
            .unwrap()
            .replace("@RESOURCE@", "model://root-include");
        let sdf = load_str(&xml, "root-include.sdf", &resolver).unwrap();
        let actual = &sdf.models[0];
        let native = &case["native"]["root_model"];
        assert_eq!(native["pose_errors"], json!([]));
        pose(actual.pose.transform, &native["pose"]);
        pose(sdf.pose_of("m", "::").unwrap(), &native["resolved_pose"]);
        assert_eq!(actual.pose.transform.translation, [1., 2., 3.]);
        assert_eq!(sdf.pose_of("m::l", "m").unwrap().translation, [4., 5., 6.]);
        assert_eq!(sdf.pose_of("m::f", "m").unwrap().translation, [5., 5., 6.]);
        let link_placement = actual.placement_frame == "l";
        assert_eq!(
            sdf.pose_of("m", "::").unwrap().translation,
            if link_placement {
                [-3., -3., -3.]
            } else {
                [-4., -3., -3.]
            }
        );
        // Global child positions below are compositions of the observed native
        // model/local child poses; native child __root__ lookup still fails26.
        assert_eq!(
            sdf.pose_of("m::l", "::").unwrap().translation,
            if link_placement {
                [1., 2., 3.]
            } else {
                [0., 2., 3.]
            }
        );
        assert_eq!(
            sdf.pose_of("m::f", "::").unwrap().translation,
            if link_placement {
                [2., 2., 3.]
            } else {
                [1., 2., 3.]
            }
        );
    }
    assert_eq!(seen, 3);
}
