# Nested implicit model-frame references

SDF references such as `nested::__model__` and
`nested::deeper::__model__` identify the named model's implicit frame. The graph
stores that frame under the scoped model name, so reference resolution maps a
terminal alias to the actual Model vertex. Pose, attachment, joint endpoints,
axis frames, canonical-link graph references and placement frames use the same
mapping. Raw DOM strings are preserved.

Only an actual model qualifies. A link, declared frame, joint or missing prefix
cannot supply an implicit model frame. The alias is terminal: it cannot be an
intermediate scope component or be repeated. This does not enable global/current
model prefixes in schema references that previously rejected them. The existing
convenience query resolve_name retains its separate fully scoped lookup policy;
pose_of still consumes actual fully scoped graph vertex names. Use resolve_name
when translating an SDF reference into one of those graph names.

The correction was found by faithfully porting unchanged
DOMJoint.LoadJointNestedParentChild: its M1::__model__ endpoints loaded natively
but failed in Rust at J6. Both that exact fixture and a smaller native-backed
frame-pose case failed before the correction and pass afterward. No old negative
assertion is removed or changed.

The native 150-row corpus covers nine supported versions, direct/deeper models,
world/static models, pose and attachment references, joint parent/child/pose/axis,
canonical and placement references, and deferred collision pose errors. One
inherited endpoint validation difference is explicit: native accepts a static-model child sink with its model-frame body identity,
while existing Rust static attachment normalization maps it to world and its
endpoint validation rejects it. That policy is not relaxed here; runtime static
and joint admissibility remain consumer concerns. Native diagnostic text/counts,
partial DOM on failure and general mutable graph facades are unclaimed.

Four complete original cases are candidate mappings:
LoadJointParentFrame, LoadJointChildFrame, LoadJointPoseRelativeTo and
LoadJointNestedParentChild. Every original presence/count/index/name/raw pose,
reference/body and semantic pose assertion is retained. Stored collections and
borrowed lookups translate native DOM access. Existing resolve_name+attached_body
resolves actual bodies; only the model scope is stripped from returned names,
with the global world sentinel translated. Pose3 comparisons retain native
component-wise <= 1e-3 for both position and quaternion, without accepting the
opposite quaternion sign. Supplementary native comparisons use 1e-12 bounds.
Independent mapping/source review approved these assertions; current-parent overlap
and mandatory staged/hosted qualification remain required.

```sh
python3 tools/joint_frames_reference.py ../sdformat
cargo test --locked --test joint_frames
```

The runner freshly compiles the alias probe, verifies every recorded observation,
checks immutable fixtures, source/header/library/math identities, and executes each
unchanged native original once with no failures or skips. Existing native binaries
and the installed SDK are qualified inputs, not a new full native rebuild or
reproducible-image/performance claim. No dependency, unsafe boundary, simulator or
joint runtime is added.
