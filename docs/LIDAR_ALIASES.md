<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Standalone Lidar aliases

`load_lidar_str` accepts `lidar`, `ray`, `gpu_lidar` and `gpu_ray` roots. All use
the existing owned `Lidar` scan/range/Noise/visibility parser and caller URI/line/
column provenance. No field, dependency, resource access or runtime is added.
The value has no CPU/GPU kind; `to_xml` emits canonical `<lidar>`. A document
Sensor still retains its literal kind independently. This addition does not
admit GPU-named configuration children inside document Sensors.

The existing required scan/horizontal/range groups and strict unknown attribute,
child and duplicate checks remain. Omitted horizontal/vertical samples are
640/1, scan resolutions are 1, angles/ranges are zero, visibility is UINT32_MAX,
and absent Noise is None. Present empty Noise selects Gaussian. The existing
scalar/u32 conversion and negative-standard-deviation checks remain. Scan ordering,
inverted ranges, signed zero, zero samples and resolution are metadata, not ray
allocation or measurement policies. Advanced Noise remains Unsupported.

## Native and retained assertion scope

Pinned `Lidar.cc` accepts all four root names before the same metadata load.
The native helper initializes the pinned current Lidar schema under the requested
root and reads a versioned SDF wrapper; it does not change the input tag or values.
Thirty-six admitted cases cover all four roots, complete configured metadata,
signed zeros, default/quantized Noise and wrappers 1.4, 1.5, 1.6, 1.8 and 1.12.
Standalone Rust fragments have no version field, so wrapper conversion and native
OriginalVersion/Element behavior are characterization only.

Twenty-eight additional native controls record missing groups, unknown fields,
advanced Noise, duplicate samples and negative standard deviation. Rust keeps its
explicit stricter rejections even where native warns, fills defaults or ignores
data. Source-located errors, case-sensitive roots and unchanged document payload
boundaries are tested separately. All prior Lidar tests and corpus remain intact.

`DOMLidar.Load` (Lidar_TEST.cc:103–124) remains unported as a complete case.
The new wrong-root test is supplemental and adds no original-case credit. It
maps a single typed failure from `<bad/>` to this value reader's existing Xml
diagnostic. It does not claim the literal native ELEMENT_INCORRECT_TYPE enum,
null Element input, a retained mutable pointer or mutation after failed loading.
Native null/bad controls and all four unchanged Lidar originals pass separately.
Constructor/Set/ToElement mappings retain their existing complete scope.

```
python3 tools/lidar_aliases_reference.py /path/to/pinned/sdformat
cargo test --locked --test lidar_aliases --test lidar_original --test lidar_xml --test lidar_xml_original
```

Native source, installed schema/headers, math headers, compiler and loaded library
identities are recorded in `reference/lidar-aliases.json`. Child library selection
is explicit; preload/audit and SDF_PATH overrides are cleared. Foreign Git checks
use the existing isolated context helper. This packet records the current system
urdfdom library already used by accepted 486's native evidence; it does not rewrite
the historical Lidar XML provenance.

Author focused checks pass 11 Rust tests and 64 native controls, the two original
Load controls and all four unchanged originals. Independent review and ordinary
strict staged/hosted qualification remain pending. No full LiDAR, Element,
conversion or simulator-consumer acceptance is claimed.
