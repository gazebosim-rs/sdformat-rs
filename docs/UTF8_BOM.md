# UTF-8 BOM source spans

The pinned quick-xml slice reader consumes an initial UTF-8 BOM without counting
its three bytes in reported buffer/error positions. The SDF cursor keeps original
XML text, so those positions need the same three-byte offset before they identify
original source ranges. The correction applies to element starts, ends and
reported tokenizer errors. The tokenizer input remains unchanged, preserving its
one-time BOM handling and the existing document grammar.

Previously, valid BOM-prefixed XML could corrupt preserved plugin/configuration
subtrees or panic when a shifted boundary landed inside a UTF-8 scalar. Exact
public-parent tests retain both failures: a truncated/incorrect plugin subtree and
a Unicode model name followed by a plugin that caused a character-boundary panic.
The same valid input is accepted by native Root. The corrected loader preserves
plugin, sensor configuration, physics, axis dynamics and include XML, including
separately BOM-prefixed imported resources and canonical source identifiers.

Source coordinates follow the existing full-original-text xmlparser routine:
LF rows and Unicode scalar columns. A leading BOM therefore counts as one source
column, while its three bytes are retained for slicing. This corrects previously
shifted BOM positions; ordinary non-BOM coordinates, error codes/messages and
raw span contracts are retained. No native column/diagnostic byte parity or
normalized serialization claim is made.

The 87-row native corpus covers supported source versions 1.4–1.12, ASCII/Greek/
emoji model names, initial BOM/declarations, world physics and opaque interior
BOM text. Paired native metadata is identical with/without an initial BOM.
Eighty-four rows match; three named repeated/intermediate BOM document-text cases
remain existing strict XML grammar differences, with native acceptance and Rust
Xml rejection. They are explicit gaps, not passing parity observations.

Nine focused Rust tests additionally compare public positions against the full
source routine across 96 Unicode/EOL/opaque-child layouts, preserve syntax/graph
error coordinates and codes/messages, and verify model.config warning locations.
The two behavioral red tests were run against the unchanged public parent before
correction. Existing assertions and the original test ledger are unchanged.

```sh
python3 tools/utf8_bom_reference.py ../sdformat
cargo test --locked --test utf8_bom --test source_positions
```

The runner freshly compiles the native probe and binds pinned upstream source,
SDK headers/libraries, native baselines and actual quick-xml/xmlparser source
versions. Three unchanged native originals each execute once without failures or
skips. These are qualified existing native builds, not a fresh full reference
build. The source-position performance cache is a separate prototype; this change
adds no dependency, unsafe code, runtime sensor behavior or new case retirement.
Independent source/assertion review passed; mandatory staged/hosted qualification
remains required.
