#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Verify native SDF 1.4 axis-conversion scope and exact original case runs."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import xml.etree.ElementTree as ET
from merge_reference import convert


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--output', type=Path, default=Path('artifacts/version14/reference.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = root / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == json.loads((root / 'reference/inventory.json').read_text())['upstream_sha']
    source = root / 'tests/version14_reference.cc'
    binary = output.parent / 'version14-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    libdir = subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()
    subprocess.run(['c++', '-std=c++17', str(source), *flags, '-Wl,-rpath,' + libdir, '-o', str(binary)], check=True)
    corpus = root / 'tests/version14.json'
    observations = []
    for case in json.loads(corpus.read_text()):
        resource = output.parent / 'resource.sdf'
        resource.write_text(case.get('resource', ''))
        run = subprocess.run([str(binary), str(resource)], input=case['xml'], text=True, capture_output=True, check=True)
        tokens = run.stdout.split()
        accepted, model = tokens[0] == '0', convert(tokens)
        assert (accepted, model, run.stderr) == (case['accepted'], case['model'], case['stderr']), (case['id'], run.stdout, run.stderr)
        observations.append(dict(id=case['id'], accepted=accepted, model=model, stderr=run.stderr))
    files = ['src/parser.cc', 'src/Converter.cc', 'src/Param.cc', 'src/Joint.cc', 'src/JointAxis.cc',
             'sdf/1.5/1_4.convert', 'sdf/1.6/1_5.convert', 'sdf/1.7/1_6.convert',
             'test/integration/joint_axis_frame.cc']
    for path in files:
        expected = subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':' + path])
        assert hashlib.sha256(expected).hexdigest() == sha(upstream / path), path
    linked = subprocess.check_output(['ldd', str(binary)], text=True)
    native_library = next(Path(line.split('=>', 1)[1].split()[0]) for line in linked.splitlines() if line.strip().startswith('libsdformat.so.'))
    assert sha(native_library) == sha(upstream / 'build/lib' / native_library.name)
    sdk_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    sdk_headers = ['Root.hh', 'Model.hh', 'World.hh', 'Link.hh', 'Joint.hh', 'JointAxis.hh', 'SemanticPose.hh']
    for name in sdk_headers:
        expected = subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':include/sdf/' + name])
        assert hashlib.sha256(expected).hexdigest() == sha(sdk_include / 'sdf' / name), name
    math_source = upstream.parent / 'gz-math'
    math_revision = subprocess.check_output(['git', '-C', str(math_source), 'rev-parse', 'HEAD'], text=True).strip()
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], text=True).strip())
    math_headers = ['Vector3.hh', 'Quaternion.hh', 'Helpers.hh']
    for name in math_headers:
        expected = subprocess.check_output(['git', '-C', str(math_source), 'show', math_revision + ':include/gz/math/' + name])
        assert hashlib.sha256(expected).hexdigest() == sha(math_include / 'gz/math' / name), name
    original = upstream / 'build/bin/INTEGRATION_joint_axis_frame'
    report = output.parent / 'original.xml'
    env = dict(os.environ)
    env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(Path.home() / 'local/lib') + ':' + env.get('LD_LIBRARY_PATH', '')
    run = subprocess.run([str(original), '--gtest_filter=JointAxisFrame.Version_1_4_missing:JointAxisFrame.Version_1_5_missing', '--gtest_output=xml:' + str(report)], env=env, text=True, capture_output=True, check=True)
    (output.parent / 'original.log').write_text(run.stdout + run.stderr)
    result = ET.parse(report).getroot()
    assert result.attrib['tests'] == '2' and result.attrib['failures'] == '0'
    proof = dict(upstream_sha=revision, corpus_sha256=sha(corpus), probe_source_sha256=sha(source),
                 observer_codec_sha256=sha(root / 'tools/merge_reference.py'), probe_binary_sha256=sha(binary),
                 sdk_header_sha256={name: sha(sdk_include / 'sdf' / name) for name in sdk_headers},
                 math_upstream_sha=math_revision, math_header_sha256={name: sha(math_include / 'gz/math' / name) for name in math_headers},
                 native_library_sha256=sha(native_library), source_sha256={path: sha(upstream / path) for path in files},
                 original_binary_sha256=sha(original), original_tests=2, original_failures=0, observations=observations)
    output.write_text(json.dumps(proof, indent=2) + '\n')
    print(f'{len(observations)} native 1.4/1.5 conversion observations and two exact originals verified')


if __name__ == '__main__':
    main()
