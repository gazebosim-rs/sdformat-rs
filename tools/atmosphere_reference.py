#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay pinned Atmosphere values and retain every differential observation."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import xml.etree.ElementTree as ET


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--output', type=Path, default=Path('artifacts/atmosphere/reference.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = root / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == json.loads((root / 'reference/inventory.json').read_text())['upstream_sha']
    paths = ['src/Atmosphere.cc', 'src/Atmosphere_TEST.cc', 'src/World.cc', 'src/Param.cc',
             'src/Utils.cc', 'include/sdf/Atmosphere.hh', 'include/sdf/World.hh',
             'include/sdf/Root.hh', 'sdf/1.12/atmosphere.sdf']
    for name in paths:
        data = subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':' + name])
        assert hashlib.sha256(data).hexdigest() == digest(upstream / name), name
    includedir = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    for name in ['Atmosphere.hh', 'World.hh', 'Root.hh']:
        assert digest(includedir / 'sdf' / name) == digest(upstream / 'include/sdf' / name)
    source = root / 'tests/atmosphere_reference.cc'
    binary = output.parent / 'atmosphere-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip())
    subprocess.run(['c++', '-std=c++17', '-Wall', '-Wextra', '-Werror', str(source), *flags,
                    '-Wl,-rpath,' + str(libdir), '-o', str(binary)], check=True)
    libraries = {}
    for line in subprocess.check_output(['ldd', str(binary)], text=True).splitlines():
        if '=>' in line and any(line.strip().startswith(p) for p in ['libsdformat.so.', 'libgz-math.so.', 'libstdc++.so.', 'libc.so.']):
            path = Path(line.split('=>', 1)[1].split()[0])
            libraries[path.name] = digest(path)
            if path.name.startswith('libsdformat.so.'):
                assert digest(path) == digest(upstream / 'build/lib' / path.name)
    assert any(name.startswith('libsdformat.so.') for name in libraries)
    corpus = root / 'tests/atmosphere.json'
    rows = json.loads(corpus.read_text())
    observations = []
    for row in rows:
        result = subprocess.run([str(binary)], input=row['xml'], text=True, capture_output=True, check=True)
        (output.parent / (row['id'] + '.stdout')).write_text(result.stdout)
        (output.parent / (row['id'] + '.stderr')).write_text(result.stderr)
        tokens = result.stdout.split()
        assert tokens == row['tokens'], (row['id'], result.stdout, result.stderr)
        assert row['native_accepted'] == (tokens[0] == '0')
        observations.append({'id': row['id'], 'tokens': tokens})
    originals = ['Construction', 'Set', 'MoveConstructor', 'CopyConstructor',
                 'MoveAssignment', 'CopyAssignment', 'CopyAssignmentAfterMove']
    original = upstream / 'build/bin/UNIT_Atmosphere_TEST'
    report = output.parent / 'original.xml'
    env = dict(os.environ)
    env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(libdir)
    run = subprocess.run([str(original), '--gtest_filter=' + ':'.join('DOMAtmosphere.' + x for x in originals),
                          '--gtest_output=xml:' + str(report)], env=env, text=True, capture_output=True, check=True)
    (output.parent / 'original.log').write_text(run.stdout + run.stderr)
    xml = ET.parse(report).getroot()
    assert xml.attrib['tests'] == '7' and xml.attrib['failures'] == '0' and xml.attrib['errors'] == '0'
    cases = xml.findall('.//testcase')
    assert len(cases) == 7 and {case.attrib['name'] for case in cases} == set(originals)
    assert all(case.attrib.get('status') == 'run' and case.find('skipped') is None for case in cases)
    proof = {'upstream_sha': revision, 'source_sha256': {p: digest(upstream / p) for p in paths},
             'probe_source_sha256': digest(source), 'probe_binary_sha256': digest(binary),
             'corpus_sha256': digest(corpus), 'linked_libraries_sha256': libraries,
             'original_binary_sha256': digest(original), 'original_xml_sha256': digest(report),
             'original_tests': 7, 'original_failures': 0,
             'explicit_gap_ids': [row['id'] for row in rows if 'gap_code' in row],
             'observations': observations,
             'scope': 'Metadata observations and seven fixed originals only; existing reference build, no clean-image or runtime atmosphere claim.'}
    output.write_text(json.dumps(proof, indent=2) + '\n')
    print(f'{len(rows)} native atmosphere observations and seven original cases verified')


if __name__ == '__main__':
    main()
