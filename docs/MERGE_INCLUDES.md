# Merged includes

A model include with effective merge=true flattens its links, joints, frames,
nested models and plugins into the parent. The imported model node disappears.
A generated `_merged__NAME__model__` frame retains its placement-adjusted pose and
attaches to the imported DOM canonical-link name. Parent static/canonical settings
remain unchanged; imported model-level static settings are not copied.

Only native direct-member references are rewritten. Link/nested-model empty or
__model__ pose references use the proxy; frame attachment defaults and explicit
__model__ references, joint endpoints/poses and expressed-in axes follow the native
rules. Non-frame references inside links are left unchanged, including the native
meaning of explicit __model__ after flattening. Imported frame graphs are checked
before flattening, preventing invalid references from binding to parent members.
The original resource root pose is checked before include overrides: its
relative_to must be empty. Valid include poses may refer to parent members.
Placement-frame lookup uses schema scope rules, rejecting a full current-model
prefix that the public query API accepts.
Deferred collision/visual/sensor pose errors remain deferred; consumers still call
validate_non_frame_poses before instantiation. Arbitrary-pose query errors retain
the existing graph-root diagnostic Source; parsed member Sources remain imported.

Model.merged_includes owns ordered MergedInclude records: the original boxed
Include snapshot, imported model Source and generated proxy-frame name. Outer records precede nested expansion records. Nested
merge records retain their original resource identities. Include is clonable;
cloned snapshots survive document destruction. This is provenance metadata, not
an Element, serializer or preserve-includes printing facade. No IO is introduced.

The merge attribute follows both native stages: schema boolean validation trims
space/tab/LF, then TinyXML's original attribute spelling selects merging. Exact
case-insensitive true words and validated numeric one merge; surrounding whitespace
on true words produces ordinary nesting. Raw source XML preserves that distinction.
Ordinary includes, including explicit merge=false, retain the existing contract.

World merge supports the model-only member subset described below. Unknown/custom model extensions and enable_wind
retain the parser's explicit Unsupported policy. Legacy parent/import name
collisions also remain Unsupported until native declaration-order renaming is
qualified. The native corpus records these five accepted-input gaps separately:
three schema-extension rows and two legacy-collision rows. All existing supported
version and normal-include semantics remain in place.

The supplementary native corpus covers versions 1.5–1.12, literal attribute edges,
canonical aliases/static/empty/nested models, placement, scoped references and
axes, plugin order, nested merges and retained non-frame behavior. The exact native
MergeInclude and MergeIncludePlacementFrame originals run successfully, but their
full unmodified resource uses unsupported extension fields. No new original case
is retired by this bounded slice. Full merge/schema/export qualification remains
SDF-02 work; no simulation or performance acceptance is claimed.

## World scope

A world include with effective `merge=true` flattens the resource model's nested
models, frames, joints and plugins. It creates the same generated proxy frame,
with placement correction and attachment to the imported canonical link. The
proxy's empty pose reference uses the world frame. `World.merged_includes` keeps
the ordered original include XML/source, imported model Source and proxy name.

A direct imported link cannot become a world child and returns
`MergeIncludeUnsupported`, corresponding to native first error 33. Joint endpoints
and axes resolve in world scope; a joint's default pose frame is its child.
Imported joint sensor poses retain deferred validation. This parses semantic
metadata and does not instantiate physics constraints.

Both exact WorldMergeInclude originals preserve their fixed predicates. Sixteen
supplemental observations cover proxy/canonical attachment, placement, nonzero and
rotated poses, global joints, plugins, provenance and native errors. Projected
fixtures retain all nonmessage fields and error-code vectors/nulls; full raw
reports retain messages. Rust returns the first diagnostic atomically rather
than exposing native later error vectors or partial DOM. Native direct
LoadSdfString Element.FilePath is empty, explicitly mapped to owned caller Source;
imported canonical identities compare directly. Mutable Element, OriginalVersion,
serialization, resolver/model.config/search-path equivalence and full SDF-02
qualification remain outside this increment.
