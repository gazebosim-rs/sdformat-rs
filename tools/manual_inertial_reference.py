#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay explicit manual inertial loading against the pinned native DOM."""
import argparse
import json
import os
import re
from pathlib import Path
import shlex
import shutil
import subprocess
import xml.etree.ElementTree as ET

from verify import isolated_git_context
from world_scene_reference import PIN, digest, linkage

ROOT = Path(__file__).resolve().parents[1]
MATH_PIN = '1316ed334f4d5bda7be59aae07cd6e8ff31cfe13'


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--capture', action='store_true')
    args = parser.parse_args()
    upstream = args.upstream.resolve()
    math = upstream.parent / 'gz-math'
    with isolated_git_context(ROOT):
        for repository, revision in [(upstream, PIN), (math, MATH_PIN)]:
            assert subprocess.check_output(['git', '-C', str(repository), 'rev-parse', 'HEAD'], text=True).strip() == revision
            subprocess.run(['git', '-C', str(repository), 'diff', '--quiet', revision, '--'], check=True)
    env = dict(os.environ)
    for name in ['LD_PRELOAD', 'LD_AUDIT', 'LD_LIBRARY_PATH', 'SDF_PATH']:
        env.pop(name, None)
    def pkg(variable, package='sdformat'):
        return Path(subprocess.check_output(['pkg-config', '--variable=' + variable, package], env=env, text=True).strip()).resolve()
    libdir, includedir = pkg('libdir'), pkg('includedir')
    env['LD_LIBRARY_PATH'] = str(libdir)
    library = (libdir / 'libsdformat.so').resolve()
    math_library = (libdir / 'libgz-math.so').resolve()
    assert digest(library) == digest(upstream / 'build/lib' / library.name)
    assert digest(math_library) == digest(math / 'build/lib' / math_library.name)
    headers = ['Root', 'World', 'Model', 'Link', 'ParserConfig']
    for name in headers:
        assert digest(includedir / 'sdf' / (name + '.hh')) == digest(upstream / 'include/sdf' / (name + '.hh'))
    math_include = pkg('includedir', 'gz-math') / 'gz/math'
    math_headers = ['Inertial', 'MassMatrix3', 'Vector3', 'Quaternion', 'Pose3']
    for name in math_headers:
        assert digest(math_include / (name + '.hh')) == digest(math / 'include/gz/math' / (name + '.hh'))
    sources = ['src/Link.cc', 'src/Link_TEST.cc', 'src/Param.cc',
               *[f'sdf/1.{version}/inertial.sdf' for version in range(4, 13)],
               *[f'sdf/1.{version + 1}/1_{version}.convert' for version in range(4, 12)],
               'sdf/1.12/root.sdf', 'sdf/1.12/model.sdf', 'sdf/1.12/link.sdf']
    schema_dir = pkg('prefix') / 'share/sdformat'
    schemas = [name.removeprefix('sdf/') for name in sources if name.startswith('sdf/')]
    for name in schemas:
        assert digest(schema_dir / name) == digest(upstream / 'sdf' / name)
    original_source = (upstream / 'src/Link_TEST.cc').read_text().split('TEST(DOMLink, ResolveAutoInertialsCalledWithAutoFalse)', 1)[1]
    literals = original_source.split('std::string sdf = ', 1)[1].split(';', 1)[0]
    original_xml = ''.join(json.loads(value) for value in re.findall(r'"(?:[^"\\]|\\.)*"', literals))
    fixture = ROOT / 'tests/fixtures/manual_inertial_original.sdf'
    assert fixture.read_bytes() == original_xml.encode()
    compiler = Path(shutil.which('c++')).resolve()
    helper = ROOT / 'tests/manual_inertial_reference.cc'
    original = upstream / 'build/bin/UNIT_Link_TEST'
    data_path = ROOT / 'tests/manual_inertial.json'
    data = json.loads(data_path.read_text())
    protected = [Path(__file__), helper, fixture, data_path, compiler, library, math_library, original, *[math_include / (name + ".hh") for name in math_headers],
                 *[upstream / name for name in sources], *[schema_dir / name for name in schemas], *[includedir / 'sdf' / (name + '.hh') for name in headers]]
    before = {str(path): digest(path) for path in protected}
    output = ROOT / 'artifacts/manual-inertial-native'
    output.mkdir(parents=True, exist_ok=True)
    assert shutil.disk_usage(output).free > 1024 ** 3, 'less than 1 GiB free'
    binary = output / 'reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], env=env, text=True))
    command = [str(compiler), '-std=c++17', '-Wall', '-Wextra', '-Werror', str(helper), '-o', str(binary),
               *flags, '-Wl,-rpath,' + str(libdir)]
    subprocess.run(command, check=True, env=env)
    for row in data['cases']:
        value = subprocess.run([str(binary)], input=row['xml'], text=True, capture_output=True,
                               env=env, timeout=10, check=True)
        assert {'stdout': value.stdout, 'stderr': value.stderr} == {key: row[key] for key in ('stdout', 'stderr')}, row['name']
    report = output / 'original.xml'
    value = subprocess.run([str(original), '--gtest_filter=DOMLink.ResolveAutoInertialsCalledWithAutoFalse',
                            '--gtest_output=xml:' + str(report)], capture_output=True, env=env, check=True, timeout=30)
    (output / 'original.stdout').write_bytes(value.stdout)
    (output / 'original.stderr').write_bytes(value.stderr)
    xml = ET.parse(report).getroot()
    assert xml.attrib['tests'] == '1' and xml.attrib['failures'] == '0'
    assert xml.attrib.get('disabled', '0') == xml.attrib.get('errors', '0') == '0'
    assert not list(xml.iter('skipped'))
    assert {row.attrib['classname'] + '.' + row.attrib['name'] for row in xml.iter('testcase')} == {'DOMLink.ResolveAutoInertialsCalledWithAutoFalse'}
    provenance = {'upstream_sha': PIN, 'math_sha': MATH_PIN,
                  'native_sources': {name: digest(upstream / name) for name in sources},
                  'installed_headers': {name: digest(includedir / 'sdf' / (name + '.hh')) for name in headers},
                  'installed_schemas': {name: digest(schema_dir / name) for name in schemas},
                  'math_headers': {name: digest(math_include / (name + '.hh')) for name in math_headers},
                  'compiler_sha256': digest(compiler),
                  'compiler_version': subprocess.check_output([str(compiler), '--version'], text=True, env=env),
                  'helper_sha256': digest(helper), 'reference_script_sha256': digest(Path(__file__)),
                  'native_libraries': linkage(binary, env), 'original_libraries': linkage(original, env),
                  'original_binary_sha256': digest(original), 'fixture_sha256': digest(fixture),
                  'corpus_sha256': digest(data_path)}
    assert before == {str(path): digest(path) for path in protected}, 'native input changed during replay'
    reference = ROOT / 'reference/manual-inertial.json'
    if args.capture:
        reference.write_text(json.dumps(provenance, indent=2) + '\n')
    else:
        assert provenance == json.loads(reference.read_text())
    (output / 'result.json').write_text(json.dumps({'cases': len(data['cases']), 'unchanged_originals': 1,
                                                 'command': command, 'provenance': provenance}, indent=2) + '\n')
    print(f"{len(data['cases'])} native manual inertial controls and unchanged original passed")


if __name__ == '__main__':
    main()
