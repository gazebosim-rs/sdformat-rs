#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay three unchanged joint endpoint originals and bind fixture provenance."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import xml.etree.ElementTree as ET


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--output', type=Path, default=Path('artifacts/joint-endpoints/reference.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = root / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == json.loads((root / 'reference/inventory.json').read_text())['upstream_sha']
    files = ['src/Joint.cc', 'src/Model.cc', 'src/FrameSemantics.cc',
             'include/sdf/Joint.hh', 'include/sdf/Model.hh', 'include/sdf/SemanticPose.hh',
             'test/integration/joint_dom.cc']
    fixtures = ['joint_parent_world.sdf', 'joint_parent_model_frame.sdf', 'joint_child_model_frame.sdf']
    files += ['test/sdf/' + name for name in fixtures]
    for path in files:
        expected = subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':' + path])
        assert hashlib.sha256(expected).hexdigest() == sha(upstream / path), path
    for name in fixtures:
        assert sha(root / 'tests/fixtures' / name) == sha(upstream / 'test/sdf' / name)
    include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    for name in ['Joint.hh', 'Model.hh', 'SemanticPose.hh']:
        assert sha(include / 'sdf' / name) == sha(upstream / 'include/sdf' / name)
    binary = upstream / 'build/bin/INTEGRATION_joint_dom'
    env = dict(os.environ)
    env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(Path.home() / 'local/lib') + ':' + env.get('LD_LIBRARY_PATH', '')
    originals = []
    for case in ['LoadJointParentWorld', 'LoadJointParentModelFrame', 'LoadJointChildModelFrame']:
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
    proof = dict(upstream_sha=revision, source_sha256={path: sha(upstream / path) for path in files},
                 fixture_sha256={name: sha(root / 'tests/fixtures' / name) for name in fixtures},
                 original_binary_sha256=sha(binary), linked_libraries_sha256=libraries,
                 math_upstream_sha=math_revision, math_header_sha256={path: sha(math_source / path) for path in math_files},
                 originals=originals, qualification='Existing pinned native test binary and installed SDK/library/header identity; no fresh full native rebuild or reproducible image claim.')
    output.write_text(json.dumps(proof, indent=2) + '\n')
    print('Three unchanged native joint endpoint originals passed, each one test and zero failures/skips')


if __name__ == '__main__':
    main()
