# Borrowed model DOM lookups

Model.model_by_name and link_by_name use explicit relative nested scopes and
return the actual stored model/link references. The model lookup follows native
Model::ModelByName; the link method exposes the already-reviewed canonical-link
lookup path. There is no semantic alias resolution or implicit descendant search.
The supported parser reserves double-colon declaration names; native legacy
fallback for such literal link names and interface-model facades are not claimed.
No IO, simulator or dependency change is involved.

The unchanged nested_model.sdf and double_pendulum.sdf fixtures support proposed
complete ports of DOMRoot.NestedModel and DOMRoot.LoadDoublePendulum. The ports
retain all existence/count/index-bound/name assertions and component-wise native
Pose equality at 1e-3, without quaternion sign relaxation. Native name-existence
wrappers map to the same borrowed Option lookup; pointer equality maps to
std::ptr::eq against the actual indexed nested model. A local test-only Clippy
allowance retains original index-zero presence assertions alongside their count
assertions. The initial lint failure is preserved in ignored evidence.

For the unchanged valid pendulum fixture only, successful typed loading already
constructs and validates both pose and attachment graphs. The proposed mapping of
its final zero-error Model::ValidateGraphs observation uses that mandatory success
contract. Independent review must explicitly approve this fixture-specific
observation or keep the mapping partial. This does not add generic mutable DOM
graph revalidation or claim error-vector/diagnostic parity.

The 36 fresh native observations compare explicit scopes, missing/unqualified
names, root/current-model prefixes, empty components and actual returned stored
objects. Native proof binds pinned Model/parser/frame source, schema/headers,
comparison headers, exact fixture bytes, linked libraries, probe and both original
binaries. The original 2018 source notice is retained. The existing SDK/build is
qualified evidence rather than a newly reproducible reference image.

Author checks: all 138 Rust tests, 13 tooling tests, fmt/clippy, 36 native rows and
both exact original cases pass. Production coverage is 3087/3107 lines and
5373/5490 regions, with 10/10 changed executable lines. No existing assertion,
coverage threshold, exclusion or dependency changes. Independent mapping review,
exact normal staged hook and hosted CI remain required; no whole packet accepted.
