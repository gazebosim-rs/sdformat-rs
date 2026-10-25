// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
fn hex(value: &str) -> String {
    if value.is_empty() {
        return "-".into();
    }
    value.bytes().map(|byte| format!("{byte:02x}")).collect()
}
fn bits(value: f64) -> String {
    value.to_bits().to_string()
}
#[test]
fn native_heightmap_and_polyline_metadata() {
    let rows: serde_json::Value = serde_json::from_str(include_str!("terrain.json")).unwrap();
    for row in rows.as_array().unwrap() {
        let result = load_str(row["xml"].as_str().unwrap(), "terrain.sdf", &NoResources);
        assert_eq!(result.is_ok(), row["accepted"].as_bool().unwrap(), "{row}");
        let Ok(sdf) = result else { continue };
        let shape = &sdf.models[0].links[0].collisions[0].geometry;
        let mut actual = Vec::new();
        match shape {
            Geometry::Heightmap(h) => {
                actual.extend(["heightmap".into(), hex(&h.uri)]);
                actual.extend(h.size.into_iter().chain(h.position).map(bits));
                actual.extend([
                    u8::from(h.use_terrain_paging).to_string(),
                    h.sampling.to_string(),
                    h.textures.len().to_string(),
                ]);
                for t in &h.textures {
                    actual.extend([bits(t.size), hex(&t.diffuse), hex(&t.normal)]);
                }
                actual.push(h.blends.len().to_string());
                for b in &h.blends {
                    actual.extend([bits(b.min_height), bits(b.fade_distance)]);
                }
                assert_eq!(h.source.uri, "terrain.sdf");
            }
            Geometry::Polyline(lines) => {
                actual.extend(["polyline".into(), lines.len().to_string()]);
                for p in lines {
                    actual.extend([bits(p.height), p.points.len().to_string()]);
                    actual.extend(p.points.iter().flatten().copied().map(bits));
                    assert_eq!(p.source.uri, "terrain.sdf");
                }
            }
            _ => panic!("wrong variant"),
        }
        assert_eq!(
            actual,
            serde_json::from_value::<Vec<String>>(row["tokens"].clone()).unwrap(),
            "{row}"
        );
    }
}
#[test]
fn terrain_schema_errors_preserve_diagnostics() {
    for shape in [
        "<heightmap bad='x'/>",
        "<heightmap><unknown/></heightmap>",
        "<heightmap><uri>a</uri><uri>b</uri></heightmap>",
        "<heightmap><uri><unknown/></uri></heightmap>",
        "<heightmap><texture><size>1</size><size>2</size></texture></heightmap>",
        "<heightmap><texture><diffuse bad='x'>d</diffuse></texture></heightmap>",
        "<heightmap><blend><unknown/></blend></heightmap>",
        "<heightmap><size bad='x'/></heightmap>",
        "<polyline><height>1</height><height>2</height></polyline>",
        "<polyline><unknown/></polyline>",
        "<polyline><point bad='x'>1 2</point></polyline>",
        "<polyline/><heightmap/>",
    ] {
        let xml = format!(
            "<sdf version='1.12'><model name='m'><link name='l'><collision name='c'><geometry>{shape}</geometry></collision></link></model></sdf>"
        );
        let error = load_str(&xml, "bad-terrain.sdf", &NoResources).unwrap_err();
        assert_eq!(error.source.uri, "bad-terrain.sdf");
        assert!(matches!(
            error.code,
            ErrorCode::Unsupported | ErrorCode::InvalidValue
        ));
    }
}

#[test]
fn imported_terrain_retains_resource_identity_without_asset_lookup() {
    struct Resolver(std::cell::RefCell<Vec<(String, String)>>);
    impl ResourceResolver for Resolver {
        fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
            self.0.borrow_mut().push((uri.into(), parent.into()));
            assert_eq!(uri, "model://terrain");
            Ok(Resource {
                canonical_uri: "cache://terrain/model.sdf".into(),
                xml: "<sdf version='1.12'><model name='terrain'><link name='l'><collision name='height'><geometry><heightmap><uri>textures/h.png</uri><texture><diffuse>textures/d.png</diffuse><normal>textures/n.png</normal></texture></heightmap></geometry></collision><collision name='outline'><geometry><polyline><point>1 2</point></polyline></geometry></collision></link></model></sdf>".into(),
            })
        }
    }
    let resolver = Resolver(std::cell::RefCell::default());
    let sdf=load_str("<sdf version='1.12'><model name='outer'><include><uri>model://terrain</uri></include></model></sdf>","root.sdf",&resolver).unwrap();
    assert_eq!(
        *resolver.0.borrow(),
        vec![("model://terrain".into(), "root.sdf".into())]
    );
    let link = &sdf.models[0].models[0].links[0];
    let Geometry::Heightmap(h) = &link.collisions[0].geometry else {
        panic!("heightmap")
    };
    assert_eq!(h.source.uri, "cache://terrain/model.sdf");
    assert_eq!(h.uri, "textures/h.png");
    assert_eq!(h.textures[0].diffuse, "textures/d.png");
    assert_eq!(h.textures[0].normal, "textures/n.png");
    let Geometry::Polyline(p) = &link.collisions[1].geometry else {
        panic!("polyline")
    };
    assert_eq!(p[0].source.uri, "cache://terrain/model.sdf");
}
