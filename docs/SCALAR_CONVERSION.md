# Native scalar conversion boundary (Linux GNU)

Pinned Param.cc forces C numeric locale and uses std::stod for scalar doubles
and individual pose tokens. Numeric prefixes are accepted; no conversion and
ERANGE are errors. Rust f64 parsing loses ERANGE. Native controls distinguish
exact subnormal decimal values from inexact underflow and rounded-normal edges.
A pure-Rust error-status experiment disagreed on these controls; an approximate
underflow heuristic would accept invalid inputs or reject valid subnormals.

The crate-private safe API is scalar_float(&str) -> Result<f64, ErrorCode>.
Optional-empty defaults and scalar Param true/false coercion remain in the
parser. Pose tokens use the raw helper without boolean coercion. Vector streams
keep a separate path because native vector underflow behavior differs. Nonfinite
semantic values remain rejected; their broader native DOM parity is still open.

On Linux GNU/glibc, maintained libc types plus safe errno access support a narrow
strtod_l boundary. Its local declaration matches the installed stdlib.h ABI:
double strtod_l(const char *, char **, locale_t). libc does not expose this GNU
function declaration on this target. newlocale creates an owned C LC_NUMERIC
locale with a null base; freelocale releases each valid result exactly once.
There is no setlocale/uselocale call or shared locale object in production.

An owned byte buffer stays NUL-terminated and alive throughout conversion.
Interior NUL terminates conversion as with native C-string getters. The end
pointer is only compared with the input start; it is never dereferenced, sliced
or retained. No native pointer escapes. The C functions do not throw exceptions;
no Rust callback or panic crosses the ABI. Allocation follows the normal Rust
allocator contract. Locale allocation failure returns Resource. No conversion,
ERANGE and nonfinite values return InvalidValue. Returned finite IEEE bits are
preserved, including signed zero and exact subnormals.

Caller thread-local errno is saved before allocation, cleared for conversion,
captured immediately after strtod_l, then restored after locale/buffer cleanup.
Each call owns its locale; concurrent callers share no mutable parser state.
Production unsafe code is denied except for the allow-scoped src/number.rs
module. A separate test-only allow scope queries global LC_NUMERIC without
changing it, immediately copying its C string; no test in that process writes
locale. Both scopes require independent code/ABI/safety review. No C++ library
is introduced. Dependencies retain their original licenses.

Other platforms select a safe Unsupported implementation for explicit
scalar/pose-token conversion; no build/runtime qualification is claimed. The execution contract is Linux
x86_64 GNU/glibc, not portable numeric parity. The required integration tests use
a C compiler for an isolated LD_PRELOAD allocation-failure child: a failed locale
must return Resource, restore errno and never reach freelocale(null). Native
scalar/Pose/vector controls, concurrent errno checks and unchanged-global-locale
checks are retained alongside all previous original assertion ports.

Per-call locale ownership has a cost. No parsing performance improvement or
cached-locale optimization is claimed. Native and complete-parser benchmarks
remain required before optimization or performance claims. Surface metadata
remains separately unaccepted until its numeric correction and independent
review pass the matching strict staged gate.
