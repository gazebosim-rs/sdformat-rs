#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Reproduce link flags and the untouched double-pendulum original assertions."""
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
    parser.add_argument('--output', type=Path, default=Path('artifacts/link-properties/reference.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = root / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == json.loads((root / 'reference/inventory.json').read_text())['upstream_sha']
    source = root / 'tests/link_properties_reference.cc'
    binary = output.parent / 'link-properties-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    libdir = subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()
    subprocess.run(['c++', '-std=c++17', '-Wall', '-Wextra', '-Werror', str(source), *flags, '-Wl,-rpath,' + libdir, '-o', str(binary)], check=True)
    corpus = root / 'tests/link_properties.json'
    cases = json.loads(corpus.read_text())
    observations = []
    for case in cases:
        run = subprocess.run([str(binary)], input=case['xml'], text=True, capture_output=True, check=True)
        tokens = run.stdout.split()
        assert tokens == case['tokens'], (case['id'], run.stdout, run.stderr)
        observations.append(dict(id=case['id'], tokens=tokens))
    files = ['src/Link.cc', 'src/Model.cc', 'src/Joint.cc', 'src/Param.cc',
             'include/sdf/Link.hh', 'include/sdf/Model.hh', 'include/sdf/Joint.hh',
             'sdf/1.12/link.sdf', 'sdf/1.4/link.sdf', 'test/integration/link_dom.cc',
             'test/integration/joint_dom.cc', 'test/sdf/double_pendulum.sdf']
    for path in files:
        original = subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':' + path])
        assert hashlib.sha256(original).hexdigest() == sha(upstream / path), path
    include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    for name in ['Link.hh', 'Model.hh', 'Joint.hh']:
        assert sha(include / 'sdf' / name) == sha(upstream / 'include/sdf' / name)
    libraries = {}
    for line in subprocess.check_output(['ldd', str(binary)], text=True).splitlines():
        if '=>' in line and any(line.strip().startswith(name) for name in ('libsdformat.so.', 'libgz-math.so.', 'libstdc++.so.', 'libc.so.')):
            path = Path(line.split('=>', 1)[1].split()[0])
            libraries[path.name] = sha(path)
            if path.name.startswith('libsdformat.so.'):
                assert sha(path) == sha(upstream / 'build/lib' / path.name)
    fixture = root / 'tests/fixtures/double_pendulum.sdf'
    assert sha(fixture) == sha(upstream / 'test/sdf/double_pendulum.sdf')
    env = dict(os.environ)
    env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(Path.home() / 'local/lib') + ':' + env.get('LD_LIBRARY_PATH', '')
    originals = []
    for stem, case in [('link', 'DOMLink.InertialDoublePendulum'), ('joint', 'DOMJoint.DoublePendulum')]:
        original = upstream / f'build/bin/INTEGRATION_{stem}_dom'
        report = output.parent / (stem + '-original.xml')
        run = subprocess.run([str(original), '--gtest_filter=' + case, '--gtest_output=xml:' + str(report)], env=env, text=True, capture_output=True, check=True)
        (output.parent / (stem + '-original.log')).write_text(run.stdout + run.stderr)
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
                 fixture_sha256=sha(fixture), originals=originals, math_upstream_sha=math_revision,
                 math_header_sha256={path: sha(math_source / path) for path in math_files},
                 observations=observations)
    output.write_text(json.dumps(proof, indent=2) + '\n')
    print(f'{len(observations)} native link flag rows and both untouched pendulum originals verified')


if __name__ == '__main__':
    main()
