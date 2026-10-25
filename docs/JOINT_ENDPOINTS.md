# Joint endpoint fixture assertions

Three candidate ports retain the complete original DOMJoint.LoadJointParentWorld,
LoadJointParentModelFrame and LoadJointChildModelFrame assertion bodies against
unchanged original fixtures. They exercise existing parsing and graph APIs; this
slice adds no production code, runtime joint support or constructor/setter API.

Native Root.Model/member counts, index bounds and name presence map to the owned
root collection and borrowed DOM queries. Native endpoint resolution returns body
names relative to the joint's model, or the `world` sentinel. The Rust test composes
`resolve_name` and `attached_body`, translating only global `::` to `world` and
stripping the actual model prefix. No expected body is synthesized from the joint
name or assertion value. Joint semantic poses use the stored joint frame relative
to the same model frame. Pose equality retains native component-wise <=1e-3,
including quaternion components with no sign relaxation.

Independent review approved this endpoint API translation for the exact known
valid fixtures and complete assertion bodies. It is not a claim of a separate
mutable Joint::ResolveParentLink/ResolveChildLink facade, generic error-vector
parity or behavior after public DOM edits. Other joint cases remain pending.

`tools/joint_endpoints_reference.py` reruns all three unchanged native originals,
requires exactly one actual test and no failures/errors/skips for each, and binds
original fixture/source/header, executable, linked-library and gz-math equality
header hashes. The existing SDK/build is qualified evidence, not a fresh full
native rebuild or reproducible reference image. Independent assertion review passed; the normal exact staged/hosted gates remain
required before publication.
