# Owned world physics profiles

World.physics retains declaration order. World::physics_default returns the first
profile marked default, otherwise the first profile; an empty edited collection
returns None. No engine is selected or configured by the parser. Profiles own
name, default flag, engine type string, maximum step size, real-time factor,
real-time update rate, signed maximum contacts, source and exact original XML.
The update-rate field is schema metadata: native Physics has no typed getter for
it, so the differential probe reads its Element parameter explicitly.

An omitted physics declaration produces the native effective singleton profile:
default_physics, false, ode, 0.001, 1, 1000 and 20. Its source/XML are None to mark
that no declaration existed; this is not a general native Element-pointer API.
Physics::default instead models a constructed native Physics with an empty name
and no XML. Explicit profiles have Some source/XML, including empty-name attributes
that take the schema default. Cloned profiles remain valid after the Sdf is dropped.

Explicit type is required, but its nonempty value is unconstrained, matching native
DOM behavior. Profile names trim only native space/tab/LF; empty names become the
schema default and reserved/duplicate names reject. The default boolean follows
native parameter spelling, and the first true profile wins ties. Scalar values
preserve native numeric prefixes/range errors and may be negative or zero; runtime
consumers must validate any value they actually use as a timestep or solver limit.
The existing finite-double policy still rejects native nonfinite metadata, recorded
as explicit gaps. Optional max_contacts uses native signed stoi prefix/i32 range
behavior, including the leading-0x rule and boolean aliases. No new FFI is added.

Engine-specific ODE/Bullet/DART/Simbody blocks, custom extensions, Element/setter
serialization and full schema/conversion behavior remain Unsupported or pending.
Preserving these core fields does not imply support for their runtime effect. The
consumer must decide whether an explicit timestep/profile is used, overridden or
unsupported and must preserve the chosen policy in reset state. SDF 1.4–1.12 core
profiles are checked; earlier versions remain outside the current loader floor.

The native proof contains 89 profile/default/error observations and the exact
DOMPhysics.DefaultConstruction original (one run, zero failures). The Rust port
retains every original empty/set-name, default/set flag/engine/step/factor assertion;
absent original Element maps to absent owned XML on the constructed value. Public
field assignments are the Rust value API for those setter observations. Original
2018 notices remain. Full assertion/scope review, exact staged and hosted gates
are required before the original mapping is accepted.


## Bounded XML output

`Physics::to_xml()` emits one SDF 1.12 physics profile for use in a world. The
existing world loader reads the same owned fields. Text must survive native
attribute trimming and XML normalization; empty, reserved or scoped profile
names reject. Engine type remains metadata. Scalars must be finite and not
subnormal; finite negative metadata, signed zero and signed i32 contacts remain
supported. Consumers still validate runtime timesteps and solver settings.

The fixed six value observations in the two native ToElement originals map to
this roundtrip. The update-rate field and boundary checks are supplemental.
Successful nonempty XML plus full typed reload translates the returned Element
presence assertion. The ErrorOutput port checks actual empty library stdout/stderr
in a child process and successful emit/load Results. See
[XML_ASSERTION_MAPPINGS.md](XML_ASSERTION_MAPPINGS.md). Full schema, native XML
byte identity, engine execution and general error-vector parity remain unqualified.
