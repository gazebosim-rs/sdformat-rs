#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay native World wind/magnetic metadata and the exact old wind input."""
import argparse
import hashlib
import json
from pathlib import Path
import os
import re
import shlex
import subprocess


def project_fixture(text):
    prefix = text[:text.index('>', text.index('<world name="default"')) + 1]
    payload = ''.join(re.search(r'<' + tag + r'>.*?</' + tag + r'>', text, re.S).group()
                      for tag in ['wind', 'gravity', 'magnetic_field'])
    return prefix + payload + '</world></sdf>\n'


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = root / 'artifacts/world-environment-reference'
    output.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    reference = json.loads((root / 'reference/world-environment.json').read_text())
    assert subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip() == reference['upstream_sha']
    for path, expected in reference['native_sources'].items():
        assert sha(upstream / path) == expected, path
    math = upstream.parent / 'gz-math'
    assert subprocess.check_output(['git', '-C', str(math), 'rev-parse', 'HEAD'], text=True).strip() == reference['math_upstream_sha']
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], text=True).strip())
    for path, expected in reference['math_sources'].items():
        assert sha(math / path) == expected, path
        assert sha(math_include / path.removeprefix('include/')) == expected, path
    include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    for header in ['World.hh', 'Light.hh', 'Root.hh']:
        assert sha(include / 'sdf' / header) == sha(upstream / 'include/sdf' / header)
    libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()).resolve()
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    peers = []
    for source in ['tests/world_environment_reference.cc', 'tests/document_lights_reference.cc']:
        binary = output / Path(source).stem
        subprocess.run(['c++', '-std=c++17', str(root / source), *flags,
                        '-Wl,--disable-new-dtags', f'-Wl,-rpath,{libdir}', '-o', str(binary)], check=True)
        libraries = {}
        for line in subprocess.check_output(['ldd', str(binary)], text=True).splitlines():
            words = line.split()
            if '=>' in words and len(words) > 2 and Path(words[2]).is_file():
                path = Path(words[2]).resolve()
                libraries[path.name] = sha(path)
        assert libraries == reference['libraries']
        peers.append(binary)
    for path, expected in reference['fixtures'].items():
        assert sha(root / path) == expected, path
    data = json.loads((root / 'tests/world_environment.json').read_text())
    def query(binary, xml='', args=()):
        value = subprocess.run([str(binary), *args], input=xml, capture_output=True,
                               text=True, check=True, timeout=10)
        return dict(stdout=value.stdout, stderr=value.stderr)
    peer, light_peer = peers
    for row in data['cases']:
        assert query(peer, row['xml']) == row['native'], row['id']
        if 'light_native' in row:
            assert query(light_peer, row['xml'], ['--single-document']) == row['light_native']
    original = (upstream / 'test/sdf/world_complete.sdf').read_text()
    projection = project_fixture(original)
    assert projection.encode() == (root / 'tests/fixtures/world_environment_original.sdf').read_bytes()
    assert query(peer, original) == data['fixture']['full']
    assert query(peer, projection) == data['fixture']['projection'] == data['fixture']['full']
    assert query(peer, args=['--profiles']) == data['profiles']
    subprocess.run(['cargo', 'test', '--locked', '--test', 'world_environment', '--test', 'document_lights'], cwd=root, check=True)
    env = os.environ.copy()
    env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(libdir)
    value = subprocess.run([str(upstream / 'build/bin/INTEGRATION_world_dom'), '--gtest_filter=DOMWorld.Load'],
                           env=env, capture_output=True, text=True, check=True, timeout=30)
    assert '[  PASSED  ] 1 test.' in value.stdout, value.stdout
    assert not value.stderr, value.stderr
    (output / 'result.json').write_text(json.dumps(dict(native_cases=len(data['cases']),
        exact_old_input_native_positive=True, full_projection_equal=True,
        raw_constructor_profiles=True, unchanged_originals=1, libraries=libraries), indent=2) + '\n')
    print('Passed 35 native cases, exact old input, full/projection equality and unchanged DOMWorld.Load')


if __name__ == '__main__':
    main()
