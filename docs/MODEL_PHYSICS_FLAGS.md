# Model collision and auto-disable permission metadata

`Model.self_collide` and `Model.allow_auto_disable` are owned boolean metadata.
They correspond to the pinned native Model SelfCollide and AllowAutoDisable
getters and setters. Defaults are false and true. Loading does not configure
collision filters, sleep bodies or disable simulation. Native schema descriptions
about runtime effects do not qualify an engine implementation here. Consumers
must explicitly support or reject the new public fields before a producer pin
migration; exhaustive Model literals and destructuring need the fields.

Pinned Model.cc initializes and loads both values. Actual Root probes accept the
flags across SDF 1.4–1.12, including self_collide in 1.4 despite its absence from
that version's model schema. Omitted, empty and space/tab/LF-only elements retain
their own defaults. Words are case insensitive; true/false and 1/0 are accepted.
Numeric prefixes, other integers, non-ASCII edge whitespace, malformed words and
carriage-return character references return InvalidValue. Literal CR is normalized
by the XML layer before conversion; CR references remain literal and are invalid.
CDATA follows the same scalar rule, while markup closes the first native text
segment. The corpus preserves actual native error vectors and stderr; Rust keeps
its existing single source-located Diagnostic contract.

The new flags use a small std-based boolean text helper because the existing
legacy shared boolean path trims CR references more broadly than native Param.
That existing path and its callers remain unchanged. Duplicate flag elements are
InvalidValue; unknown attributes or nested children are Unsupported under the
loader's existing strict policy. Wrong World scope remains Unsupported. Uppercase
tags remain existing opaque Model extensions and do not populate typed flags;
native warning side effects are not reproduced.

Model flags do not overwrite Link metadata or nested Model flags. Normal includes
retain their resource's values and canonical source URI; merge includes preserve
parent flags. Owned Clone copies flags independently and keeps the existing
immutable loaded-element view. Typed edits do not update that view or Model Source.
No new Model default/constructor facade, generic Element mutation, serializer or
include override syntax is added.

The partial DOMModel.Construction mapping retains only the four default/toggle
observations at original Model_TEST.cc lines 59–65. Rust uses loaded defaults and
ordinary owned assignment for those values. Native constructor identity, null
Element, name/namespace observations, collections, poses, graph identity and all
other original assertions remain unported by this mapping. Existing namespace
observations remain supplemental. Copy/mutation checks and loaded snapshots are
supplemental; no full original-case credit is claimed.

`python3 tools/model_physics_flags_reference.py /path/to/pinned/sdformat` replays
91 controls, constructor/value-copy mutation and the unchanged complete original.
Its provenance verifies the pinned clean source, canonical SDK headers/libraries,
selected-library child environment and actual transitive linkage. Independent prior review approved only the two complete
same-input/root.sdf positives for empty self_collide and allow_auto_disable. The
original branches now use those exhaustive positives. All other predicates and
corpus bytes remain unchanged.
