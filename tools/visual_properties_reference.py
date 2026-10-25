#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Verify native visual property values and the GNU float getter ABI."""
import argparse
import hashlib
import json
from pathlib import Path
import shlex
import subprocess


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('upstream',type=Path)
    parser.add_argument('--output',type=Path,default=Path('artifacts/visual-properties/reference.json'))
    args=parser.parse_args()
    root=Path(__file__).resolve().parents[1]
    upstream=args.upstream.resolve()
    revision=subprocess.check_output(['git','-C',str(upstream),'rev-parse','HEAD'],text=True).strip()
    assert revision==json.loads((root/'reference/inventory.json').read_text())['upstream_sha']
    output=root/args.output;output.parent.mkdir(parents=True,exist_ok=True)
    binary=output.parent/'visual-properties-reference'
    flags=shlex.split(subprocess.check_output(['pkg-config','--cflags','--libs','sdformat'],text=True))
    libdir=subprocess.check_output(['pkg-config','--variable=libdir','sdformat'],text=True).strip()
    source=root/'tests/visual_properties_reference.cc'
    subprocess.run(['c++','-std=c++17',str(source),*flags,'-Wl,-rpath,'+libdir,'-o',str(binary)],check=True)
    corpus=root/'tests/visual_properties.json';observations=[]
    for case in json.loads(corpus.read_text()):
        run=subprocess.run([str(binary)],input=case['xml'],text=True,capture_output=True,check=True)
        parts=run.stdout.split();accepted=parts[0]!='error'
        assert accepted==case['accepted'],(case['id'],run.stdout,run.stderr)
        row=dict(id=case['id'],accepted=accepted)
        if accepted:
            actual=[int(value) for value in parts]
            expected=[int(case['cast_shadows']),case['transparency_bits'],case['visibility_flags'],int(case['has_laser_retro']),case['laser_retro_bits']]
            assert actual==expected,(case['id'],actual,expected)
            row['values']=actual
        observations.append(row)
    abi=output.parent/'float-abi.c'
    abi.write_text('''#define _GNU_SOURCE
#include <stdlib.h>
#include <locale.h>
#include <stdint.h>
typedef float (*conversion)(const char *,char **,locale_t);
_Static_assert(__builtin_types_compatible_p(__typeof__(&strtof_l),conversion), "strtof_l ABI");
_Static_assert(sizeof(float)==sizeof(uint32_t), "float32 storage");
''')
    subprocess.run(['cc','-std=c11','-Wall','-Wextra','-Werror','-fsyntax-only',str(abi)],check=True)
    sha=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
    paths=['src/Visual.cc','include/sdf/Visual.hh','src/Param.cc','include/sdf/Param.hh','src/Element.cc','include/sdf/Element.hh','sdf/1.12/visual.sdf','src/Visual_TEST.cc']
    for path in paths:
        expected=subprocess.check_output(['git','-C',str(upstream),'show',revision+':'+path])
        assert hashlib.sha256(expected).hexdigest()==sha(upstream/path),path
    linked=subprocess.check_output(['ldd',str(binary)],text=True)
    library=next(Path(line.split('=>',1)[1].split()[0]) for line in linked.splitlines() if line.strip().startswith('libsdformat.so.'))
    cpp_library=next(Path(line.split('=>',1)[1].split()[0]) for line in linked.splitlines() if line.strip().startswith('libstdc++.so.'))
    assert sha(library)==sha(upstream/'build/lib'/library.name),'native build/install library mismatch'
    proof=dict(upstream_sha=revision,corpus_sha256=sha(corpus),probe_source_sha256=sha(source),probe_binary_sha256=sha(binary),native_library_sha256=sha(library),native_cpp_library_sha256=sha(cpp_library),source_sha256={p:sha(upstream/p) for p in paths},stdlib_header_sha256=sha(Path('/usr/include/stdlib.h')),abi_check_sha256=sha(abi),fault_shim_sha256=sha(root/'tests/visual_locale_failure.c'),libc_version=subprocess.check_output(['ldd','--version'],text=True).splitlines()[0],observations=observations,native_observations=len(observations),matching_finite_or_error_observations=len(observations)-2,explicit_nonfinite_gaps=['transparency_inf','transparency_nan'],floating_environment='FE_TONEAREST; default C++ classic formatting; no other rounding-mode qualification')
    output.write_text(json.dumps(proof,indent=2)+'\n')
    print(f'{len(observations)} native visual rows verified; two nonfinite DOM gaps remain explicit')


if __name__=='__main__':main()
