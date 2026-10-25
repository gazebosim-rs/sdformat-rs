#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Reproduce Root-loaded axis limits and unchanged construction baseline."""
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
    parser.add_argument('--output', type=Path, default=Path('artifacts/axis-limits/reference.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = root / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == json.loads((root / 'reference/inventory.json').read_text())['upstream_sha']
    source = root / 'tests/axis_limits_reference.cc'
    binary = output.parent / 'axis-limits-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    libdir = subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()
    subprocess.run(['c++', '-std=c++17', '-Wall', '-Wextra', '-Werror', str(source), *flags, '-Wl,-rpath,' + libdir, '-o', str(binary)], check=True)
    corpus = root / 'tests/axis_limits.json'
    cases = json.loads(corpus.read_text())
    native_env = dict(os.environ)
    native_env.pop('SDF_PATH', None)
    embedded = subprocess.check_output([str(binary), '--joint-spec'], env=native_env)
    # embedSdf.py frames each raw string literal with one newline at each end.
    assert embedded == b'\n' + (upstream / 'sdf/1.12/joint.sdf').read_bytes() + b'\n'
    observations = []
    for case in cases:
        run = subprocess.run([str(binary)], input=case['xml'], env=native_env, text=True, capture_output=True, check=True)
        tokens = run.stdout.split()
        assert tokens == case['tokens'], (case['id'], run.stdout, run.stderr)
        observations.append(dict(id=case['id'], tokens=tokens))
    files = ['src/JointAxis.cc', 'src/Param.cc', 'src/JointAxis_TEST.cc', 'src/parser.cc',
             'src/Root.cc', 'src/Joint.cc', 'src/SDF.cc', 'sdf/embedSdf.py', 'include/sdf/JointAxis.hh',
             'include/sdf/Root.hh', 'include/sdf/SDFImpl.hh']
    files += ['sdf/' + version + '/joint.sdf' for version in
              ['1.4', '1.5', '1.6', '1.7', '1.8', '1.9', '1.10', '1.11', '1.12']]
    for path in files:
        original = subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':' + path])
        assert hashlib.sha256(original).hexdigest() == sha(upstream / path), path
    include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    for name in ['JointAxis.hh', 'Root.hh', 'SDFImpl.hh']:
        assert sha(include / 'sdf' / name) == sha(upstream / 'include/sdf' / name)
    libraries = {}
    for line in subprocess.check_output(['ldd', str(binary)], text=True).splitlines():
        if '=>' in line and any(line.strip().startswith(name) for name in ('libsdformat.so.', 'libgz-math.so.', 'libstdc++.so.', 'libc.so.')):
            path = Path(line.split('=>', 1)[1].split()[0])
            libraries[path.name] = sha(path)
            if path.name.startswith('libsdformat.so.'):
                assert sha(path) == sha(upstream / 'build/lib' / path.name)
    original = upstream / 'build/bin/UNIT_JointAxis_TEST'
    report = output.parent / 'original.xml'
    env = native_env.copy()
    env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(Path.home() / 'local/lib') + ':' + env.get('LD_LIBRARY_PATH', '')
    run = subprocess.run([str(original), '--gtest_filter=DOMJointAxis.Construction', '--gtest_output=xml:' + str(report)], env=env, text=True, capture_output=True, check=True)
    (output.parent / 'original.log').write_text(run.stdout + run.stderr)
    result = ET.parse(report).getroot()
    assert result.attrib['tests'] == '1' and result.attrib['failures'] == '0'
    proof = dict(upstream_sha=revision, corpus_sha256=sha(corpus), probe_source_sha256=sha(source),
                 probe_binary_sha256=sha(binary), linked_libraries_sha256=libraries,
                 source_sha256={path: sha(upstream / path) for path in files},
                 embedded_current_schema_sha256={'1.12/joint.sdf': hashlib.sha256(embedded).hexdigest()},
                 schema_environment='SDF_PATH removed; runtime SDF::EmbeddedSpec matches pinned 1.12 joint source with exact embedSdf.py newline framing',
                 original_binary_sha256=sha(original), original_tests=1, original_failures=0,
                 explicit_gap_ids=[case['id'] for case in cases if 'gap' in case], observations=observations)
    output.write_text(json.dumps(proof, indent=2) + '\n')
    print(f'{len(observations)} native axis limit rows and original DOMJointAxis.Construction verified')


if __name__ == '__main__':
    main()
