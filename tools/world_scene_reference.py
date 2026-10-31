# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay the bounded Scene controls and unchanged original native cases."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
PIN = '1249805d9fceeff94ca1fe63e26ab9c051cea24e'

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def linkage(binary, env):
    result = {}
    for line in subprocess.check_output(['ldd', str(binary)], env=env, text=True).splitlines():
        words = line.split()
        if '=>' in words and len(words) > 2 and words[2].startswith('/'):
            path = Path(words[2]).resolve()
            assert path.name not in result
            result[path.name] = digest(path)
    assert any(name.startswith('libsdformat.so') for name in result)
    return result

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--write-provenance', action='store_true')
    args = parser.parse_args()
    upstream = args.upstream.resolve()
    assert subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip() == PIN
    subprocess.run(['git', '-C', str(upstream), 'diff', '--quiet', PIN, '--'], check=True)
    libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip())
    includedir = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    env = dict(os.environ, LD_LIBRARY_PATH=str(libdir))
    names = ['Scene', 'World', 'Root', 'Element']
    for name in names:
        assert digest(includedir / 'sdf' / (name + '.hh')) == digest(upstream / 'include/sdf' / (name + '.hh'))
    library = (libdir / 'libsdformat.so').resolve()
    assert digest(library) == digest(upstream / 'build/lib' / library.name)
    source_names = ['src/Scene.cc', 'src/Scene_TEST.cc', 'src/World.cc', 'test/integration/world_dom.cc',
            'src/Param.cc', 'src/parser.cc', 'sdf/1.12/scene.sdf', 'sdf/1.12/world.sdf', 'test/sdf/world_complete.sdf']
    originals = [('INTEGRATION_world_dom', 'DOMWorld.Load'), ('UNIT_Scene_TEST', 'DOMScene.Construction')]
    compiler = Path(shutil.which('c++')).resolve()
    protected = [compiler, library, *(upstream / name for name in source_names),
            *(includedir / 'sdf' / (name + '.hh') for name in names),
            *(upstream / 'build/bin' / name for name, _ in originals),
            ROOT / 'tests/world_scene_reference.cc', ROOT / 'tests/document_lights_reference.cc', ROOT / 'tests/fixtures/world_scene_original.sdf', ROOT / 'tests/world_scene.json']
    before = {str(path): digest(path) for path in protected}
    data = json.loads((ROOT / 'tests/world_scene.json').read_text())
    output = ROOT / 'artifacts/world-scene-native'
    output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='sdf-scene-native-') as temporary:
        binary = Path(temporary) / 'world-scene-reference'
        flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
        subprocess.run(['c++', '-std=c++17', '-Wall', '-Wextra', '-Werror', str(ROOT / 'tests/world_scene_reference.cc'), '-o', str(binary), *flags, '-Wl,-rpath,' + str(libdir)], check=True)
        libraries = linkage(binary, env)
        light_binary = Path(temporary) / 'document-lights-reference'
        subprocess.run(['c++', '-std=c++17', str(ROOT / 'tests/document_lights_reference.cc'), '-o', str(light_binary), *flags, '-Wl,-rpath,' + str(libdir)], check=True)
        light_libraries = linkage(light_binary, env)
        def check(command, expected, xml=None):
            result = subprocess.run(command, input=xml.encode() if xml is not None else None, capture_output=True, env=env)
            assert (result.returncode, result.stdout.decode(), result.stderr.decode()) == (expected['exit'], expected['stdout'], expected['stderr'])
        for row in data['cases']:
            check([str(binary)], row, row['xml'])
            if 'light_native' in row:
                check([str(light_binary), '--single-document'], row['light_native'], row['xml'])
        check([str(binary), '--constructor'], data['constructor'])
        check([str(binary), '--file', str(upstream / 'test/sdf/world_complete.sdf')], data['full_original'])
        projected = (ROOT / 'tests/fixtures/world_scene_original.sdf').read_text()
        result = subprocess.run([str(binary)], input=projected.encode(), capture_output=True, env=env, check=True)
        assert result.stdout.decode() == data['full_original']['stdout'] and result.stderr == b''
        full = (upstream / 'test/sdf/world_complete.sdf').read_text()
        start = full.index('    <scene>'); end = full.index('</scene>', start) + len('</scene>')
        assert full[start:end] in projected
        original_provenance = {}
        for name, case in originals:
            original = upstream / 'build/bin' / name
            original_provenance[name] = {'binary_sha256': digest(original), 'libraries': linkage(original, env)}
            result = subprocess.run([str(original), '--gtest_filter=' + case, '--gtest_output=xml:' + str(output / (name + '.xml'))], capture_output=True, env=env)
            (output / (name + '.stdout')).write_bytes(result.stdout); (output / (name + '.stderr')).write_bytes(result.stderr)
            assert result.returncode == 0 and b'[  PASSED  ] 1 test.' in result.stdout
        provenance = {'upstream_sha': PIN, 'native_sources': {name: digest(upstream / name) for name in source_names},
                'installed_headers': {name: digest(includedir / 'sdf' / (name + '.hh')) for name in names},
                'helper_sha256': digest(ROOT / 'tests/world_scene_reference.cc'), 'light_helper_sha256': digest(ROOT / 'tests/document_lights_reference.cc'), 'light_helper_libraries': light_libraries, 'native_libraries': libraries,
                'originals': original_provenance, 'fixture_sha256': digest(ROOT / 'tests/fixtures/world_scene_original.sdf'),
                'corpus_sha256': digest(ROOT / 'tests/world_scene.json')}
    assert before == {str(path): digest(path) for path in protected}, 'protected native inputs changed during replay'
    path = ROOT / 'reference/world-scene.json'
    if args.write_provenance:
        path.write_text(json.dumps(provenance, indent=2) + '\n')
    else:
        assert provenance == json.loads(path.read_text()), 'native source or linkage provenance changed'
    print(f"{len(data['cases'])} complete native controls, constructor, full/projected Scene equality and two unchanged originals passed")

if __name__ == '__main__':
    main()
