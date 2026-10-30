// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
#[path = "support/geometry_empty.rs"]
mod empty_support;
use serde_json::Value;
#[path = "support/heightmap_projection.rs"]
mod heightmap_projection;
#[path = "support/mesh_projection.rs"]
mod mesh_projection;
#[path = "support/polyline_projection.rs"]
mod polyline_projection;
#[path = "support/geometry_projection.rs"]
mod projection;
fn data() -> Value {
    serde_json::from_str(include_str!("geometry_xml.json")).unwrap()
}
#[test]
fn native_primitive_inputs_and_current_value_outputs() {
    let data = data();
    let rows = data["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 48);
    let export = std::env::var_os("SDF_GEOMETRY_XML_EXPORT").map(std::path::PathBuf::from);
    if let Some(path) = &export {
        std::fs::create_dir_all(path).unwrap();
    }
    for row in rows {
        let result = load_geometry_str(row["xml"].as_str().unwrap(), "geometry.xml");
        if let Some(code) = row.get("gap_code") {
            let code = match code.as_str().unwrap() {
                "Unsupported" => ErrorCode::Unsupported,
                "InvalidValue" => ErrorCode::InvalidValue,
                _ => panic!("unknown boundary"),
            };
            assert_eq!(result.expect_err(row["id"].as_str().unwrap()).code, code);
        } else if row["export"] == true {
            let value = result.unwrap_or_else(|e| panic!("{}:{e:?}", row["id"]));
            let lines: Vec<_> = row["native"]["stdout"].as_str().unwrap().lines().collect();
            assert_eq!(lines.len(), 3);
            assert_eq!(&lines[..2], ["READ 1", "LOAD"]);
            let expected: Vec<_> = lines[2].split_whitespace().map(str::to_owned).collect();
            assert_eq!(projection::projection(&value), expected, "{}", row["id"]);
            if matches!(value, Geometry::Mesh(_)) {
                let d: Value = serde_json::from_str(include_str!("mesh_xml.json")).unwrap();
                let row = &d["geometry_controls"][0];
                assert_eq!(
                    row["xml"],
                    rows.iter().find(|r| r["id"] == "mesh").unwrap()["xml"]
                );
                mesh_projection::assert_geometry(
                    row["xml"].as_str().unwrap(),
                    "geometry.xml",
                    row["native"]["stdout"].as_str().unwrap(),
                );
            }
            if matches!(value, Geometry::Heightmap(_)) {
                let d: Value = serde_json::from_str(include_str!("heightmap_xml.json")).unwrap();
                let row = &d["geometry_controls"][0];
                assert_eq!(
                    row["xml"],
                    rows.iter().find(|r| r["id"] == "heightmap").unwrap()["xml"]
                );
                heightmap_projection::assert_geometry(
                    row["xml"].as_str().unwrap(),
                    "geometry.xml",
                    row["native"]["stdout"].as_str().unwrap(),
                );
            }
            if matches!(value, Geometry::Polyline(_)) {
                let d: Value = serde_json::from_str(include_str!("polyline_xml.json")).unwrap();
                let row = &d["geometry_controls"][0];
                assert_eq!(
                    row["xml"],
                    rows.iter().find(|r| r["id"] == "polyline").unwrap()["xml"]
                );
                polyline_projection::assert_geometry(
                    row["xml"].as_str().unwrap(),
                    "geometry.xml",
                    row["native"]["stdout"].as_str().unwrap(),
                );
            }
            let xml = value.to_xml().unwrap();
            assert_eq!(
                projection::projection(&load_geometry_str(&xml, "roundtrip.xml").unwrap()),
                expected,
                "{} roundtrip",
                row["id"]
            );
            if let Some(path) = &export {
                std::fs::write(
                    path.join(format!("{}.xml", row["id"].as_str().unwrap())),
                    xml,
                )
                .unwrap();
            }
        } else {
            assert!(result.is_err(), "{}", row["id"]);
        }
    }
}
#[test]
fn edited_plane_normals_are_emitted_raw_and_reload_with_existing_policy() {
    let data = data();
    let profiles: Vec<Vec<String>> = data["plane_profiles"]["stdout"]
        .as_str()
        .unwrap()
        .lines()
        .map(|line| line.split_whitespace().map(str::to_owned).collect())
        .collect();
    assert_eq!(profiles.len(), 12);
    for (index, normal) in [
        [0., 0., 0.],
        [1e-7, 0., 0.],
        [1e-6, 0., 0.],
        [1e-5, 0., 0.],
        [1., 2., 3.],
        [1e300, 1e300, 0.],
    ]
    .into_iter()
    .enumerate()
    {
        let value = Geometry::Plane {
            normal,
            size: [1., 1.],
        };
        let before = value.clone();
        let xml = value.to_xml().unwrap();
        assert_eq!(value, before);
        if let Some(path) = std::env::var_os("SDF_GEOMETRY_XML_EXPORT") {
            std::fs::create_dir_all(&path).unwrap();
            std::fs::write(
                std::path::PathBuf::from(path).join(format!("edited-plane-{index}.xml")),
                &xml,
            )
            .unwrap();
        }
        let loaded = load_geometry_str(&xml, "plane.xml").unwrap();
        assert_eq!(projection::projection(&loaded), profiles[2 * index]);
        assert_eq!(profiles[2 * index], profiles[2 * index + 1]);
        if let Geometry::Plane { normal: loaded, .. } = loaded {
            if index <= 2 {
                assert_eq!(loaded.map(f64::to_bits), normal.map(f64::to_bits));
            }
        } else {
            panic!("not plane");
        }
    }
}
#[test]
fn output_rejects_dimensions_or_types_it_cannot_preserve() {
    for bad in [0., -0., -1., f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for value in [
            Geometry::Box {
                size: [bad, 1., 1.],
            },
            Geometry::Ellipsoid {
                radii: [1., bad, 1.],
            },
            Geometry::Plane {
                normal: [0., 0., 1.],
                size: [1., bad],
            },
            Geometry::Sphere { radius: bad },
            Geometry::Cylinder {
                radius: bad,
                length: 1.,
            },
            Geometry::Capsule {
                radius: 1.,
                length: bad,
            },
            Geometry::Cone {
                radius: bad,
                length: 1.,
            },
        ] {
            assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
        }
    }
    for value in [
        Geometry::Sphere {
            radius: f64::from_bits(1),
        },
        Geometry::Cylinder {
            radius: 1.,
            length: f64::from_bits(1),
        },
        Geometry::Capsule {
            radius: f64::from_bits(1),
            length: 1.,
        },
        Geometry::Cone {
            radius: 1.,
            length: f64::from_bits(1),
        },
    ] {
        assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    }
    for coordinate in 0..3 {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut normal = [0., 0., 1.];
            normal[coordinate] = bad;
            assert_eq!(
                Geometry::Plane {
                    normal,
                    size: [1., 1.]
                }
                .to_xml()
                .unwrap_err()
                .code,
                ErrorCode::InvalidValue
            );
        }
    }
    for shape in [
        "<mesh><uri>mesh.dae</uri></mesh>",
        "<heightmap><uri>height.png</uri></heightmap>",
        "<polyline><height>1</height><point>0 0</point></polyline>",
    ] {
        let xml = format!(
            "<sdf version='1.12'><model name='m'><link name='l'><collision name='c'><geometry>{shape}</geometry></collision></link></model></sdf>"
        );
        let sdf = load_str(&xml, "complex.xml", &NoResources).unwrap();
        if matches!(
            sdf.models[0].links[0].collisions[0].geometry,
            Geometry::Mesh(_)
        ) {
            let d: Value = serde_json::from_str(include_str!("mesh_xml.json")).unwrap();
            let row = &d["geometry_controls"][1];
            assert_eq!(row["xml"], xml);
            mesh_projection::assert_geometry(
                &xml,
                "complex.xml",
                row["native"]["stdout"].as_str().unwrap(),
            );
        } else if matches!(
            sdf.models[0].links[0].collisions[0].geometry,
            Geometry::Heightmap(_)
        ) {
            let d: Value = serde_json::from_str(include_str!("heightmap_xml.json")).unwrap();
            let row = &d["geometry_controls"][1];
            assert_eq!(row["xml"], xml);
            heightmap_projection::assert_geometry(
                &xml,
                "complex.xml",
                row["native"]["stdout"].as_str().unwrap(),
            );
        } else if matches!(
            sdf.models[0].links[0].collisions[0].geometry,
            Geometry::Polyline(_)
        ) {
            let d: Value = serde_json::from_str(include_str!("polyline_xml.json")).unwrap();
            let row = &d["geometry_controls"][1];
            assert_eq!(row["xml"], xml);
            polyline_projection::assert_geometry(
                &xml,
                "complex.xml",
                row["native"]["stdout"].as_str().unwrap(),
            );
        } else {
            panic!("unexpected geometry fixture type");
        }
    }
}
#[test]
fn standalone_error_locations_and_strict_xml_boundaries() {
    assert!(load_geometry_str("<", "bad.xml").is_err());
    assert_eq!(
        load_geometry_str("<surface/>", "bad.xml").unwrap_err().code,
        ErrorCode::Xml
    );
    let d: Value = serde_json::from_str(include_str!("mesh_xml.json")).unwrap();
    let row = &d["geometry_controls"][2];
    let xml = "<geometry><mesh/></geometry>";
    assert_eq!(row["xml"], xml);
    mesh_projection::assert_geometry(xml, "caller.xml", row["native"]["stdout"].as_str().unwrap());
    let Geometry::Mesh(value) = load_geometry_str(xml, "caller.xml").unwrap() else {
        panic!("not Mesh")
    };
    assert_eq!(
        value.source,
        Source {
            uri: "caller.xml".into(),
            line: 1,
            column: 11
        }
    );
    let (xml, code, column) = (
        "<geometry><box bad='x'/></geometry>",
        ErrorCode::Unsupported,
        11,
    );
    let error = load_geometry_str(xml, "caller.xml").unwrap_err();
    assert_eq!(error.code, code);
    assert_eq!(
        error.source,
        Source {
            uri: "caller.xml".into(),
            line: 1,
            column
        }
    );
    empty_support::geometry("<geometry/>", "caller.xml");
}
