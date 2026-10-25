# Owned joint axis dynamics

Axis.dynamics owns damping, friction, spring_reference and spring_stiffness, all
zero by default. Its optional Source and original XML distinguish an omitted
block from an explicitly empty declaration. Clones survive document destruction.
Axis.stiffness and Axis.dissipation describe joint-stop values independently of
spring stiffness, with native defaults 1e8 and 1. Both axis and axis2 use the same
loader, including axes retained on other supported joint kinds.

These are parser metadata only. No damping, friction, spring, solver or joint is
simulated here. Native DOM loading accepts negative values and signed zero without
clamping; consumers must validate or explicitly reject unsupported runtime use.
Damping/friction and stop parameters are optional and empty text takes the native
default. Explicit empty required spring_reference/spring_stiffness rejects. Existing
scalar prefix and GNU range semantics apply. The existing finite-double policy
still rejects native inf/nan metadata; all twelve observations are explicit gaps.
Two native-accepted custom children remain explicitly Unsupported. No new FFI,
math library, source-version conversion, name/frame or axis normalization change
is included. Core source versions 1.4 through 1.12 are probed.

The partial DOMJointAxis.Construction port retains twelve exact observations:
six dynamics/stop defaults and six per-field setter readbacks, using the Rust
parsed-default value and public field API. It does not claim a native constructor
facade. Element/null state, XYZ setter/error validation, missing-graph resolution,
bounds/effort/velocity defaults and mimic observations remain pending. In particular,
the native constructed-axis bounds are infinite while parsed schema bounds are
+/-1e16; no assertion substitutes one for the other. Original 2018 notices remain.
The unchanged complete native case is run as baseline evidence, not retired as a
complete port. Mimic, initial position, engine-specific joint parameters, mutable
Element/serialization and complete joint schema remain unsupported or pending.

Fresh native evidence binds 68 observations, the original case (one run, zero
failures), immutable source/schema/header hashes and installed/build library
identity. This is the qualified existing native SDK/build, not a completely
reproduced reference baseline. Independent source/assertion review and the exact
staged/hosted gates remain required before publication.
