# Owned visual material metadata

Visual.material is optional boxed Material data. Presence distinguishes no material
from an explicitly empty material with defaults. Material owns ambient, diffuse,
specular and emissive RGBA float32 arrays, shininess float64, render_order float32,
lighting/double_sided flags, ShaderType, normal-map string, optional script metadata
and Source. MaterialScript owns ordered URI strings, name and Source. Its first URI
matches native Material::ScriptUri; later declarations remain available as metadata.
No script/image loading, texture lookup or rendering/lidar effect is performed.
Native StoreResolvedURIs mode and material file-path resolution remain separate.

Empty materials use black/opaque colors, zero shininess/render order, lighting=true,
double_sided=false and pixel shading. Script presence requires a valid first URI
and name; native __default__ placeholders count as missing. Later URI declarations
retain schema defaults. Script names use native ordinary-text collapse and the
space/tab/LF parameter trim while preserving NBSP and CDATA VT. Shader presence requires its type attribute. Both native
object/tangent normal-map spellings map to owned enum variants, and those shaders
require a nonempty normal map. PBR remains explicitly Unsupported in this slice.

Param's color parser tokenizes in the classic locale, calls std::stof for each
component and requires three/four values in [0,1]; RGB gets alpha one. It accepts
numeric prefixes and rejects float range errors, including subnormal ERANGE.
NaN passes its range comparison and is then clamped to zero by Color construction;
infinities reject. This differs from Color::operator>> and from vector stream
parsing. Render order uses direct std::stof and retains native inf/nan metadata.
Shininess retains the existing finite-double policy, with its two nonfinite gaps
explicit in the corpus. Unknown/custom scalar children remain Unsupported.

The raw float conversion reuses the already audited GNU strtof_l ABI, C-locale
ownership, NUL buffer, immediate errno capture and cleanup/restoration. No process
or thread locale is changed, no pointer escapes and no new ABI is declared. The
stored-double getter still formats 17 significant digits and defaults zero on
conversion failure; it preserves errno across formatting and temporary destruction.
Direct-float callers retain prefix/ERANGE semantics, while field-specific validation
happens after conversion. Linux GNU and FE_TONEAREST remain the measured platform;
no other rounding-mode, portability or performance claim is added.

Native corpus rows retain exact float/double bits, acceptance, shader/flags and
script strings, with four explicit PBR/schema/double-nonfinite gaps. The full
unchanged material.sdf fixture and DOMVisual.Material assertions are ported with
original color <=1e-6 and EXPECT_FLOAT_EQ four-ULP bounds. Original 2018 notices
remain. Extra tests retain script order/source ownership, allocation-failure cleanup,
errno and independent concurrent parsing. The former exact empty-material negative
inputs remain positive default-material regressions; unrelated rejection tests stay.

Independent /root/msgs source/assertion/provenance and numeric safety review
approves the corrected isolated scope. Exact integrated overlap review, strict
frozen staged gate and hosted CI are still required before publication. Setter/Element/error-vector
contracts, PBR, URI resolution modes, renderer effects and full material/schema
qualification remain pending.
