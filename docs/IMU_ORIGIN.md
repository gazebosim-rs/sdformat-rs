<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# IMU input origin

Imu::source records the caller URI and exact line/column of an explicit input
IMU declaration. Default construction and omitted IMU configuration have None.
Editing values, cloning and immutable XML emission preserve this input origin;
loading emitted XML records the new caller/declaration. The writer does not use
origin to select output, and accepted XML bytes remain unchanged.

This describes declaration provenance. It does not expose native Element pointer
identity, ownership or mutation. Native Sensor Load materializes an IMU Element
for both omitted and explicit configuration; Rust None for an omitted group only
means no explicit input declaration existed.

## Original assertions and equality

The two native IMU ToElement originals check constructor Element null after
successful emission. The proposed exact-case Rust translation asserts that no
loaded input declaration was fabricated: constructed source stays None after
immutable emission. All six Noise, raw gravity/RPY, frame/localization/orientation,
reload/edit, returned nonempty XML and quiet-output assertions remain.

Two existing Rust whole-value equalities require metadata-aware expected values
because actual origin is now represented. Prepared positive controls independently
assert literal Source oracles (imu.xml/edited.xml, line 1, column 1), then retain
complete equality against the configured expected clone with that source. Independent narrow review approved those exact replacements before edits; all
existing typed gravity/RPY assertions and complete equalities remain.

A third existing whole-value equality compares omitted and explicit Sensor
configuration. Independent narrow review approved the exact old inputs: omitted
None and explicit literal imu.sdf, line 1/column 80 are asserted independently;
complete explicit equality uses the omitted clone with only that literal source.
All subsequent default and Noise assertions remain unchanged.

## Evidence

Three new Rust tests and the folded existing equality controls cover explicit standalone, standalone Sensor and full
SDF origin, omitted groups, edit/clone/immutable emission and complete equality
with literal origin oracles. A native lifecycle probe confirms constructor-null
after emission and loaded FilePath/line retained after edit/emission, while the
omitted/explicit Sensor control records the native materialization distinction.
Both unchanged native originals pass with empty stderr. Independent narrow review
passed the four prepared controls/two original ports and exact native replay
before the two equality edits. Final independent review passed fourteen relevant Rust checks in fresh source/
target, native origin controls and both unchanged originals with empty stderr.
It independently replayed the existing twenty-four-case IMU corpus, exact fixture
and four real exchanges. Both exact constructor-only mappings are reviewed
complete. Normal strict staged/hosted qualification remains pending: candidate
186 complete /26 partial, accepted 184 complete /28 partial. Prior twelve helper
replays and 211 unchanged XML export comparisons remain author evidence.
