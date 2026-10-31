# World GUI declaration metadata

`World.gui` is optional owned `Gui` metadata. An omitted block is None; an
explicit empty block is Some with fullscreen=false and no plugins. The value
owns the actual declaration Source, exact input XML and ordered Plugin records.
Loading never opens a window, invokes a GUI framework or activates a plugin.
The new public World field affects exhaustive struct literals and destructuring;
consumers must explicitly support or reject it before adopting the producer.
No simulator pin changes here.

The fullscreen attribute follows native schema boolean conversion: omitted,
empty, space/tab/LF-only, false and 0 are false; true and 1 are true. Words are
case insensitive after space/tab/LF edge trimming and XML attribute decoding.
Other spellings, numeric prefixes, carriage-return character references and
non-ASCII edge whitespace fail with InvalidValue. The reference corpus covers
SDF 1.4–1.12 and plugin declarations at the old/current schema boundaries.

Plugins reuse the existing opaque metadata parser, preserving names, filenames,
order, duplicate names and exact plugin subtrees. Missing or empty filenames
return Missing. Plugin content and XML/source snapshots survive dropping the
input buffer and independent cloning. Typed edits do not rewrite either snapshot.
There is no Gui default-constructor facade, native equality facade, serializer,
mutable Element identity or plugin/runtime loader.

Camera controls and other GUI children or attributes return Unsupported rather
than being dropped. Duplicate GUI blocks return InvalidValue; native retains
the first block. The loader returns its first owned diagnostic atomically, not
native partial DOM, warning output or full error-vector multiplicity.

The pinned Gui.cc, World.cc, Param.cc, Plugin.cc and schemas were inspected before
implementation. `tools/world_gui_reference.py UPSTREAM` verifies the clean pinned
source and matching installed SDK headers/library, hashes actual transitive
libraries, clears LD_PRELOAD/LD_AUDIT and sets the selected child library path.
It replays 54 complete native controls, constructor defaults, full/projected GUI
getter equality and the unchanged native DOMWorld.Load original. The existing
native light helper additionally checks the exact old GUI-plus-light input,
including all supported light getters and its semantic pose.

The partial World.Load assertion port retains World presence/count/name, owned
GUI presence and fullscreen=true on the unmodified original GUI subtree. Both
plugin records retain their exact names, filenames, order and XML. Source is the
caller's actual declaration URI/position; it is not native Element/FilePath or
pointer-identity equivalence. Remaining World/Element assertions stay unported,
and DOMWorld.Load remains partial. No original-case counters advance.

Independent prior review passed five prepared Rust tests and all 54 native
controls, full/projection/original getters and the coexisting Light semantic
pose. Root approved only the seven exact old input/URI transitions. Each now
uses the reviewed complete World positive in its original test branch; all
other predicates and corpus bytes remain unchanged. Author folded replay passes
19 affected Rust tests, all 615 retained Rust tests, 44 tooling checks and
Clippy. The final 54-case native replay also passes. The native driver isolates only repository-local
Git environment during its foreign source checks and binds its own script hash
in provenance; the actual inherited-environment calls were independently checked.
Final fold review passed, including a fresh independent 19-test replay. The
first normal staged hook then failed closed on one absent-DA constructor prefix;
its temporary export was removed. A separate persistent approved-source LLVM
diagnostic confirms 22/22 measured changed lines and 7825/7852 production lines,
12289/12531 regions. The bounded existing constructor proof now covers Gui,
with 27 helper tests, 45 tooling tests, three real LLVM controls and Clippy
passing. Independent proof review, the normal protected retry and exact hosted
success remain required.
