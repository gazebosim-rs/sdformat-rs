#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay bounded standalone Lidar aliases against the pinned native DOM."""
import argparse
import json
import os
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
    def pkg(variable):
        return Path(subprocess.check_output(['pkg-config', '--variable=' + variable, 'sdformat'], env=env, text=True).strip()).resolve()
    libdir, includedir = pkg('libdir'), pkg('includedir')
    schema = pkg('prefix') / 'share/sdformat/1.12/lidar.sdf'
    assert digest(schema) == digest(upstream / 'sdf/1.12/lidar.sdf')
    env['LD_LIBRARY_PATH'] = str(libdir)
    library = (libdir / 'libsdformat.so').resolve()
    assert digest(library) == digest(upstream / 'build/lib' / library.name)
    math_library = (libdir / 'libgz-math.so').resolve()
    assert digest(math_library) == digest(math / 'build/lib' / math_library.name)
    math_include = Path(subprocess.check_output(
        ['pkg-config', '--variable=includedir', 'gz-math'], env=env, text=True).strip())
    math_headers = ['Angle', 'Helpers', 'Vector3']
    for name in math_headers:
        assert digest(math_include / 'gz/math' / (name + '.hh')) == digest(math / 'include/gz/math' / (name + '.hh'))
    headers = ['Lidar', 'Element', 'Noise', 'parser']
    for name in headers:
        assert digest(includedir / 'sdf' / (name + '.hh')) == digest(upstream / 'include/sdf' / (name + '.hh'))
    compiler = Path(shutil.which('c++')).resolve()
    helper = ROOT / 'tests/lidar_aliases_reference.cc'
    corpus = ROOT / 'tests/lidar_aliases.json'
    original = upstream / 'build/bin/UNIT_Lidar_TEST'
    sources = ['src/Lidar.cc', 'src/Lidar_TEST.cc', 'src/Noise.cc', 'src/Sensor.cc', 'src/parser.cc',
               'src/Param.cc', 'sdf/1.12/lidar.sdf', 'sdf/1.12/ray.sdf', 'sdf/1.12/sensor.sdf']
    protected = [Path(__file__), helper, compiler, library, math_library, schema, original,
                 *[upstream / name for name in sources],
                 *[math_include / 'gz/math' / (name + '.hh') for name in math_headers],
                 *[includedir / 'sdf' / (name + '.hh') for name in headers]]
    before = {str(path): digest(path) for path in protected}
    output = ROOT / 'artifacts/lidar-aliases-native'
    output.mkdir(parents=True, exist_ok=True)
    assert shutil.disk_usage(output).free > 1024 ** 3, 'less than 1 GiB free'
    binary = output / 'reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], env=env, text=True))
    command = [str(compiler), '-std=c++17', '-Wall', '-Wextra', '-Werror', str(helper), '-o', str(binary),
               *flags, '-Wl,-rpath,' + str(libdir)]
    subprocess.run(command, check=True, env=env)
    libraries = linkage(binary, env)
    data = json.loads(corpus.read_text())
    def query(arguments, xml=None):
        result = subprocess.run([str(binary), *arguments], input=xml, text=True,
                                capture_output=True, check=True, env=env, timeout=10)
        return {'stdout': result.stdout, 'stderr': result.stderr}
    for row in data['cases']:
        actual = query([row['tag'], row['version'], str(schema)], row['xml'])
        if args.capture:
            row['native'] = actual
        else:
            assert row['native'] == actual, row['name']
    originals = {name: query(['--' + name]) for name in ['null', 'bad']}
    if args.capture:
        data['original_controls'] = originals
        corpus.write_text(json.dumps(data, indent=2) + '\n')
    else:
        assert data['original_controls'] == originals
    report = output / 'originals.xml'
    result = subprocess.run([str(original), '--gtest_filter=DOMLidar.*', '--gtest_output=xml:' + str(report)],
                            capture_output=True, check=True, env=env, timeout=30)
    (output / 'originals.stdout').write_bytes(result.stdout)
    (output / 'originals.stderr').write_bytes(result.stderr)
    xml = ET.parse(report).getroot()
    assert xml.attrib['tests'] == '4' and xml.attrib['failures'] == '0'
    assert xml.attrib.get('disabled', '0') == xml.attrib.get('errors', '0') == '0'
    assert not list(xml.iter('skipped'))
    assert {case.attrib['classname'] + '.' + case.attrib['name'] for case in xml.iter('testcase')} == {
        'DOMLidar.Construction', 'DOMLidar.Set', 'DOMLidar.Load', 'DOMLidar.ToElement'}
    provenance = {'upstream_sha': PIN, 'math_sha': MATH_PIN,
                  'compiler_sha256': digest(compiler),
                  'compiler_version': subprocess.check_output([str(compiler), '--version'], env=env, text=True),
                  'native_sources': {name: digest(upstream / name) for name in sources},
                  'math_headers': {name: digest(math_include / 'gz/math' / (name + '.hh')) for name in math_headers},
                  'installed_headers': {name: digest(includedir / 'sdf' / (name + '.hh')) for name in headers},
                  'reference_script_sha256': digest(Path(__file__)), 'helper_sha256': digest(helper),
                  'native_libraries': libraries, 'original_libraries': linkage(original, env),
                  'original_binary_sha256': digest(original), 'schema_sha256': digest(schema),
                  'corpus_sha256': digest(corpus)}
    assert before == {str(path): digest(path) for path in protected}, 'native input changed during replay'
    reference = ROOT / 'reference/lidar-aliases.json'
    if args.capture:
        reference.write_text(json.dumps(provenance, indent=2) + '\n')
    else:
        assert provenance == json.loads(reference.read_text()), 'native provenance changed'
    (output / 'result.json').write_text(json.dumps({'cases': len(data['cases']), 'original_controls': len(originals),
        'unchanged_originals': 4, 'command': command, 'provenance': provenance}, indent=2) + '\n')
    print(f"{len(data['cases'])} native alias cases, two original Load controls and four unchanged Lidar tests passed")


if __name__ == '__main__':
    main()
