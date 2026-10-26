# SDFormat in Rust

Independent SDF syntax, semantic frame graphs, and typed world/model support.
This is an early implementation of SDF-01, not a complete SDFormat replacement.

```rust
use sdformat::{load_str, NoResources};
let sdf = load_str("<sdf version='1.12'><model name='robot'><link name='base'/></model></sdf>", "robot.sdf", &NoResources)?;
let pose = sdf.pose_of("robot::base", "::")?;
# Ok::<(), sdformat::Diagnostic>(())
```

The supported subset has SDF 1.4–1.12 worlds with owned physics profile and atmosphere metadata,
nested and included models,
links with owned wind/gravity/kinematic/self-collision flags, primitive and mesh
collision/visual metadata and poses, owned visual/material
metadata, physical mass/inertia,
fixed/revolute/prismatic/continuous/universal/revolute2 joints with owned axis
dynamics and stop metadata, typed IMU/contact/force-torque/magnetometer/altimeter/air-pressure/NavSat/air-speed/Lidar sensor configuration, frames, and opaque plugin content.
Borrowed model/link/joint/frame DOM lookups follow explicit relative nested scopes.
Model and include placement frames resolve through nested links, frames, joints,
and models. Pose and attachment graphs are separate. Loading is atomic; non-frame
pose errors remain deferred until resolution, matching the native DOM. Scene
builders can call `validate_non_frame_poses` before constructing objects. Unsupported elements,
attributes, versions, merge features, DTD declarations and automatic inertia produce
explicit diagnostics. IMU noise, orientation settings and original XML are retained together. SensorConfig
selects boxed IMU/ForceTorque/Magnetometer/Altimeter/AirPressure/NavSat/AirSpeed/Lidar/Camera or owned Contact settings; optional accessors
expose the chosen configuration without dummy values. Contact collision names
and nested topic are metadata, with no contact runtime effect. Joint sensors retain
their joint-default pose frame and exact raw configuration; force/torque fields
remain metadata with no wrench runtime. Magnetometer preserves three noise axes
without sampling a magnetic field; Altimeter preserves vertical position and
velocity noise without computing measurements. AirPressure retains pressure noise
and reference altitude without sampling pressure. NavSat retains four position
and velocity noise records for both `navsat` and legacy `gps` spellings without
computing coordinates. AirSpeed retains differential pressure noise without
measuring speed. Lidar retains scan, range, noise and visibility settings for
ray/lidar and GPU spellings without simulating rays. An owned Camera value
object retains construction defaults and setters; a typed parser retains the
pinned camera sensor fixture fragment. Camera rendering remains pending. See [ForceTorque](docs/FORCE_TORQUE.md) and
[the contract](docs/CONTRACT.md).
The same owned Camera configuration also parses the bounded native depth and
RGBD fixture fragments; depth image processing remains outside this library.
Segmentation and bounding box camera fragments retain their declared output
type and presence flags without implementing rendering.
Thermal and wide angle camera fixture fragments retain the native Camera values,
including the wide angle lens and sensor topic/rate, without rendering.
The sensor's literal XML type remains available as `kind`; `effective_type()`
provides the native classification for parsed sensor kinds, including legacy
GPS and GPU/CPU lidar aliases. Seven additional native-known fixture types
retain their envelope with `SensorConfig::Unconfigured`; family payloads
and runtime behavior remain pending. A bounded custom-sensor extension retains
one vendor element name, its direct text parameters and exact raw subtree;
full mutable Element and extension grammar remain pending.

Scalar and pose-component conversion is qualified for Linux GNU/glibc using an
owned C locale and native range/prefix semantics. One reviewed FFI module is the
only production unsafe exception; other platforms return Unsupported for these
explicit conversions pending qualification. No global locale is changed. See
[the conversion boundary](docs/SCALAR_CONVERSION.md).

