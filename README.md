# SDFormat in Rust

Independent SDF syntax, semantic frame graphs, and typed world/model support.
This is an early implementation of SDF-01, not a complete SDFormat replacement.

```rust
use sdformat::{load_str, NoResources};
let sdf = load_str("<sdf version='1.12'><model name='robot'><link name='base'/></model></sdf>", "robot.sdf", &NoResources)?;
let pose = sdf.pose_of("robot::base", "::")?;
# Ok::<(), sdformat::Diagnostic>(())
```

The supported subset has SDF 1.6–1.12 worlds, nested and included models,
links, primitive and mesh collision/visual metadata and poses, physical mass/inertia,
fixed/revolute/prismatic/continuous/universal/revolute2 joints, typed IMU sensor configuration, frames, and opaque plugin content.
Model and include placement frames resolve through nested links, frames, joints,
and models. Pose and attachment graphs are separate. Loading is atomic; non-frame
pose errors remain deferred until resolution, matching the native DOM. Scene
builders can call `validate_non_frame_poses` before constructing objects. Unsupported elements,
attributes, versions, merge features, DTD declarations and automatic inertia produce
explicit diagnostics. IMU noise, orientation settings and original XML are retained together. See
[the contract](docs/CONTRACT.md).

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
