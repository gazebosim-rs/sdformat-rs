<!--
Copyright (C) 2026 Open Source Robotics Foundation
SPDX-License-Identifier: Apache-2.0
-->

# Standalone Plugin metadata XML

`load_plugin_str` and `Plugin::to_xml` read/write one opaque SDF plugin record.
They do not load a shared library, execute configuration or create a simulation
system host. The simulator's built-in-system policy does not remove independent
SDF plugin metadata. A simulator may interpret recognized legacy records as
built-in configuration separately.

The reader delegates to the existing plugin parser: filename is required, name
is optional, attribute padding is trimmed and the complete element source span
is retained. Unknown root attributes, ordered child XML, comments, CDATA and
lexical namespace prefixes remain opaque. No new namespace validator or child
DOM facade is introduced. Input outside the root span is not retained.

The writer reuses the accepted Visual serializer. Its body and string guards are
unchanged; Visual keeps its original `<visual>` diagnostic scope, while the new
Plugin entry point uses `<plugin>`. Nonempty raw XML must be exactly one plugin
element with owned name/filename agreement. Public edits that disagree with its
raw root fail explicitly. An empty raw string emits an empty record from owned
attributes. Opaque input is otherwise emitted byte-for-byte and remains unchanged.
Native can canonicalize or drop unknown root attributes on emission; no general
native producer-byte or loaded mutation equivalence is claimed.

## Original assertion mapping

The proposed `DOMPlugin.ToElement` port retains literal name `my-plugin`, filename
`filename.so` and one inserted `an-element` child. Public XML construction replaces
the C++ insertion helper; pre-emission name/filename and child-count predicates
remain. `Result::Ok(nonempty XML)` translates nonnull returned output. Reload
checks both strings, one child and its exact name. A maintained XML reader
observes child names from the public raw string in tests; no production query
facade is added. Stream formatting, insertion failure, mutation/ownership and
runtime plugin originals remain unclaimed. This genuine original has no quiet
or input-Element identity predicate.

Author checks pass 543 Rust tests. Native proof covers 13 input observations, one
exact constructor XML fixture, 11 actual emitted-output exchanges and the
unchanged original. Raw input/output and child order are checked independently
of native metadata getters. Existing Visual native 28/fixture/25 exchanges and
original replay protect shared behavior and output. An initial author helper
export-variable typo was corrected before the successful export replay; no
production or assertion changes were required. Independent fresh source/owned
target review passed 33 selected Rust controls and reproduced both complete
native pipelines. Original mapping, extracted body, opaque strings/order and
diagnostics were approved without findings. Reviewed candidate counts are
201 complete/27 partial versus accepted 200/27 pending normal strict/exact
hosted qualification.

```sh
python3 tools/plugin_xml_reference.py ../sdformat
cargo test --offline --locked --test plugin_xml --test visual_xml
```
