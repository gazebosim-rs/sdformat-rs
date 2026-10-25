#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Reproduce scoped DOM lookups and unchanged model DOM original assertions."""
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
    parser.add_argument('--output', type=Path, default=Path('artifacts/model-dom/reference.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = root / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == json.loads((root / 'reference/inventory.json').read_text())['upstream_sha']
    source = root / 'tests/model_dom_reference.cc'
    binary = output.parent / 'model-dom-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    libdir = subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()
    subprocess.run(['c++', '-std=c++17', '-Wall', '-Wextra', '-Werror', str(source), *flags, '-Wl,-rpath,' + libdir, '-o', str(binary)], check=True)
    corpus = root / 'tests/model_dom.json'
    cases = json.loads(corpus.read_text())
    observations = []
    for case in cases:
        fixture = root / 'tests/fixtures' / case['fixture']
        run = subprocess.run([str(binary), case['scope'], case['kind'], case['name']],
                             input=fixture.read_text(), text=True, capture_output=True, check=True)
        assert run.stdout == case['path'] + '\n', (case['id'], run.stdout, run.stderr)
        observations.append(dict(id=case['id'], path=case['path']))
    files = ['src/Model.cc', 'src/FrameSemantics.cc', 'src/parser.cc',
             'include/sdf/Model.hh', 'include/sdf/Link.hh', 'sdf/1.6/model.sdf',
             'test/integration/model_dom.cc', 'test/sdf/double_pendulum.sdf',
             'test/sdf/nested_model.sdf']
    for path in files:
        original = subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':' + path])
        assert hashlib.sha256(original).hexdigest() == sha(upstream / path), path
    include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    for name in ['Link.hh', 'Model.hh']:
        assert sha(include / 'sdf' / name) == sha(upstream / 'include/sdf' / name)
    libraries = {}
    for line in subprocess.check_output(['ldd', str(binary)], text=True).splitlines():
        if '=>' in line and any(line.strip().startswith(name) for name in ('libsdformat.so.', 'libgz-math.so.', 'libstdc++.so.', 'libc.so.')):
            path = Path(line.split('=>', 1)[1].split()[0])
            libraries[path.name] = sha(path)
            if path.name.startswith('libsdformat.so.'):
                assert sha(path) == sha(upstream / 'build/lib' / path.name)
    fixtures = ['double_pendulum.sdf', 'nested_model.sdf']
    for name in fixtures:
        assert sha(root / 'tests/fixtures' / name) == sha(upstream / 'test/sdf' / name)
    env = dict(os.environ)
    env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(Path.home() / 'local/lib') + ':' + env.get('LD_LIBRARY_PATH', '')
    originals = []
    for case in ['DOMRoot.LoadDoublePendulum', 'DOMRoot.NestedModel']:
        original = upstream / 'build/bin/INTEGRATION_model_dom'
        report = output.parent / (case + '-original.xml')
        run = subprocess.run([str(original), '--gtest_filter=' + case, '--gtest_output=xml:' + str(report)], env=env, text=True, capture_output=True, check=True)
        (output.parent / (case + '-original.log')).write_text(run.stdout + run.stderr)
        result = ET.parse(report).getroot()
        assert result.attrib['tests'] == '1' and result.attrib['failures'] == '0'
        originals.append(dict(case=case, binary_sha256=sha(original), tests=1, failures=0))
    math_source = upstream.parent / 'gz-math'
    math_revision = subprocess.check_output(['git', '-C', str(math_source), 'rev-parse', 'HEAD'], text=True).strip()
    math_files = ['include/gz/math/' + name for name in ['Pose3.hh', 'Quaternion.hh', 'Vector3.hh', 'MassMatrix3.hh']]
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], text=True).strip())
    for path in math_files:
        expected = subprocess.check_output(['git', '-C', str(math_source), 'show', math_revision + ':' + path])
        assert hashlib.sha256(expected).hexdigest() == sha(math_source / path), path
        assert sha(math_include / 'gz/math' / Path(path).name) == sha(math_source / path), path
    proof = dict(upstream_sha=revision, corpus_sha256=sha(corpus), probe_source_sha256=sha(source),
                 probe_binary_sha256=sha(binary), linked_libraries_sha256=libraries,
                 source_sha256={path: sha(upstream / path) for path in files},
                 fixture_sha256={name: sha(root / 'tests/fixtures' / name) for name in fixtures}, originals=originals, math_upstream_sha=math_revision,
                 math_header_sha256={path: sha(math_source / path) for path in math_files},
                 observations=observations)
    output.write_text(json.dumps(proof, indent=2) + '\n')
    print(f'{len(observations)} native scoped-lookup rows and both untouched model originals verified')


if __name__ == '__main__':
    main()
