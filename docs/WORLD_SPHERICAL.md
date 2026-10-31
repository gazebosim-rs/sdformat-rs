# World spherical-reference metadata

`World.spherical_coordinates` owns an optional `SphericalCoordinates` value.
An omitted block is `None`; an explicit empty block creates Earth WGS84 / ENU
metadata with zero reference latitude, longitude, elevation and heading. The
Source records the actual declaration URI, line and column. The value and its
Source survive dropping the XML buffer and can be cloned independently.

`SphericalSurface` distinguishes Earth WGS84 and Moon SCS.
`WorldFrameOrientation::EastNorthUp` records the only native accepted orientation.
Latitude, longitude and heading are radians; elevation is metres. Parsing uses
`degrees * std::f64::consts::PI / 180.0` in the native gz-math Angle order, including
signed zero. No latitude/longitude range restriction or angle normalization is
added. The parser does not calculate a position, transform a coordinate, derive
surface parameters, sample a sensor, or instantiate runtime state.

The pinned Root/schema inserts omitted required children with Earth and zero
defaults. Explicit empty surface/latitude/longitude/elevation/heading values fail;
empty optional orientation uses ENU. Surface and nonempty orientation strings
are case sensitive after native parameter text handling. SDF 1.4–1.12 inputs
with the original Earth reference fields are qualified. No general version
conversion or broader spherical schema parity is claimed.

Custom surfaces and either surface-axis child return Unsupported. Native custom
ellipsoid fallback, derived radius/flattening and geodesy remain unqualified.
Unknown group children and attributes return Unsupported. Nested parameter
content also fails explicitly; the existing required-scalar empty check can
return Missing before validating an empty parameter's nested content or attributes.
Duplicate singleton groups/children return InvalidValue, while native accepts the
first value. Invalid supported strings and invalid numeric conversions return
InvalidValue. Explicit empty required values return Missing. Loading is atomic;
native error-vector multiplicity, partial DOM and warning streams are not exposed.

The existing finite scalar policy rejects NaN/infinity and conversion range
errors. Angles whose intermediate multiplication by PI overflows also return
InvalidValue; the native DOM can retain infinity for a finite input such as
1e308 degrees. Values are not silently normalized, clamped or replaced.

`tools/world_spherical_reference.py UPSTREAM` replays the captured inputs in
`tests/world_spherical.json`. It verifies the pinned clean source, canonical
installed SDF/math headers and libraries against their source builds, records
all loaded library hashes, uses the selected libdir for child transitive loading,
and clears LD_PRELOAD/LD_AUDIT. The original full world fixture and its exact
spherical subtree projection produce identical getter bits. The unchanged
native `DOMWorld.Load` original passes. The partial Rust port preserves World
presence/count/name, coordinate presence, Earth surface, three native
EXPECT_DOUBLE_EQ angle observations (four ULP) and elevation. GUI, remaining
World fields and native Element/path identity remain outside this mapping;
DOMWorld.Load remains partial. No original-case counters advance.

Independent review passed five prepared Rust tests, all-target Clippy and 57
native controls plus constructor/full-projection/unchanged original. Root
approved only the exact SDF 1.12 empty-block/root.sdf rejection transition,
which now uses the reviewed complete World assertion. Author folded replay
passes seven affected Rust tests. Final fold review, the protected staged gate
and exact hosted success remain pending.
