<!--
Copyright (C) 2026 Open Source Robotics Foundation
SPDX-License-Identifier: Apache-2.0
-->

# Atmosphere metadata XML

`Atmosphere::to_xml` and `load_atmosphere_str` preserve typed Adiabatic sea-level
temperature in kelvins, temperature gradient in kelvins/metre and pressure in
pascals. This is metadata; no altitude, weather, pressure sensor or simulator
atmosphere model is evaluated. The standalone loader delegates unchanged to the
existing document Atmosphere parser.

All three output scalars must be finite and non-subnormal under the existing
native scalar conversion policy. Finite negative values and signed zero remain
metadata and are not given new physical restrictions. Output roundtrips retain
all three scalar bits. Required explicit type, defaults, unknown/duplicate XML
and source-located error policies remain unchanged. Native world omission still
uses defaults; an explicit declaration without type fails. Native unknown-type
fallback/nonfinite metadata differences remain explicit Rust errors.

The retained `source` and `xml` fields describe input origin. Output uses current
typed values, leaving both fields unchanged after public edits and emission;
reload receives its own literal URI/location and exact input XML. No Element
pointer identity or mutation behavior is implied.

## Original assertion mapping

Both genuine `DOMAtmosphere.ToElement` and `ToElementErrorOutput` configure
Adiabatic temperature 123 K, gradient 1.34 and pressure 2.65. The ports preserve
that complete setup and every returned-output/reload field assertion.
`Result::Ok(nonempty XML)` translates successful nonnull output and empty errors.
Temperature comparison retains native Temperature equality's absolute <=1e-6;
gradient and pressure retain the original four-ULP bounds. The ErrorOutput port
uses actual child-process stdout/stderr capture. No constructor/loaded Element
identity predicate exists in these two originals.

Author proof reproduces 23 actual native inputs, an exact native constructor
fixture, 14 real Rust-to-native XML exchanges, world-omission defaults and both
unchanged native originals with empty stderr. Scalar projections compare exact
bits. Unknown payload/type, duplicate scalar, missing type and nonfinite/subnormal
controls remain separate from successful output. Independent fresh source/owned
target review passed 14 selected Rust checks and reproduced all native proof.
Stdout/stderr injections each failed actual quiet capture; restored source
passed all 14. No findings remain. Reviewed candidate counts are 200 complete/27
partial versus accepted 198/27 pending normal strict staged and exact hosted
qualification.

```sh
python3 tools/atmosphere_xml_reference.py ../sdformat
cargo test --offline --locked --test atmosphere_xml --test atmosphere_xml_original
```
