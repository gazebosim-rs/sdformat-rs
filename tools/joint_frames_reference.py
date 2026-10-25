#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay nested model-frame aliases and four unchanged joint frame originals."""
import argparse
import hashlib
import json
import os
import shlex
from pathlib import Path
import subprocess
import xml.etree.ElementTree as ET


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--output', type=Path, default=Path('artifacts/joint-frames/reference.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = root / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == json.loads((root / 'reference/inventory.json').read_text())['upstream_sha']
    headers = ['Joint.hh', 'JointAxis.hh', 'Model.hh', 'SemanticPose.hh', 'Root.hh',
               'Frame.hh', 'Link.hh', 'Collision.hh', 'World.hh']
    files = ['src/Joint.cc', 'src/JointAxis.cc', 'src/Model.cc', 'src/FrameSemantics.cc',
             'src/FrameSemantics.hh', 'src/ScopedGraph.hh', 'src/SemanticPose.cc',
             'src/Frame.cc', 'src/Link.cc', 'src/Collision.cc', 'test/integration/joint_dom.cc']
    files += ['include/sdf/' + name for name in headers]
    fixtures = ['joint_parent_frame.sdf', 'joint_child_frame.sdf', 'model_joint_relative_to.sdf', 'joint_nested_parent_child.sdf']
    files += ['test/sdf/' + name for name in fixtures]
    for path in files:
        expected = subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':' + path])
        assert hashlib.sha256(expected).hexdigest() == sha(upstream / path), path
    for name in fixtures:
        assert sha(root / 'tests/fixtures' / name) == sha(upstream / 'test/sdf' / name)
    include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    for name in headers:
        assert sha(include / 'sdf' / name) == sha(upstream / 'include/sdf' / name)
    binary = upstream / 'build/bin/INTEGRATION_joint_dom'
    env = dict(os.environ)
    env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(Path.home() / 'local/lib') + ':' + env.get('LD_LIBRARY_PATH', '')
    env.pop('SDF_PATH', None)
    probe_source = root / 'tests/model_frame_alias_reference.cc'
    probe = output.parent / 'model-frame-alias-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], env=env, text=True))
    libdir = subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], env=env, text=True).strip()
    subprocess.run(['c++', '-std=c++17', '-Wall', '-Wextra', '-Werror', str(probe_source), *flags,
                    '-Wl,-rpath,' + libdir, '-o', str(probe)], env=env, check=True)
    corpus = root / 'tests/model_frame_alias.json'
    observations = []
    for row in json.loads(corpus.read_text()):
        run = subprocess.run([str(probe), row['kind']], input=row['xml'], text=True,
                             capture_output=True, check=True, env=env)
        assert run.stdout.split() == row['tokens'], (row['id'], run.stdout, run.stderr)
        observations.append(dict(id=row['id'], tokens=run.stdout.split()))
    originals = []
    for case in ['LoadJointParentFrame', 'LoadJointChildFrame', 'LoadJointPoseRelativeTo', 'LoadJointNestedParentChild']:
        report = output.parent / (case + '.xml')
        run = subprocess.run([str(binary), '--gtest_filter=DOMJoint.' + case, '--gtest_output=xml:' + str(report)], env=env, text=True, capture_output=True, check=True)
        (output.parent / (case + '.log')).write_text(run.stdout + run.stderr)
        result = ET.parse(report).getroot()
        assert result.attrib['tests'] == '1' and result.attrib['failures'] == '0'
        assert int(result.attrib.get('errors', '0')) == 0
        assert int(result.attrib.get('disabled', '0')) == 0
        cases = result.findall('.//testcase')
        assert len(cases) == 1 and cases[0].attrib['name'] == case
        assert cases[0].attrib.get('status') == 'run' and cases[0].find('skipped') is None
        originals.append(dict(case='DOMJoint.' + case, tests=1, failures=0, skipped=0))
    libraries = {}
    for line in subprocess.check_output(['ldd', str(binary)], env=env, text=True).splitlines():
        if '=>' in line and any(line.strip().startswith(name) for name in ('libsdformat.so.', 'libgz-math.so.', 'libstdc++.so.', 'libc.so.')):
            path = Path(line.split('=>', 1)[1].split()[0])
            libraries[path.name] = sha(path)
            if path.name.startswith('libsdformat.so.'):
                assert sha(path) == sha(upstream / 'build/lib' / path.name)
    math_source = upstream.parent / 'gz-math'
    math_revision = subprocess.check_output(['git', '-C', str(math_source), 'rev-parse', 'HEAD'], text=True).strip()
    math_files = ['include/gz/math/' + name for name in ['Pose3.hh', 'Quaternion.hh', 'Vector3.hh']]
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], text=True).strip())
    for path in math_files:
        expected = subprocess.check_output(['git', '-C', str(math_source), 'show', math_revision + ':' + path])
        assert hashlib.sha256(expected).hexdigest() == sha(math_source / path), path
        assert sha(math_include / 'gz/math' / Path(path).name) == sha(math_source / path), path
    proof = dict(upstream_sha=revision, corpus_sha256=sha(corpus), probe_source_sha256=sha(probe_source),
                 probe_binary_sha256=sha(probe), observations=observations, source_sha256={path: sha(upstream / path) for path in files},
                 fixture_sha256={name: sha(root / 'tests/fixtures' / name) for name in fixtures},
                 original_binary_sha256=sha(binary), linked_libraries_sha256=libraries,
                 math_upstream_sha=math_revision, math_header_sha256={path: sha(math_source / path) for path in math_files},
                 originals=originals, qualification='Existing pinned native test binary and installed SDK/library/header identity; no fresh full native rebuild or reproducible image claim.')
    output.write_text(json.dumps(proof, indent=2) + '\n')
    print('150 native model-frame observations and four unchanged joint originals verified')


if __name__ == '__main__':
    main()
