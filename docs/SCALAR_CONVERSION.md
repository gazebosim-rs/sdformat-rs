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
remain required before optimization or performance claims. Surface metadata was
subsequently reviewed on this boundary; physics parameter mapping remains a
separate consumer capability.


## Typed visual float getter

Visual XML transparency is a double Param. Native Visual::Load requests float;
Param::Get therefore prints the stored double with max_digits10 (17 significant
digits), then uses std::stof. Range errors in that second conversion keep the
float getter default zero and do not enter Root load errors. A direct f64-to-f32
cast is observably different at float midpoints. Raw-token strtof also bypasses
the first double conversion and loses native range/fallback behavior.

The private stored_double_to_float(f64) -> Result<f32, ErrorCode> follows that
finite-double getter path. Its sole parser caller first uses the existing scalar
checker. The standard Rust formatter emits 17 significant decimal digits in
scientific notation; exponent spelling and trailing zeros do not change stof's
numeric input. The qualified floating environment is Linux GNU/glibc with default
FE_TONEAREST and native classic formatting. Other rounding environments are not
qualified. Two native-accepted nonfinite DOM controls remain rejected by the
existing finite scalar policy, explicitly outside matching finite/error rows.

The added local strtof_l declaration matches the installed stdlib.h float ABI.
As with strtod_l, each call owns a fresh C numeric locale and NUL-terminated input;
end is comparison-only, errno is captured immediately and restored after freeing
the valid locale and dropping the buffer. Invalid/range conversion returns the
native getter's zero default. Locale allocation failure remains Resource. There
is no process/thread locale switch, pointer escape, callback or new dependency.
The existing deny lint and single allow-scoped module remain unchanged. This
extension requires independent code, ABI, lifecycle and assertion review before
publication; prior strtod approval alone does not approve strtof.

Native controls compare exact float bits for signed zero, both signs of midpoint
neighbors, normal/subnormal/overflow edges, boolean/prefix/hex input and malformed
values. The original scalar global-locale snapshot also exercises visual inputs.
Concurrent errno tests and a separate LD_PRELOAD child force failure on the second
locale allocation, checking cleanup after the first successful double conversion
and no null free. No formatting or parsing speedup is claimed.

Native float-getter conversion failures also write console diagnostics. This
new-value Rust API retains the returned zero and successful DOM load, but does
not emit those console warnings; diagnostic bytes and side effects are not
qualified. No mutable Visual setter or serializer facade is introduced.

The material increment reuses the existing strtof_l boundary for direct float text.
`number::float` owns the same locale/buffer lifecycle and reports no-conversion or
ERANGE as InvalidValue. It deliberately returns nonfinite float values so each
metadata field can apply the native rule: color clamps NaN to zero and rejects
infinity/out-of-range components; render_order retains inf/nan metadata. The
stored-double wrapper retains its prior zero default on conversion errors and
preserves caller errno across formatting/deallocation. No ABI declaration changes.
This refactor requires its own independent code/safety review; the previous
stored-double review alone does not approve these new bytes or callers.
