<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Heightmap metadata XML

Heightmap::to_xml, load_heightmap_str and Geometry XML preserve existing URI,
size/position vectors, paging, sampling, ordered texture records and blend
records. They reuse the existing reader without resolving resources or decoding
images. No terrain generation, rendering, collision or physics behavior follows.

Finite vector components retain signed zero and finite subnormals. Negative and
zero size/position components remain metadata; no physical constraint is added.
Texture size and blend scalars retain the existing finite/non-subnormal output
policy, including negative values and signed zero. Native accepts infinite scalar
input that the existing Rust reader rejects; that named difference stays explicit.
Sampling preserves the existing full u32 domain, including zero. Texture/blend
ordering, duplicates and counts are retained without inventing a count relation.

Omitted URI/texture strings use existing __default__ values; explicit empty strings
fail. Empty vector nodes keep defaults, whereas required empty scalar children
fail. Empty optional sampling keeps its default. Unknown nodes/attributes,
duplicate singleton nodes and extra vector components retain the existing strict
reader boundaries despite native admission. These policies are not broadened.

Text uses the shared native-Param policy accepted for Mesh. Ordinary XML escaping
preserves markup; one CDATA section retains internal repeated spaces/tabs/line
feeds. Outer ASCII whitespace and forbidden XML characters fail. A value requiring
CDATA and containing ]]> is Unsupported because native truncates split CDATA.
Unicode nonbreaking spaces are preserved. Mesh's prior codes/messages and accepted
text policy are unchanged by extracting the common validator; Contact keeps its
own stricter validator and identical output on its accepted domain.

Native default-constructed Heightmap output returns nonnull XML with error 34,
then reloads the schema URI default. This invalid partial-output contract remains
unimplemented. The valid writer returns a Result and does not imitate partial
errors. Both Geometry originals stay partial for their invalid producer blocks.

The two valid Heightmap originals check every configured URI/vector/paging/
sampling field, texture/blend counts and values, successful output/reload and
actual empty console in the error-output case. Their exact ports preserve vector
component <=1e-3 and scalar four-ULP assertions. They have no input Element identity
assertion. Both exact complete-case mappings are independently reviewed.

Independent prior review approved exactly two temporary Geometry rejection
transitions before edits, preserving their literal XML and caller URIs. Complete
native observations, document ancestry/source, immutable emission and successful
field reload now replace those predicates. All unrelated failures and genuine
originals remain unchanged. No full Geometry credit is proposed.

All 517 author Rust tests, 17 tooling checks, formatting and all-target Clippy pass.
Author native proof passes 28 inputs, one exact configured fixture, two exact old
inputs, 15 actual Rust/native exchanges and both unchanged originals with empty
stderr. Existing Mesh proof passes 28 inputs/18 outputs, Geometry passes 48/45
and Contact passes 16/17 plus their unchanged originals. Independent fresh-source
review reproduced all native pipelines and passed 27 selected Rust checks. Both
private stdout/stderr injections failed quiet capture; restored source passed all
27. Every exact original predicate and shared text/numeric boundary is reviewed
with no findings. Strict coverage and hosted qualification remain pending. Candidate 195 complete/27 partial remains separate
from accepted 193/27. No resource or runtime support follows.
