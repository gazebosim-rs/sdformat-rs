#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay native surface metadata, original fixtures and actual XML exchanges."""
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
    output = root / 'artifacts/surface-xml-reference'
    output.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    reference = json.loads((root / 'reference/surface-xml.json').read_text())
    assert subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip() == reference['upstream_sha']
    for path, expected in reference['native_sources'].items():
        assert sha(upstream / path) == expected, path
    include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    assert sha(include / 'sdf/Surface.hh') == sha(upstream / 'include/sdf/Surface.hh')
    math = upstream.parent / 'gz-math'
    assert subprocess.check_output(['git', '-C', str(math), 'rev-parse', 'HEAD'], text=True).strip() == reference['math_upstream_sha']
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], text=True).strip())
    for path, expected in reference['math_sources'].items():
        assert sha(math / path) == expected, path
        assert sha(math_include / path.removeprefix('include/')) == expected, path
    libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()).resolve()
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    binary = output / 'surface-xml-reference'
    subprocess.run(['c++', '-std=c++17', str(root / 'tests/surface_xml_reference.cc'), *flags,
                    '-Wl,--disable-new-dtags', f'-Wl,-rpath,{libdir}', '-o', str(binary)], check=True)
    libraries = {}
    for line in subprocess.check_output(['ldd', str(binary)], text=True).splitlines():
        words = line.split()
        if '=>' in words and len(words) > 2 and Path(words[2]).is_file():
            path = Path(words[2]).resolve()
            libraries[path.name] = sha(path)
    assert libraries == reference['libraries']
    for mode, name in [('--fixture', 'surface_to_element_native.xml'), ('--quiet-fixture', 'surface_to_element_quiet_native.xml')]:
        fixture = root / 'tests/fixtures' / name
        value = subprocess.run([str(binary), mode], capture_output=True, check=True, timeout=10)
        assert not value.stderr and value.stdout == fixture.read_bytes()
        assert sha(fixture) == reference['fixtures'][str(fixture.relative_to(root))]
    def query(xml):
        value = subprocess.run([str(binary)], input=xml, capture_output=True,
                               text=True, check=True, timeout=10)
        return dict(stdout=value.stdout, stderr=value.stderr)
    rows = json.loads((root / 'tests/surface_xml.json').read_text())
    for row in rows:
        assert query(row['xml']) == row['native'], row['id']
    exports = output / 'exports'
    if exports.exists():
        for path in exports.glob('*.xml'):
            path.unlink()
    env = dict(os.environ, SDF_SURFACE_XML_EXPORT=str(exports))
    subprocess.run(['cargo', 'test', '--locked', '--test', 'surface_xml', '--test', 'surface_xml_original'],
                   cwd=root, env=env, check=True)
    expected_files = {row['id'] + '.xml' for row in rows if row['export']}
    assert {p.name for p in exports.glob('*.xml')} == expected_files
    for row in rows:
        if row['export']:
            observed = query((exports / (row['id'] + '.xml')).read_text())
            assert not observed['stderr'] and observed['stdout'] == row['native']['stdout'], row['id']
    env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(libdir)
    value = subprocess.run([str(upstream / 'build/bin/UNIT_Surface_TEST'),
        '--gtest_filter=DOMsurface.ToElement:DOMsurface.ToElementErrorOutput'],
        env=env, capture_output=True, text=True, check=True, timeout=30)
    assert '[  PASSED  ] 2 tests.' in value.stdout, value.stdout
    assert not value.stderr, value.stderr
    (output / 'result.json').write_text(json.dumps(dict(native_cases=len(rows),
        actual_rust_to_native_exchanges=len(expected_files), unchanged_originals=2,
        exact_native_fixtures=2, libraries=libraries), indent=2) + '\n')
    print(f'Passed {len(rows)} native cases, {len(expected_files)} actual XML exchanges and both unchanged originals')


if __name__ == '__main__':
    main()
