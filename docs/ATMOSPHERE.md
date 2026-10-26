# Owned atmosphere metadata

`World.atmosphere` always contains an owned `Atmosphere`. Omitted XML produces
Adiabatic type, 288.15 K temperature, -0.0065 K/m temperature gradient and
101325 Pa pressure. Source/XML are absent for implicit defaults. An explicit
`<atmosphere type="adiabatic">` retains the parsed metadata and exact original
fragment/source location. Clones survive dropping the document. Raw XML records
input provenance and does not track later field edits.

The type attribute is required on explicit elements, including older supported
versions. Native space/tab/LF padding is trimmed. Case changes and unknown types reject;
a numeric-entity CR suffix in the type attribute is not trimmed and also rejects. Scalars reuse the audited native conversion, including
numeric prefixes, signed zero and empty scalar defaults. Finite negative metadata
is accepted as native does; runtime consumers validate the model they execute.
The 35-row corpus covers explicit pressure values across SDF 1.4-1.12.

Six native-accepted inputs remain explicit gaps: unknown attributes/children,
nonfinite pressure metadata, and native first-selection of duplicate atmosphere
or pressure elements. The bounded loader rejects these using its existing grammar,
finite-value and duplicate policies. Native warning/error-vector parity and full
schema support are not claimed.

The seven construction/set/copy/move originals preserve their fixed type,
Kelvin temperature, gradient and pressure observations. The two original equality
checks use native absolute <=1e-6 in a test helper; this does not create a public
C++ equality or Temperature facade. Runtime atmosphere sampling and ToElement
serialization remain unported.

This change adds a public World field. Exhaustive Rust struct literals must now
supply `atmosphere`, normally `Atmosphere::default()`. No such current Root consumer
literal was found; its integration pins remain coordinator-owned.
