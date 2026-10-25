#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay native Root noise types, exact malformed-input disposition and baselines."""
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
    parser.add_argument('--output', type=Path, default=Path('artifacts/noise-type/reference.json'))
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
    source = root / 'tests/noise_type_reference.cc'
    binary = output.parent / 'noise-type-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], env=env, text=True))
    libdir = subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], env=env, text=True).strip()
    subprocess.run(['c++', '-std=c++17', '-Wall', '-Wextra', '-Werror', str(source), *flags, '-Wl,-rpath,' + libdir, '-o', str(binary)], env=env, check=True)
    embedded = subprocess.check_output([str(binary), '--noise-spec'], env=env)
    assert embedded == b'\n' + (upstream / 'sdf/1.12/noise.sdf').read_bytes() + b'\n'
    corpus = root / 'tests/noise_type.json'
    cases = json.loads(corpus.read_text())
    observations = []
    for case in cases:
        run = subprocess.run([str(binary)], input=case['xml'], env=env, text=True, capture_output=True, check=True)
        assert run.stdout.split() == case['tokens'], (case['id'], run.stdout, run.stderr)
        observations.append(dict(id=case['id'], tokens=run.stdout.split()))
    disposition = json.loads((root / 'tests/noise_type_disposition.json').read_text())
    old = next(row for row in cases if row['id'] == 'former_supplementary_positive')
    canonical = next(row for row in cases if row['id'] == 'canonical_supplementary_positive')
    assert old['xml'] == disposition['exact_old_xml'] and old['tokens'][0] != '0'
    assert hashlib.sha256(old['xml'].encode()).hexdigest() == disposition['exact_old_xml_sha256']
    assert canonical['tokens'][0] == '0'
    assert canonical['xml'] == old['xml'].replace(disposition['old_fragment'], disposition['canonical_replacement'])
    rust = root / 'tests/imu.rs'
    assert sha(rust) == disposition['after_sha256']
    restored = rust.read_bytes().replace(disposition['canonical_replacement'].encode(), disposition['old_fragment'].encode())
    assert hashlib.sha256(restored).hexdigest() == disposition['before_sha256']
    files = ['src/Noise.cc', 'src/Noise_TEST.cc', 'src/Imu.cc', 'src/Sensor.cc', 'src/Types.cc', 'src/Utils.hh',
             'src/Param.cc', 'src/parser.cc', 'src/Root.cc', 'src/SDF.cc', 'sdf/embedSdf.py',
             'include/sdf/Noise.hh', 'include/sdf/Imu.hh', 'include/sdf/Sensor.hh', 'include/sdf/SDFImpl.hh',
             'include/sdf/Element.hh', 'include/sdf/Param.hh', 'include/sdf/Types.hh',
             'include/sdf/Root.hh', 'include/sdf/Model.hh', 'include/sdf/Link.hh', 'include/sdf/Error.hh',
             'test/integration/link_dom.cc', 'test/sdf/sensors.sdf']
    files += ['sdf/' + version + '/noise.sdf' for version in
              ['1.4', '1.5', '1.6', '1.7', '1.8', '1.9', '1.10', '1.11', '1.12']]
    for path in files:
        original = subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':' + path])
        assert hashlib.sha256(original).hexdigest() == sha(upstream / path), path
    include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    for name in ['Noise.hh', 'Imu.hh', 'Sensor.hh', 'SDFImpl.hh', 'Element.hh', 'Param.hh', 'Types.hh', 'Root.hh', 'Model.hh', 'Link.hh', 'Error.hh']:
        assert sha(include / 'sdf' / name) == sha(upstream / 'include/sdf' / name)
    libraries = {}
    for line in subprocess.check_output(['ldd', str(binary)], env=env, text=True).splitlines():
        if '=>' in line and any(line.strip().startswith(name) for name in ('libsdformat.so.', 'libgz-math.so.', 'libstdc++.so.', 'libc.so.')):
            path = Path(line.split('=>', 1)[1].split()[0])
            libraries[path.name] = sha(path)
            if path.name.startswith('libsdformat.so.'):
                assert sha(path) == sha(upstream / 'build/lib' / path.name)
    assert sha(root / 'tests/fixtures/sensors.sdf') == sha(upstream / 'test/sdf/sensors.sdf')
    original_env = dict(env)
    original_env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(Path.home() / 'local/lib') + ':' + env.get('LD_LIBRARY_PATH', '')
    originals = []
    for stem, case in [('UNIT_Noise_TEST', 'DOMNoise.Load'), ('INTEGRATION_link_dom', 'DOMLink.Sensors')]:
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
                 embedded_current_schema_sha256={'1.12/noise.sdf': hashlib.sha256(embedded).hexdigest()},
                 schema_environment='SDF_PATH removed; actual embedded noise schema matches pinned source and embedSdf.py newline framing',
                 unchanged_fixture_sha256=sha(root / 'tests/fixtures/sensors.sdf'),
                 disposition_sha256=sha(root / 'tests/noise_type_disposition.json'),
                 originals=originals, observations=observations,
                 qualification='Existing pinned native baseline executables/SDK libraries, freshly compiled probe; no new original case retirement or full native rebuild claim.')
    output.write_text(json.dumps(proof, indent=2) + '\n')
    print(f'{len(observations)} native noise type rows, exact disposition and both unchanged originals verified')


if __name__ == '__main__':
    main()
