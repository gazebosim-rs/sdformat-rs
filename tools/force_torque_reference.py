#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay typed ForceTorque / joint-sensor semantics and unchanged native cases."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import tempfile
import xml.etree.ElementTree as ET


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--output', type=Path, default=Path('artifacts/force-torque/reference.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = root / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == json.loads((root / 'reference/inventory.json').read_text())['upstream_sha']
    env = dict(os.environ)
    env.pop('SDF_PATH', None)
    source = root / 'tests/force_torque_reference.cc'
    binary = output.parent / 'force-torque-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], env=env, text=True))
    libdir = subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], env=env, text=True).strip()
    subprocess.run(['c++', '-std=c++17', '-Wall', '-Wextra', '-Werror', str(source), *flags, '-Wl,-rpath,' + libdir, '-o', str(binary)], env=env, check=True)
    embedded = {}
    for name in ['forcetorque.sdf', 'noise.sdf', 'sensor.sdf', 'joint.sdf']:
        text = subprocess.check_output([str(binary), '--spec', name], env=env)
        assert text == b'\n' + (upstream / 'sdf/1.12' / name).read_bytes() + b'\n', name
        embedded['1.12/' + name] = hashlib.sha256(text).hexdigest()
    corpus = root / 'tests/force_torque.json'
    cases = json.loads(corpus.read_text())
    observations = []
    with tempfile.TemporaryDirectory(prefix='force-torque-resource-', dir=output.parent) as temporary:
        resource = Path(temporary) / 'resource.sdf'
        for case in cases:
            command = [str(binary)]
            if 'resource' in case:
                resource.write_text(case['resource'])
                command.append(str(resource.resolve()))
            run = subprocess.run(command, input=case['xml'], env=env, text=True, capture_output=True, check=True)
            assert run.stdout.split() == case['tokens'], (case['id'], run.stdout, run.stderr)
            observations.append(dict(id=case['id'], tokens=run.stdout.split()))
    files = ['src/ForceTorque.cc', 'src/ForceTorque_TEST.cc', 'src/Joint.cc', 'src/Sensor.cc',
             'src/Imu.cc', 'src/Noise.cc', 'src/Types.cc', 'src/Utils.hh', 'src/Param.cc',
             'src/parser.cc', 'src/FrameSemantics.cc', 'src/SemanticPose.cc', 'src/SDF.cc',
             'sdf/embedSdf.py', 'test/integration/joint_dom.cc', 'test/integration/link_dom.cc',
             'test/sdf/joint_sensors.sdf', 'test/sdf/sensors.sdf']
    headers = ['ForceTorque.hh', 'Noise.hh', 'Imu.hh', 'Sensor.hh', 'Joint.hh', 'SDFImpl.hh',
               'Element.hh', 'Param.hh', 'Types.hh', 'Root.hh', 'Model.hh', 'Link.hh',
               'SemanticPose.hh', 'Error.hh']
    files += ['include/sdf/' + name for name in headers]
    files += ['sdf/' + version + '/' + name for version in
              ['1.4', '1.5', '1.6', '1.7', '1.8', '1.9', '1.10', '1.11', '1.12']
              for name in ['forcetorque.sdf', 'sensor.sdf', 'joint.sdf', 'noise.sdf']]
    for path in files:
        original = subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':' + path])
        assert hashlib.sha256(original).hexdigest() == sha(upstream / path), path
    include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    for name in headers:
        assert sha(include / 'sdf' / name) == sha(upstream / 'include/sdf' / name), name
    libraries = {}
    for line in subprocess.check_output(['ldd', str(binary)], env=env, text=True).splitlines():
        if '=>' in line and any(line.strip().startswith(name) for name in ('libsdformat.so.', 'libgz-math.so.', 'libstdc++.so.', 'libc.so.')):
            path = Path(line.split('=>', 1)[1].split()[0])
            libraries[path.name] = sha(path)
            if path.name.startswith('libsdformat.so.'):
                assert sha(path) == sha(upstream / 'build/lib' / path.name)
    fixtures = {}
    for name in ['joint_sensors.sdf', 'sensors.sdf']:
        fixtures[name] = sha(root / 'tests/fixtures' / name)
        assert fixtures[name] == sha(upstream / 'test/sdf' / name)
    original_env = dict(env)
    original_env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(Path.home() / 'local/lib') + ':' + env.get('LD_LIBRARY_PATH', '')
    originals = []
    for stem, case in [('INTEGRATION_joint_dom', 'DOMJoint.Sensors'),
                       ('INTEGRATION_link_dom', 'DOMLink.Sensors'), ('UNIT_Noise_TEST', 'DOMNoise.Load')]:
        original = upstream / 'build/bin' / stem
        report = output.parent / (stem + '.xml')
        run = subprocess.run([str(original), '--gtest_filter=' + case, '--gtest_output=xml:' + str(report)], env=original_env, text=True, capture_output=True, check=True)
        (output.parent / (stem + '.log')).write_text(run.stdout + run.stderr)
        result = ET.parse(report).getroot()
        assert result.attrib['tests'] == '1' and result.attrib['failures'] == '0' and int(result.attrib.get('errors', '0')) == 0
        actual = result.findall('.//testcase')
        assert len(actual) == 1 and actual[0].attrib['status'] == 'run' and actual[0].find('skipped') is None
        originals.append(dict(case=case, binary_sha256=sha(original), tests=1, failures=0, skipped=0))
    proof = dict(upstream_sha=revision, corpus_sha256=sha(corpus), probe_source_sha256=sha(source),
                 probe_binary_sha256=sha(binary), linked_libraries_sha256=libraries,
                 source_sha256={path: sha(upstream / path) for path in files},
                 embedded_current_schema_sha256=embedded,
                 schema_environment='SDF_PATH removed; actual embedded schemas match pinned source and embedSdf.py newline framing',
                 unchanged_fixture_sha256=fixtures, originals=originals, observations=observations,
                 qualification='Existing pinned native executables/SDK libraries and freshly compiled probe; no full native rebuild, mutable Element facade, runtime sensor or exact diagnostic parity claim.')
    output.write_text(json.dumps(proof, indent=2) + '\n')
    print(f'{len(observations)} native ForceTorque/joint sensor rows and three unchanged originals verified')


if __name__ == '__main__':
    main()
