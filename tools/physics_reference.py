#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Reproduce world physics metadata and the original construction assertions."""
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
    parser.add_argument('--output', type=Path, default=Path('artifacts/physics/reference.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = root / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == json.loads((root / 'reference/inventory.json').read_text())['upstream_sha']
    source = root / 'tests/physics_reference.cc'
    binary = output.parent / 'physics-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    libdir = subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()
    subprocess.run(['c++', '-std=c++17', '-Wall', '-Wextra', '-Werror', str(source), *flags, '-Wl,-rpath,' + libdir, '-o', str(binary)], check=True)
    corpus = root / 'tests/physics.json'
    cases = json.loads(corpus.read_text())
    observations = []
    for case in cases:
        run = subprocess.run([str(binary)], input=case['xml'], text=True, capture_output=True, check=True)
        tokens = run.stdout.split()
        assert tokens == case['tokens'], (case['id'], run.stdout, run.stderr)
        observations.append(dict(id=case['id'], tokens=tokens))
    files = ['src/Physics.cc', 'src/World.cc', 'src/Param.cc', 'src/Utils.cc',
             'src/Physics_TEST.cc', 'include/sdf/Physics.hh', 'include/sdf/World.hh',
             'sdf/1.12/physics.sdf', 'sdf/1.4/physics.sdf']
    for path in files:
        original = subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':' + path])
        assert hashlib.sha256(original).hexdigest() == sha(upstream / path), path
    include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    for name in ['Physics.hh', 'World.hh']:
        assert sha(include / 'sdf' / name) == sha(upstream / 'include/sdf' / name)
    libraries = {}
    for line in subprocess.check_output(['ldd', str(binary)], text=True).splitlines():
        if '=>' in line and any(line.strip().startswith(name) for name in ('libsdformat.so.', 'libgz-math.so.', 'libstdc++.so.', 'libc.so.')):
            path = Path(line.split('=>', 1)[1].split()[0])
            libraries[path.name] = sha(path)
            if path.name.startswith('libsdformat.so.'):
                assert sha(path) == sha(upstream / 'build/lib' / path.name)
    original = upstream / 'build/bin/UNIT_Physics_TEST'
    report = output.parent / 'original.xml'
    env = dict(os.environ)
    env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(Path.home() / 'local/lib') + ':' + env.get('LD_LIBRARY_PATH', '')
    run = subprocess.run([str(original), '--gtest_filter=DOMPhysics.DefaultConstruction', '--gtest_output=xml:' + str(report)], env=env, text=True, capture_output=True, check=True)
    (output.parent / 'original.log').write_text(run.stdout + run.stderr)
    result = ET.parse(report).getroot()
    assert result.attrib['tests'] == '1' and result.attrib['failures'] == '0'
    proof = dict(upstream_sha=revision, corpus_sha256=sha(corpus), probe_source_sha256=sha(source),
                 probe_binary_sha256=sha(binary), linked_libraries_sha256=libraries,
                 source_sha256={path: sha(upstream / path) for path in files},
                 original_binary_sha256=sha(original), original_tests=1, original_failures=0,
                 explicit_gap_ids=[case['id'] for case in cases if 'gap' in case], observations=observations)
    output.write_text(json.dumps(proof, indent=2) + '\n')
    print(f'{len(observations)} native physics rows and original DOMPhysics.DefaultConstruction verified')


if __name__ == '__main__':
    main()
