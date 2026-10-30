#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay supported primitive geometry metadata and actual XML exchanges."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = root / 'artifacts/geometry-xml-reference'
    output.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    reference = json.loads((root / 'reference/geometry-xml.json').read_text())
    assert subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip() == reference['upstream_sha']
    for path, expected in reference['native_sources'].items():
        assert sha(upstream / path) == expected, path
    include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    assert sha(include / 'sdf/Geometry.hh') == sha(upstream / 'include/sdf/Geometry.hh')
    math = upstream.parent / 'gz-math'
    assert subprocess.check_output(['git', '-C', str(math), 'rev-parse', 'HEAD'], text=True).strip() == reference['math_upstream_sha']
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], text=True).strip())
    for path, expected in reference['math_sources'].items():
        assert sha(math / path) == expected, path
        assert sha(math_include / path.removeprefix('include/')) == expected, path
    libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()).resolve()
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    binary = output / 'geometry-xml-reference'
    subprocess.run(['c++', '-std=c++17', str(root / 'tests/geometry_xml_reference.cc'), *flags,
                    '-Wl,--disable-new-dtags', f'-Wl,-rpath,{libdir}', '-o', str(binary)], check=True)
    libraries = {}
    for line in subprocess.check_output(['ldd', str(binary)], text=True).splitlines():
        words = line.split()
        if '=>' in words and len(words) > 2 and Path(words[2]).is_file():
            path = Path(words[2]).resolve(); libraries[path.name] = sha(path)
    assert libraries == reference['libraries']
    def query(xml='', arguments=()):
        value = subprocess.run([str(binary), *arguments], input=xml, capture_output=True,
                               text=True, check=True, timeout=10)
        return dict(stdout=value.stdout, stderr=value.stderr)
    data = json.loads((root / 'tests/geometry_xml.json').read_text())
    for kind in reference['supported']:
        fixture = root / 'tests/fixtures' / ('geometry_' + kind + '_native.xml')
        generated = query(arguments=[kind]); assert not generated['stderr']
        assert generated['stdout'].encode() == fixture.read_bytes()
        assert sha(fixture) == reference['fixtures'][str(fixture.relative_to(root))]
        assert next(row['xml'] for row in data['rows'] if row['id'] == 'default-' + kind) == fixture.read_text()
    for row in data['rows']: assert query(row['xml']) == row['native'], row['id']
    assert query(arguments=['--plane-profiles']) == data['plane_profiles']
    exports = output / 'exports'
    if exports.exists():
        for path in exports.glob('*.xml'): path.unlink()
    env = dict(os.environ, SDF_GEOMETRY_XML_EXPORT=str(exports))
    subprocess.run(['cargo', 'test', '--locked', '--test', 'geometry_xml', '--test', 'geometry_xml_original'],
                   cwd=root, env=env, check=True)
    expected = {row['id'] + '.xml' for row in data['rows'] if row['export']}
    expected.update(f'edited-plane-{index}.xml' for index in range(6))
    assert {p.name for p in exports.glob('*.xml')} == expected
    for row in data['rows']:
        if row['export']:
            observed = query((exports / (row['id'] + '.xml')).read_text())
            assert not observed['stderr'] and observed['stdout'] == row['native']['stdout'], row['id']
    profiles = data['plane_profiles']['stdout'].splitlines()
    for index in range(6):
        observed = query((exports / f'edited-plane-{index}.xml').read_text())
        assert observed == dict(stdout='READ 1\nLOAD\n' + profiles[2*index] + '\n', stderr='')
    env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(libdir)
    for case in data['original_stderr']:
        value = subprocess.run([str(upstream / 'build/bin/UNIT_Geometry_TEST'), '--gtest_filter=' + case['case']],
            env=env, capture_output=True, text=True, check=True, timeout=30)
        assert '[  PASSED  ] 1 test.' in value.stdout, value.stdout
        assert value.stderr == case['stderr'], case['case']
    (output / 'result.json').write_text(json.dumps(dict(native_cases=len(data['rows']),
        actual_rust_to_native_exchanges=len(expected), unchanged_originals=2,
        exact_native_fixtures=7, native_plane_setter_load_profiles=6, libraries=libraries), indent=2) + '\n')
    print(f'Passed {len(data["rows"])} native cases, {len(expected)} actual XML exchanges and both unchanged originals')


if __name__ == '__main__': main()
