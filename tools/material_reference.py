#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Verify material metadata, direct-float ABI and the untouched original fixture."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import xml.etree.ElementTree as ET


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--output', type=Path, default=Path('artifacts/material/reference.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = root / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == json.loads((root / 'reference/inventory.json').read_text())['upstream_sha']
    source = root / 'tests/material_reference.cc'
    binary = output.parent / 'material-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    libdir = subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()
    subprocess.run(['c++', '-std=c++17', str(source), *flags, '-Wl,-rpath,' + libdir, '-o', str(binary)], check=True)
    corpus = root / 'tests/material.json'
    observations = []
    cases = json.loads(corpus.read_text())
    for case in cases:
        run = subprocess.run([str(binary)], input=case['xml'], text=True, capture_output=True, check=True)
        tokens = run.stdout.split()
        assert tokens == case['tokens'], (case['id'], run.stdout, run.stderr)
        observations.append(dict(id=case['id'], tokens=tokens))
    abi = output.parent / 'float-abi.c'
    abi.write_text('''#define _GNU_SOURCE
#include <stdlib.h>
#include <locale.h>
#include <stdint.h>
typedef float (*conversion)(const char *,char **,locale_t);
_Static_assert(__builtin_types_compatible_p(__typeof__(&strtof_l),conversion), "strtof_l ABI");
_Static_assert(sizeof(float)==sizeof(uint32_t), "float32 storage");
''')
    subprocess.run(['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', '-fsyntax-only', str(abi)], check=True)
    files = ['src/Material.cc', 'src/Param.cc', 'src/Utils.cc', 'include/sdf/Material.hh',
             'sdf/1.12/material.sdf', 'test/integration/visual_dom.cc', 'test/sdf/material.sdf']
    for path in files:
        expected = subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':' + path])
        assert hashlib.sha256(expected).hexdigest() == sha(upstream / path), path
    sdk_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    assert sha(sdk_include / 'sdf/Material.hh') == sha(upstream / 'include/sdf/Material.hh')
    math_source = upstream.parent / 'gz-math'
    math_revision = subprocess.check_output(['git', '-C', str(math_source), 'rev-parse', 'HEAD'], text=True).strip()
    math_files = ['src/Color.cc', 'include/gz/math/Color.hh', 'include/gz/math/Helpers.hh']
    for path in math_files:
        expected = subprocess.check_output(['git', '-C', str(math_source), 'show', math_revision + ':' + path])
        assert hashlib.sha256(expected).hexdigest() == sha(math_source / path), path
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], text=True).strip())
    for name in ['Color.hh', 'Helpers.hh']:
        assert sha(math_include / 'gz/math' / name) == sha(math_source / 'include/gz/math' / name)
    fixture = root / 'tests/fixtures/material.sdf'
    assert sha(fixture) == sha(upstream / 'test/sdf/material.sdf')
    linkage = subprocess.check_output(['ldd', str(binary)], text=True)
    libraries = {}
    for line in linkage.splitlines():
        if '=>' in line and any(line.strip().startswith(name) for name in ('libsdformat.so.', 'libgz-math.so.', 'libstdc++.so.', 'libc.so.')):
            path = Path(line.split('=>', 1)[1].split()[0])
            libraries[path.name] = sha(path)
            if path.name.startswith('libsdformat.so.'):
                assert sha(path) == sha(upstream / 'build/lib' / path.name)
    original = upstream / 'build/bin/INTEGRATION_visual_dom'
    report = output.parent / 'original.xml'
    env = dict(os.environ)
    env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(Path.home() / 'local/lib') + ':' + env.get('LD_LIBRARY_PATH', '')
    run = subprocess.run([str(original), '--gtest_filter=DOMVisual.Material', '--gtest_output=xml:' + str(report)], env=env, text=True, capture_output=True, check=True)
    (output.parent / 'original.log').write_text(run.stdout + run.stderr)
    result = ET.parse(report).getroot()
    assert result.attrib['tests'] == '1' and result.attrib['failures'] == '0'
    proof = dict(upstream_sha=revision, corpus_sha256=sha(corpus), probe_source_sha256=sha(source),
                 probe_binary_sha256=sha(binary), linked_libraries_sha256=libraries,
                 source_sha256={path: sha(upstream / path) for path in files},
                 math_upstream_sha=math_revision, math_source_sha256={path: sha(math_source / path) for path in math_files},
                 fixture_sha256=sha(fixture), original_binary_sha256=sha(original), original_tests=1, original_failures=0,
                 stdlib_header_sha256=sha(Path('/usr/include/stdlib.h')), abi_check_sha256=sha(abi),
                 fault_shim_sha256=sha(root / 'tests/visual_locale_failure.c'),
                 floating_environment='FE_TONEAREST; explicit C numeric locale; no other rounding-mode qualification',
                 explicit_gap_ids=[case['id'] for case in cases if 'gap' in case], observations=observations)
    output.write_text(json.dumps(proof, indent=2) + '\n')
    print(f'{len(observations)} native material rows, original DOMVisual.Material and direct-float ABI verified')


if __name__ == '__main__':
    main()
