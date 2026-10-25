# Native noise type loading

An explicitly declared `<noise>` needs a nonempty `type` attribute. A missing,
empty, NUL-truncated or ASCII-trimmed-empty attribute produces a source-located
Missing diagnostic. Omitted noise declarations still retain Noise::default()
with kind None; this is distinct from malformed explicit XML. No constructor,
noise sampling, scheduling or runtime policy changes.

Native Param string normalization and sdf::trim remove exactly space, tab and LF
from attribute edges before Noise's ASCII case-insensitive enum match. Literal
XML CR is normalized before references are decoded; a reference-created CR, VT,
FF or Unicode whitespace is not part of that trim set. No Rust Unicode trim is
used. The corpus covers all 25 Unicode whitespace codepoints as references,
literal six-character C whitespace, NUL and nonwhitespace BOM/ZWSP/joiner edges.
Existing XML normalization/reference behavior supplies the decoded attribute.

The supplementary positive in tests/imu.rs formerly contained `<z><noise/></z>`.
Native Root rejects that full exact XML. It remains in the corpus and Rust test
as a required-type negative. The positive now uses explicit `type='none'` on the
same noise node, preserving every value, comparison and unrelated error assertion.
Reversing that single literal replacement reconstructs the entire public-parent
file byte-for-byte; the disposition records hashes and full old XML. The unchanged
original sensors.sdf fixture and original IMU assertion subset remain intact.
This is an explicit obsolete supplementary acceptance disposition, not silent
retirement or a changed upstream assertion.

Tests retain existing name/pose/boolean envelope precedence, schema errors before
noise type, and type errors before numeric noise fields. The native 126-row corpus
covers every supported source version and all six noise records' exact types and
seven f64 bit patterns. Both behavioral regressions fail before the parser fix.
The initial new test's u32/usize column compile failure is separately retained
and is not classified as a behavioral red test.

`tools/noise_type_reference.py` compiles the probe and checks actual runtime
EmbeddedSpec noise bytes against pinned current schema plus embedSdf.py newline
framing with SDF_PATH removed. Source/header/library/binary/corpus/disposition
hashes and unchanged native DOMNoise.Load / DOMLink.Sensors runs are bound. The
native baseline uses the existing qualified build/SDK; there is no fresh full
native rebuild or new original case retirement claim. General nonfinite input,
scalar/vector and other schema limitations remain separate. Independent source
and obsolete-test disposition review approved the bounded correction. Exact staged
and hosted gates remain required before publication.
