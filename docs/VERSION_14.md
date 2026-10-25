# Bounded SDF 1.4 loading

The common supported world/model subset now accepts source version 1.4. Existing
1.5–1.12 behavior stays required. The 1.4-to-1.5 converter adds
use_parent_model_frame=true to existing axis/axis2 elements of direct root models
and direct world models. It does not visit nested models. An existing flag keeps
first-element precedence, even though the native converter appends another flag.
The subsequent 1.6-to-1.7 mapping supplies effective expressed_in=__model__ only
for the already qualified four true spellings. An implicitly added true flag with
missing xyz produces a required-value error. Nested missing xyz keeps its usual
schema default. No axis element is invented when it was absent.

The parser implements this effective metadata directly, retaining source XML and
source version rather than exposing a converted Element. Scope comes from the
resource document's own XML root; an included 1.4 model gets the conversion even
inside a modern parent. Modern resources included by an older parent keep their
own rules. Merged include reference rewriting happens afterward as before.

The same table also renames an actor attribute. Actors remain explicitly
Unsupported, so no actor conversion or execution is claimed. Earlier source
versions remain Version errors. General schema/Element/printing/convert-to-version
interfaces and complete historical compatibility remain separate SDF-02 work.

The 61 native observations cover direct/nested/world models, both axes, omitted
and explicit flags, text/CDATA boundaries, missing xyz, mixed-version ordinary and
merged resources, and the exact formerly unsupported simple 1.4 input. Numeric
axis comparisons use a supplementary 1e-12 component bound; no original numeric
assertion is replaced. The existing exact 1.4 negative input becomes a positive
regression, and version 1.3 retains an explicit lower-floor check.

Two unchanged native JointAxisFrame missing-flag originals pass. Rust retains typed
load success, axis presence and the 1.4 effective __model__ string observation;
converted Element presence/removal assertions remain pending. Both mappings are
partial, with original 2014 source notices preserved and no complete-case retirement.
The native generator binds conversion tables, parser/Param/joint code, installed
SDK/math headers, native library, observer codec, corpus and original binaries.
Existing build/installed dependency qualification remains; this is not an immutable
full-reference rebuild or performance acceptance.

Independent source/assertion/provenance review, integration overlap and exact
strict staged/hosted gates are required before publication.
