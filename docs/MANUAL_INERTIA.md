<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Explicit manual inertia

`inertial auto="false"` and `auto="0"` select the existing manual mass,
inertia and pose loader. Omitted and empty attributes have the same false
default. ASCII case and space/tab/newline padding follow native Param boolean
parsing; character-reference carriage returns and other unknown tokens reject.
The canonical pinned Root loader admits these spellings across SDF 1.4–1.12,
including older documents converted before DOM loading. This does not claim
historical-schema validation or expose a new automatic-inertia API.

Outside the bounded [explicit-density origin-box path](AUTO_BOX_INERTIA.md),
true/1 remains Unsupported, including the historical native-valid unit box
whose collision density is omitted. Link-level density and auto_inertia_params remain
Unsupported even with auto=false; their metadata is not silently discarded.
Unknown attributes/children, duplicate elements and all existing finite,
positive-mass and physically realizable inertia checks remain unchanged.
No computation, collision aggregation, mesh IO or new value type is added.
The admitted false/default spelling normalizes to the existing Inertial value;
there is no public flag or explicit-presence getter. Sources and input version
retain their existing loader meaning.

`DOMLink.ResolveAutoInertialsCalledWithAutoFalse` uses the byte-identical inline
SDF 1.11 fixture. The partial Rust mapping covers successful loading and default
mass 1 / diagonal moments (1,1,1). Native Root.Element identity and the separate
ResolveAutoInertials call/error vector are not recreated; no complete-case or
automatic-inertia credit is claimed. The full unchanged C++ test passes.

The frozen corpus has 60 native controls, including a valid automatically computed
box that Rust still refuses. The full unchanged C++ original and a hostile-Git
replay pass. Installed schemas/conversion files, headers, compiler and selected
libraries are checked against the pinned sources; native replay clears child
preload/audit overrides.

Sixteen focused Rust tests pass, including three new controls and retained
manual/default/fluid/link/frame tests. All-target Clippy and formatting pass.
The first exact supplemental comparison exposed one ULP in an existing
omitted-auto Euler quaternion. That failure is retained. Root source review
confirmed the native gtest four-ULP convention for only these positive nonzero
quaternion controls; mass, all inertia components, translation, original/default
values and all omitted/false/0 Rust bit identities remain exact. No math code or
retained bound changes. Root independently reviewed the exact12-path packet from
a fresh accepted6bd archive and own target:16 focused tests,60 native controls,
the unchanged original, all-target Clippy and formatting pass; all583 preexisting
test/tool bytes are unchanged. Author warm-target full673 tests also pass with
zero failures or ignored tests. The mapping remains partial; ordinary strict
staged/hosted qualification is pending. Consumer pins remain unchanged.
