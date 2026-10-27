# Model and sensor XML metadata

The source gate passes 385 Rust tests and 16 tooling tests, with unfiltered
coverage of 4941/4965 lines and 8178/8421 regions and 84/84 executable additions.
Independent source and full-evidence binding pass. Final protected metadata, the
mandatory normal hook, publication and exact hosted qualification are separate
acceptance obligations.

`Model.enable_wind` retains the model's boolean declaration. Each
`Link.properties.enable_wind` retains the link's own flag. These metadata values
do not calculate effective physical wind behavior.

`Model.extensions` owns non-schema child metadata in declaration order. Each
`ExtensionMetadata` contains the lexical tag, XML-decoded attributes in declaration
order, the exact original XML span, and the resource URI and position. Cloning a
model owns its metadata independently of the source buffer. The record has no
native mutable Element or Param operations.

An ordinary include retains its model's metadata. A model-scope merge copies
extension tags containing a colon and preserves their XML and provenance. Opaque
text such as `__model__` stays unchanged. Imported unqualified unknown children
and the imported model wind declaration are omitted during merge; the parent's
own declaration remains. Known unsupported declarations such as `self_collide`,
`allow_auto_disable`, `gripper` and `model_state` remain explicit errors. Imported
custom metadata in world-scope merge remains outside the qualified loader subset.

`Sensor.visualize` retains the canonical boolean metadata, with a false default.
The qualified sensor extension spelling `alwaysOn` is opaque XML, including
invalid boolean text, attributes, nested XML and repeated declarations. It remains
independent of the canonical `Sensor.always_on` flag. Other unqualified sensor
extension names remain unsupported. The native first-value behavior for repeated
known scalar declarations remains an explicit loader subset gap.

The exact upstream merge fixtures are retained. The PlacementFrame port
preserves the original model, link and joint counts and resolved pose predicates.
The MergeInclude port remains partial: its graph, axis, canonical-link
alias and selected metadata predicates are exercised, while the native
Model.Element/FindElement pointer API and generic HasElement absence assertions
remain unported. False DTO flags are not used as evidence of XML-element absence.
Pose and axis comparisons retain the pinned component-wise 1e-3 tolerance and
quaternion signs. The oracle's wheel angle is the exact decimal -1.5707,
represented as -15707.0/10000.0 with identical binary64 bits.

Native evidence consists of the unchanged two-original baseline, twenty Model
observations and twelve sensor Element observations, each with a separate
independent replay. The pinned existing native build, loaded libraries, source,
headers, compiler and raw streams are qualified. This is not an immutable native
rebuild. Native serialization and all error vectors remain raw observations;
these records do not qualify a Rust serializer, Element facade, warning-policy
API, error-vector equivalence or physical sensor behavior. The three nondefault
Model policy observations and the known gripper boundary are explicit limits.
No full 20/12-case Rust cross-product parity or comparative performance is claimed.
