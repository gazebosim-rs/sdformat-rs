#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay native BOM acceptance/metadata and bind tokenizer offset provenance."""
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
    parser.add_argument('--output', type=Path, default=Path('artifacts/utf8-bom/reference.json'))
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
    source = root / 'tests/utf8_bom_reference.cc'
    binary = output.parent / 'utf8-bom-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], env=env, text=True))
    libdir = subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], env=env, text=True).strip()
    subprocess.run(['c++', '-std=c++17', '-Wall', '-Wextra', '-Werror', str(source), *flags, '-Wl,-rpath,' + libdir, '-o', str(binary)], env=env, check=True)
    corpus = root / 'tests/utf8_bom.json'
    observations = []
    cases = json.loads(corpus.read_text())
    for row in cases:
        run = subprocess.run([str(binary)], input=row['xml'], env=env, text=True, capture_output=True, check=True)
        assert run.stdout.split() == row['tokens'] and row['tokens'][0] == '0', (row['id'], run.stdout, run.stderr)
        observations.append(dict(id=row['id'], tokens=run.stdout.split()))
    # Each supported-version/name triple is identical with/without BOM/declaration.
    for version in ['1.4', '1.5', '1.6', '1.7', '1.8', '1.9', '1.10', '1.11', '1.12']:
        for name in range(3):
            tokens = [next(row['tokens'] for row in cases if row['id'] == f'{version}_{name}_{prefix}') for prefix in range(3)]
            assert tokens[0] == tokens[1] == tokens[2]
    headers = ['Root.hh', 'World.hh', 'Model.hh', 'Link.hh', 'Joint.hh', 'JointAxis.hh',
               'Sensor.hh', 'ForceTorque.hh', 'Physics.hh', 'Plugin.hh', 'Element.hh']
    files = ['src/Root.cc', 'src/SDF.cc', 'src/parser.cc', 'src/Param.cc', 'src/Plugin.cc',
             'src/Sensor.cc', 'src/ForceTorque.cc', 'test/integration/model_dom.cc',
             'test/integration/joint_dom.cc', 'src/Physics_TEST.cc',
             'test/sdf/world_complete.sdf', 'test/sdf/joint_sensors.sdf']
    files += ['include/sdf/' + name for name in headers]
    for path in files:
        expected = subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':' + path])
        assert hashlib.sha256(expected).hexdigest() == sha(upstream / path), path
    include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    for name in headers:
        assert sha(include / 'sdf' / name) == sha(upstream / 'include/sdf' / name), name
    libraries = {}
    for line in subprocess.check_output(['ldd', str(binary)], env=env, text=True).splitlines():
        if '=>' in line and any(line.strip().startswith(name) for name in ('libsdformat.so.', 'libgz-math.so.', 'libtinyxml2.so.', 'libstdc++.so.', 'libc.so.')):
            path = Path(line.split('=>', 1)[1].split()[0])
            libraries[path.name] = sha(path)
            if path.name.startswith('libsdformat.so.'):
                assert sha(path) == sha(upstream / 'build/lib' / path.name)
    metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--offline', '--format-version=1'], cwd=root, text=True))
    dependency = {}
    for name, paths in [('quick-xml', ['src/reader/mod.rs', 'src/reader/slice_reader.rs']),
                        ('xmlparser', ['src/stream.rs'])]:
        package = next(p for p in metadata['packages'] if p['name'] == name)
        directory = Path(package['manifest_path']).parent
        dependency[name] = dict(version=package['version'], source_sha256={p: sha(directory / p) for p in paths})
    original_env = dict(env)
    original_env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(Path.home() / 'local/lib') + ':' + env.get('LD_LIBRARY_PATH', '')
    originals = []
    for stem, case in [('INTEGRATION_model_dom', 'DOMModel.ModelPlugins'),
                       ('INTEGRATION_joint_dom', 'DOMJoint.Sensors'), ('UNIT_Physics_TEST', 'DOMPhysics.DefaultConstruction')]:
        original = upstream / 'build/bin' / stem
        report = output.parent / (stem + '.xml')
        run = subprocess.run([str(original), '--gtest_filter=' + case, '--gtest_output=xml:' + str(report)], env=original_env, text=True, capture_output=True, check=True)
        (output.parent / (stem + '.log')).write_text(run.stdout + run.stderr)
        result = ET.parse(report).getroot()
        assert result.attrib['tests'] == '1' and result.attrib['failures'] == '0' and int(result.attrib.get('errors', '0')) == 0
        actual = result.findall('.//testcase')
        assert len(actual) == 1 and actual[0].attrib['status'] == 'run' and actual[0].find('skipped') is None
        originals.append(dict(case=case, binary_sha256=sha(original), tests=1, failures=0, skipped=0))
    tinyxml_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'tinyxml2'], text=True).strip())
    proof = dict(native_tinyxml2_header_sha256=sha(tinyxml_include / 'tinyxml2.h'),
                 native_tinyxml2_version=subprocess.check_output(['pkg-config', '--modversion', 'tinyxml2'], text=True).strip(),
                 upstream_sha=revision, corpus_sha256=sha(corpus), probe_source_sha256=sha(source),
                 probe_binary_sha256=sha(binary), linked_libraries_sha256=libraries,
                 source_sha256={p: sha(upstream / p) for p in files}, dependency_source=dependency,
                 originals=originals, observations=observations,
                 qualification='Fresh native probe and qualified existing SDK/baseline binaries. Raw-span/source-coordinate assertions are the owned original-source contract; no native diagnostic byte/column or normalized XML serialization parity claim. No new original mapping retirement.')
    output.write_text(json.dumps(proof, indent=2) + '\n')
    print(f'{len(observations)} native BOM observations and three unchanged originals verified')


if __name__ == '__main__':
    main()
