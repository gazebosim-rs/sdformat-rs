<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# ToElement assertion mappings

This audit translates seven fixed native tests to the approved owned Rust APIs:
Frame.ToElement and ToElementErrorOutput, Physics.ToElement and
ToElementErrorOutput, ForceTorque.ToElement and ToElementErrorOutput, and
Camera.ToElement. Existing test function names stay unchanged for traceability.
No production code, fixture, original assertion or tolerance is removed.

For these cases, a non-null returned Element maps to successful, nonempty XML
followed by successful typed loading and every original field assertion. Empty
native emit/load error lists map to successful Results at those calls. Frame
and Physics originals do not inspect input Element identity. Their ErrorOutput
ports now execute in child processes: stderr must be empty and stdout must
contain only libtest's exact successful protocol, numeric filter count and finite
nonnegative duration. Both Frame attachment branches run inside that check.

Frame and Light ports retain their prior dot-product checks and additionally
compare every quaternion component directly within 1e-3, preserving native sign
and Pose3 equality semantics. Light remains partial for its distinct constructed
and loaded Element observations and uncaptured console assertion.

The coordinator explicitly approved a narrower mapping for the two ForceTorque
cases and Camera case: after successful immutable emission of the constructed
value, source.is_none() observes that no loaded input declaration was fabricated.
This translates only their constructor Element()==nullptr-after-ToElement check.
It is not a general equivalence between absent Source and null native Element;
loaded default groups, pointer identity, ownership and mutation are unqualified.
All existing Noise, camera values, positive finite four-ULP comparisons, edited
output and load assertions remain. The existing force/torque child-console check
also remains unchanged.

IMU stays partial for src/Imu_TEST.cc:158,228: the constructor input-Element
absence has no corresponding owned input-origin observation. Light stays partial
for constructed-null and loaded-nonnull observations at
src/Light_TEST.cc:372,378,444,451, and the empty console assertion at 470.
Joint stays partial for src/Joint_TEST.cc:289-293 type-none sensor Add, duplicate
rejection and Clear, and the emitted sensor lookup/name assertions at 316-319;
its constructor input absence at 298 is also unqualified. This audit makes no new runtime or full-grammar claim.

Author evidence: 15 relevant Rust tests and nine unchanged native originals
(Frame2, Physics2, ForceTorque2, Camera1, Light2) pass with empty native stderr.
Native replay output is in ignored artifacts/xml-assertion-reference/result.json;
upstream source/library provenance and XML exchange proofs remain in the existing
family references. Candidate ledger is 176 complete and 28 partial; accepted
baseline remains 169 complete and 35 partial until the normal
strict staged hook and exact hosted CI pass. Independent review approved all
seven exact mappings after 15 tests from fresh separate source/target, nine
unchanged native originals and two private adversarial controls proving stdout
and stderr noise are rejected by the shared quiet helper. No production source changed.

Reproduce native originals against the pinned SDK (each target uses
`--gtest_filter=DOM<Family>.ToElement:DOM<Family>.ToElementErrorOutput`; Camera
uses only DOMCamera.ToElement): build/bin/UNIT_Frame_TEST, UNIT_Physics_TEST,
UNIT_ForceTorque_TEST, UNIT_Camera_TEST and UNIT_Light_TEST. Run Rust with:

```
cargo test --locked --test frame_roundtrip_originals --test physics_xml_originals --test light_roundtrip_originals --test force_torque_xml_original --test camera_base_xml_original
```

Author full suite passes all 441 Rust tests; formatting and Clippy pass.