Resources are injected through `ResourceResolver`; the library performs no network
or filesystem IO. Plain f64 values cross the API; no Bevy, simulator, Fuel, or
transport dependency is present. See [evidence and remaining work](docs/STATUS.md).

Run `sh tools/bootstrap.sh` to install Rust 1.94.1 and its matching LLVM tools,
fetch locked dependencies and install the checked-in pre-commit hook. Run
`python3 tools/verify.py` for the required core profile or
`python3 tools/verify.py --staged` for an isolated export of the Git index.
Coverage must strictly exceed 95% of both production lines and regions, including
changed executable lines. The gate rejects missing instrumentation and files;
protected assertion/policy changes require a separate review receipt. CI runs the
same gate and publishes artifacts. Repository administrators must configure the
`verify / core` status and independent reviews as required branch checks.
The covered layout is one library in `src`, integration tests in `tests`, no
package features, and the Linux x86_64 core runner. Additional Cargo targets
(including automatically discovered build scripts), local packages or features
require a reviewed gate extension before they can compile in verification.
Rust source files are checked against those roots; the runtime coverage report
also rejects other owned instrumented inputs, regardless of their file extension.
Staging verification freezes the Git tree and exports it through a private index.
Every exported blob, file type, owner executable bit and path is checked against
that tree before compilation. Export-ignore cannot omit files; checkout transforms
such as EOL conversion are rejected. Index or HEAD changes invalidate the check.

The upstream pin, runtime inventory, fixture hashes and port mapping are in
[reference/](reference/). Most upstream cases remain pending; passing the small
current suite does not establish full parity. The canonical
[plan](https://github.com/gazebosim-rs/gz-sim-rs/blob/main/docs/PLAN.md),
[contract](https://github.com/gazebosim-rs/gz-sim-rs/blob/main/docs/EXTERNAL_REPOS.md#sdformat-rs),
and [test policy](https://github.com/gazebosim-rs/gz-sim-rs/blob/main/docs/TESTING.md)
remain the acceptance requirements.

Apache-2.0. Original fixtures/test assertions retain OSRF provenance; see LICENSE.

Copyright (C) 2026 Open Source Robotics Foundation. Owned code uses Apache-2.0;
see [LICENSE](LICENSE) and [NOTICE](NOTICE). Original imported notices and years
remain intact, and third-party dependencies retain their own licenses.


World atmosphere defaults and explicit Adiabatic temperature/gradient/pressure
values are owned metadata; see [atmosphere scope and limits](docs/ATMOSPHERE.md).


Owned `Geometry` and `Mesh` values support cloning, including nested metadata and
source locations, without loading resources. See the [ownership contract](docs/CONTRACT.md#owned-geometry-cloning)
for the fixed Box observations and remaining native Shape/runtime limits.

Owned `Visual` metadata supports independent cloning of its pose, geometry, material,
plugins and provenance. Five fixed native copy/move observations are retained;
rendering and mutable Element identity remain outside this value contract.

Owned `Collision` metadata supports independent cloning of pose, geometry,
surface and source provenance. Five fixed native pose ownership observations
are retained; cloning performs no resource loading or physical execution.

Owned `Link` and `Inertial` metadata support independent cloning, including nested
geometry, visual, sensor and provenance data. Raw scoped references are retained;
cloning does not rebuild frame graphs or perform resource or physics work.

Owned `Model` and `Frame` values support cloning. Model `raw_namespace` retains
optional metadata; `namespace()` replaces literal `{name}` placeholders using
the current name. Model `namespace` attributes and nonempty include `<namespace>`
overrides are parsed as metadata. Empty XML attributes are absent; empty include
overrides keep the resource's value. This does not change frame scopes or topics.


A standalone model can also come from a root-level nonmerged `<include>` through
an injected resource resolver. Namespace, name, static, pose, placement and plugin
overrides retain imported source and raw include XML. Literal root-model pose
references remain empty-only. Mixed world/model roots remain an explicit Rust
subset rejection, even though the native loader accepts that combination.
