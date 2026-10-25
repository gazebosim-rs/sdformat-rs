# Pure model.config selection

`select_model_config(xml, source)` returns an owned ModelConfigSelection with a
file_name, optional declared_version and ordered, source-located warnings. It
performs no IO. Callers own model.config versus deprecated manifest.xml lookup,
path assembly/containment, offline cache policy and subsequent SDF loading.
A selected filename is raw metadata, including relative/absolute paths, URIs,
whitespace or an empty first text node; this function does not validate paths.

The pinned parser advertises version 1.12. Selection scans direct sdf children,
chooses the highest version no newer than 1.12 and retains the first tie. Its
initial candidate is the first sdf child, and its initial best version is 0.0.
If no candidate improves that best version, the first child remains selected,
even if it declares a future version. Future candidates generate warnings;
missing version generates a warning only if the selected child lacks it. Loading
the returned file remains a separate operation with its own version/capability
checks. Choosing a filename does not establish that its content is supported.

The native gz-math SemanticVersion parser differs from standard SemVer: missing
components, signed-32-bit numeric prefixes, extra dot components overwriting the
major component, and ignored prerelease label ordering affect selection. Build
metadata and malformed delimiter ordering also have native effects. A narrow
measured adapter retains these rules; a standard semver crate would change them.
It uses safe Rust only. Native stoi exceptions for invalid/out-of-range components
become source-located InvalidValue errors in the new Result API. Native warning
construction can throw or dereference a missing model name; typed warning values
safely retain those selections instead. Native stderr/error text is not reproduced,
and errors return one Diagnostic rather than a partially accumulated error vector.

Filename text follows TinyXML GetText semantics, rather than SDF Param strings:
ordinary text collapses six-character C whitespace, CDATA preserves whitespace,
first text nodes win, and decoded NUL terminates the native string view. Empty
CDATA differs from an absent text node. The existing legacy XML cursor supplies
lexical names/references and source ranges; this addition does not broaden its
qualified XML grammar or provide an Element/document serialization facade. Five
explicit accepted-native XML gaps remain: leading/trailing extra roots, a second
model root, leading document text and DOCTYPE. Rust returns Xml or Unsupported
for these inherited boundaries; they are not counted as native/Rust parity.

The native corpus contains 97 observations covering version ordering/ties/fallback,
partial/prefix/overflow/prerelease/build forms, warnings, invalid XML, name-failure
boundaries, text/CDATA/comments/child/reference/NUL behavior and path-like names.
`tools/config_reference.py` rebuilds a C++ probe calling actual getModelFilePath in
an isolated directory, removes only the directory prefix from the returned path,
and compares status/stdout/stderr exactly. Two native original model-version cases
also pass against untouched fixtures. Rust retains only their filename-selection
portions; filesystem path assembly remains pending, so both mappings are partial.
The probe binds parser/math sources, installed headers, linked libraries, fixtures
and native original binaries, including libc/libstdc++ and installed parser
header identity. Existing native builds/installed dependencies are
qualified rather than claimed to be a fully reproduced immutable baseline.

Independent source/assertion/provenance review, strict staged checks and hosted CI
remain required before publication. No full retained case, SDF packet or performance
acceptance is implied by this bounded resource-loading helper.
