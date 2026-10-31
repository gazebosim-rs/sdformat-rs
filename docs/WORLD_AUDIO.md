<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Audio-device metadata

`World.audio_device` owns the `<audio><device>` string. Missing audio or device
uses `default`. Empty or whitespace-only required device text returns Missing.
The existing string conversion preserves native ordinary-text collapse, CDATA
trimming, entities and UTF-8; mixed text uses the native first text node.
Unknown children/attributes return Unsupported. Duplicate audio/device elements
retain the existing InvalidValue policy; native accepts the first occurrence.

Loading stores metadata only. It opens no device, allocates no audio engine and
starts no playback or worker. There is no audio API, serializer or simulator
activation. Consumers must explicitly support or reject the field before adopting
this producer revision. Direct World struct literals gain a required field;
this slice does not add a World constructor or claim full native construction
parity.

The schema block is unchanged across supported SDF 1.4–1.12. A native helper
checks loaded/default/string/error observations against pinned SDFormat
1249805d9fceeff94ca1fe63e26ab9c051cea24e. Its replay checks installed headers,
selected libraries and sources before/after execution. The corpus preserves
native duplicate behavior separately from Rust's explicit strict policy.

The original `DOMWorld.Load` audio assertion (`AudioDevice() == /dev/audio`)
uses an unchanged complete audio subtree from `world_complete.sdf`, retaining
the SDF/world ancestry and world name. Full-document and projected native getter
outputs agree; the unchanged original C++ case also passes. The existing vector
and Scene fixtures/assertions are untouched. The original case remains partial:
spherical coordinates, GUI, remaining physics/frame and native Element/path
contracts are not covered by this addition. No extra complete-case credit.

Author checks pass four focused Rust tests, 31 native controls, constructor and
full/projected native equality, plus unchanged DOMWorld.Load. Independent review approved the four exact known-audio capability transitions before
their existing predicates changed. The old corpus bytes and caller URIs remain
unchanged; full metadata positives replace only those four temporary rejections.
Final independent fold review passed seven affected Rust tests, native replay and
all-target Clippy. The protected staged gate and exact hosted checks remain pending.
