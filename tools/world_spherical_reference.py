# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay spherical reference metadata against the pinned native DOM."""
import argparse
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import tempfile

from world_scene_reference import PIN, digest, linkage

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--write-provenance', action='store_true')
    args = parser.parse_args()
    upstream = args.upstream.resolve()
    assert subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip() == PIN
    subprocess.run(['git', '-C', str(upstream), 'diff', '--quiet', PIN, '--'], check=True)
    env = dict(os.environ)
    for name in ['LD_PRELOAD', 'LD_AUDIT', 'LD_LIBRARY_PATH']:
        env.pop(name, None)
    libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip())
    includedir = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    env['LD_LIBRARY_PATH'] = str(libdir)
    library = (libdir / 'libsdformat.so').resolve()
    assert digest(library) == digest(upstream / 'build/lib' / library.name)
    headers = [includedir / 'sdf' / (name + '.hh') for name in ['World', 'Root']]
    for header in headers:
        assert digest(header) == digest(upstream / 'include/sdf' / header.name)
    sources = [upstream / name for name in [
        'src/World.cc', 'src/Param.cc', 'src/parser.cc',
        'test/integration/world_dom.cc', 'test/sdf/world_complete.sdf',
        *[f'sdf/1.{v}/spherical_coordinates.sdf' for v in range(4, 13)],
    ]]
    helper = ROOT / 'tests/world_spherical_reference.cc'
    corpus = ROOT / 'tests/world_spherical.json'
    fixture = ROOT / 'tests/fixtures/world_spherical_original.sdf'
    original = upstream / 'build/bin/INTEGRATION_world_dom'
    compiler = Path(shutil.which('c++')).resolve()
    math = upstream.parent / 'gz-math'
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], text=True).strip())
    math_library = (libdir / 'libgz-math.so').resolve()
    assert digest(math_library) == digest(math / 'build/lib' / math_library.name)
    math_headers = [math_include / 'gz/math' / (name + '.hh') for name in ['Angle', 'SphericalCoordinates']]
    for header in math_headers:
        assert digest(header) == digest(math / 'include/gz/math' / header.name)
    math_sources = [math / 'src' / (name + '.cc') for name in ['Angle', 'SphericalCoordinates']]
    protected = [compiler, library, helper, corpus, fixture, original, *headers, *sources, math_library, *math_headers, *math_sources]
    before = {str(path): digest(path) for path in protected}
    data = json.loads(corpus.read_text())
    with tempfile.TemporaryDirectory(prefix='sdf-spherical-native-') as temporary:
        binary = Path(temporary) / 'spherical-reference'
        flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
        subprocess.run([str(compiler), '-std=c++17', '-Wall', '-Wextra', '-Werror', str(helper), '-o', str(binary), *flags, '-Wl,-rpath,' + str(libdir)], check=True, env=env)
        libraries = linkage(binary, env)
        for row in data['cases']:
            result = subprocess.run([str(binary)], input=row['xml'], capture_output=True, text=True, env=env)
            assert (result.returncode, result.stdout, result.stderr) == (row['exit'], row['stdout'], row['stderr']), (row['name'], result.returncode, result.stdout, result.stderr)
        full = (upstream / 'test/sdf/world_complete.sdf').read_text()
        start = full.index('    <spherical_coordinates>')
        end = full.index('</spherical_coordinates>', start) + len('</spherical_coordinates>')
        assert full[start:end] in fixture.read_text()
        assert fixture.read_text() == data['original_projection']['xml']
        for name, xml in [('original_projection', fixture.read_text()), ('full_original', full)]:
            result = subprocess.run([str(binary)], input=xml, capture_output=True, text=True, env=env)
            row = data[name]
            assert (result.returncode, result.stdout, result.stderr) == (row['exit'], row['stdout'], row['stderr'])
        assert data['original_projection']['stdout'] == data['full_original']['stdout']
        constructor = subprocess.run([str(binary), '--ctor'], capture_output=True, text=True, env=env, check=True)
        assert constructor.stdout == 'COORDS 0\n' and not constructor.stderr
        result = subprocess.run([str(original), '--gtest_filter=DOMWorld.Load'], capture_output=True, env=env)
        assert result.returncode == 0 and b'[  PASSED  ] 1 test.' in result.stdout
        provenance = {
            'upstream_sha': PIN,
            'math_sha': subprocess.check_output(['git', '-C', str(math), 'rev-parse', 'HEAD'], text=True).strip(),
            'math_sources': {str(path.relative_to(math)): digest(path) for path in math_sources},
            'math_headers': {path.name: digest(path) for path in math_headers},
            'native_sources': {str(path.relative_to(upstream)): digest(path) for path in sources},
            'installed_headers': {path.name: digest(path) for path in headers},
            'compiler_sha256': digest(compiler), 'native_libraries': libraries,
            'helper_sha256': digest(helper), 'corpus_sha256': digest(corpus),
            'fixture_sha256': digest(fixture),
            'original_sha256': digest(original), 'original_libraries': linkage(original, env),
        }
    assert before == {str(path): digest(path) for path in protected}, 'native inputs changed during replay'
    destination = ROOT / 'reference/world-spherical.json'
    if args.write_provenance:
        destination.write_text(json.dumps(provenance, indent=2) + '\n')
    else:
        assert provenance == json.loads(destination.read_text()), 'native provenance changed'
    print(f"{len(data['cases'])} native spherical controls, constructor and unchanged DOMWorld.Load passed")


if __name__ == '__main__':
    main()
