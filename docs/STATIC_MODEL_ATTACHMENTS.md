# Static model attachment identities

The attachment graph preserves native `STATIC_MODEL` sinks as
`<fully scoped model>::__model__`. A static model does not attach to world; its
links remain independent link sinks. This identity is separate from pose
resolution, canonical-link DOM selection and simulator static-body behavior.

Dynamic models use the recursive canonical-link name, including a link inside a
static descendant. If that name is empty, the graph follows the first nested
model, which can eventually reach an empty static-model sink. The implementation
reuses the existing native-compatible canonical DOM selector and the existing
cycle/reference checks. Only static model vertices have no attachment edge;
that closed construction invariant identifies static sinks without introducing
another public graph type.

The 61-row native corpus covers all supported versions, root/world models,
static/dynamic chains, empty models, valid/invalid canonical names, frames on
links/models/world, canonical selection, relative poses and joint endpoints.
The native helper compiles the immutable upstream FrameSemantics.cc alongside
its own source for direct graph queries, and uses the installed native DOM for
frame/joint queries. Source, private/public headers, math graph headers, linked
libraries, fixtures and the unchanged original test binary are hash-bound.
This is not a fresh full native rebuild or a portable-image qualification.

`NestedFrameSemantics.ModelWithoutLinksWithNestedStaticModel` is ported with its
unchanged fixture and exact resolved body. The typed loader builds and validates
both graphs before returning success; the port uses that mandatory eager
validation for this known-valid fixture's graph-build/validate observations.
Independent /root/transport_fallback explicitly approved that assertion mapping. No generic
mutable graph facade or error-vector equivalence is claimed.

Four old Rust-only world attachment values are corrected against the exact
unchanged native-positive inputs. The historical `static_joint_child` row keeps
its original XML, native tokens and gap metadata, and now runs through all ordinary
pose/axis assertions after a positive load assertion. The unrelated invalid
reference, duplicate, cycle, same-body and world-child errors remain asserted.
`reference/static_model_disposition.json` records exact replacements; reversing
them reconstructs all three original static-packet parent test files byte-for-byte.
The reviewed later sensor-test edits are separately hash-bound and rewound first. Existing
canonical/model-frame JSON corpora are byte-identical. The full native 150-row /
four-original alias replay is unchanged, including its historical gap annotation.

The two new tests fail on the public parent: static roots resolve incorrectly to
world, and the unchanged nested-static original returns `::` instead of the
static model-frame sink. The retained red log is author evidence. The corrected
corpus separately checks the dynamic-parent canonical-link path.

Run `python3 tools/static_model_reference.py /path/to/pinned/sdformat` and
`cargo test --locked --test static_model`. The original static-model parser/frame
packet passed independent review and public qualification. It does not decide
simulator D23, execute joint limits, change physics static flags or adapt any
downstream sensor API.

The reference helper now accepts only the exact original post-static test-file
hashes and six later independently reviewed `tests/semantics.rs` successor
hashes. It reverses the camera→RFID→unknown-kind controls, two mimic axis
initializer changes and two axis diagnostic expectation changes, then reverses
the original static-model edits and checks the exact original parent
hashes. Any unreviewed mutation to those guarded files or listed unchanged
inputs fails before native execution. This restores
replay from both the adopted SDF575 raw export and the current raw export; each
freshly passed all 61 native observations and the unchanged original. The
helper change has no parser, native fixture or simulator behavior effect. Its
separate independent source receipt is SHA256 `b0f2104d7a4475e131971462fa0b00b93188519a59d670f93bd5831c71d83589`. Independent final binding
(receipt SHA256 `57b849447b0699de9e2b102e6394e722b70ebd8dc7084226bb725d61065061bb`), the normal protected hook and
exact hosted verify run 40236633573 passed at public `7e13dfae9ab440113f2f141e0d248cd8f3ae2d34`.
