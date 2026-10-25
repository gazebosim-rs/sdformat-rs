# Scoped joint and frame DOM lookup

Model.joint_by_name and frame_by_name return actual stored member references,
using explicit relative model scopes. Frame lookup includes declared Frame
objects only; implicit link/joint/model graph vertices are not Frame objects.
No semantic aliases, global-root normalization or implicit descendant search is
introduced. Shared member-scope selection also serves link_by_name, preserving
all prior model/link native observations and canonical selection assertions.

The native getters fall back to an exact direct name if the model prefix is
missing. This matters for owned DOM fields assigned literal-colon names in
memory, even though the parser reserves such declaration names. If a prefix
resolves, its missing member does not fall back to a direct literal name. New
native controls reproduce both rules through native SetName and corresponding
Rust field assignment. These lookups do not rebuild semantic graphs after edits;
mutable graph revalidation/interface models remain outside this contract.

The unchanged model_multi_nested_model.sdf supports a proposed complete
DOMRoot.MultiNestedModel port. Every original count, presence, index bound, name,
explicit nested lookup and pointer identity is retained; pointer identity checks
use std::ptr::eq against actual indexed stored objects. Native NameExists wrappers
map to the corresponding Option result. The seven graph membership observations
use attachment lookup at the exact scoped names on the unchanged valid graph,
including UnknownReference for the unqualified missing middle link. Independent
review must explicitly approve this known-fixture membership mapping or keep the
case partial; no generic mutable graph or error-vector parity is claimed.

The 104 fresh native rows retain all 36 prior model/link cases unchanged and add
nested joint/frame/link identity, kind separation, implicit-frame absence,
missing prefixes, empty names and literal-name fallback/precedence controls.
Source/header/schema/fixture, comparison headers, linked library/probe and original
binary hashes are bound. The original 2018 assertion notice and fixture bytes are
retained. Existing SDK/build qualification is not a reproducible reference-image
claim. Initial new-test compilation used the wrong enum spelling UnknownFrame;
that log is retained, with the actual UnknownReference contract asserted.

Author checks pass all 140 Rust tests, 13 tooling tests, fmt/clippy, 104 native
rows and the exact original (one run/zero failures). Coverage is 3099/3119 lines,
5398/5515 regions and 16/16 changed executable lines. There are no dependency,
unsafe, old assertion, exclusion or gate changes. Independent mapping review and
normal staged/hosted gates remain mandatory; no whole packet is accepted here.
