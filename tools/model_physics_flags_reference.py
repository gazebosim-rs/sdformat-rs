# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay owned Model boolean metadata against the pinned native DOM."""
import argparse
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import tempfile
import xml.etree.ElementTree as ET

from verify import isolated_git_context
from world_scene_reference import PIN, digest, linkage

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--write-provenance', action='store_true')
    args = parser.parse_args()
    upstream = args.upstream.resolve()
    with isolated_git_context(ROOT):
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
    headers = [includedir / 'sdf' / (name + '.hh') for name in ['Root', 'Model', 'World', 'ParserConfig']]
    for header in headers:
        assert digest(header) == digest(upstream / 'include/sdf' / header.name)
    source_names = ['src/Model.cc', 'src/Model_TEST.cc', 'src/Root.cc', 'src/World.cc', 'src/Param.cc',
                    'src/parser.cc', *[f'sdf/1.{v}/model.sdf' for v in range(4, 13)]]
    helper = ROOT / 'tests/model_physics_flags_reference.cc'
    corpus = ROOT / 'tests/model_physics_flags.json'
    original = upstream / 'build/bin/UNIT_Model_TEST'
    compiler = Path(shutil.which('c++')).resolve()
    protected = [Path(__file__), compiler, library, helper, corpus, original, *headers,
                 *[upstream / name for name in source_names]]
    before = {str(path): digest(path) for path in protected}
    data = json.loads(corpus.read_text())
    with tempfile.TemporaryDirectory(prefix='sdf-model-flags-native-') as temporary:
        binary = Path(temporary) / 'model-flags-reference'
        flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
        subprocess.run([str(compiler), '-std=c++17', '-Wall', '-Wextra', '-Werror', str(helper), '-o', str(binary), *flags, '-Wl,-rpath,' + str(libdir)], check=True, env=env)
        libraries = linkage(binary, env)
        resource = Path(temporary) / 'resource.sdf'
        for row in data['cases']:
            command = [str(binary)]
            if 'resource' in row:
                resource.write_text(row['resource'])
                command.append(str(resource))
            result = subprocess.run(command, input=row['xml'], capture_output=True, text=True, env=env)
            assert (result.returncode, result.stdout, result.stderr) == (row['exit'], row['stdout'], row['stderr']), row['name']
        row = data['constructor']
        result = subprocess.run([str(binary), '--constructor'], capture_output=True, text=True, env=env)
        assert (result.returncode, result.stdout, result.stderr) == (row['exit'], row['stdout'], row['stderr'])
        output = ROOT / 'artifacts/model-physics-flags-native'
        output.mkdir(parents=True, exist_ok=True)
        result = subprocess.run([str(original), '--gtest_filter=DOMModel.Construction', '--gtest_output=xml:' + str(output / 'original.xml')], capture_output=True, env=env)
        (output / 'original.stdout').write_bytes(result.stdout)
        (output / 'original.stderr').write_bytes(result.stderr)
        report = ET.parse(output / 'original.xml').getroot()
        assert result.returncode == 0 and report.attrib['tests'] == '1' and report.attrib['failures'] == '0'
        assert report.attrib.get('disabled', '0') == '0' and report.attrib.get('errors', '0') == '0'
        cases = list(report.iter('testcase'))
        assert len(cases) == 1 and cases[0].attrib['result'] == 'completed' and not list(report.iter('skipped'))
        provenance = {
            'upstream_sha': PIN, 'native_sources': {name: digest(upstream / name) for name in source_names},
            'installed_headers': {path.name: digest(path) for path in headers},
            'compiler_sha256': digest(compiler), 'native_libraries': libraries,
            'helper_sha256': digest(helper), 'corpus_sha256': digest(corpus),
            'reference_script_sha256': digest(Path(__file__)),
            'original_sha256': digest(original), 'original_libraries': linkage(original, env),
        }
    assert before == {str(path): digest(path) for path in protected}, 'native inputs changed during replay'
    destination = ROOT / 'reference/model-physics-flags.json'
    if args.write_provenance:
        destination.write_text(json.dumps(provenance, indent=2) + '\n')
    else:
        assert provenance == json.loads(destination.read_text()), 'native provenance changed'
    print(f"{len(data['cases'])} Model flag controls, constructor/value-copy mutation and unchanged original passed")


if __name__ == '__main__':
    main()
