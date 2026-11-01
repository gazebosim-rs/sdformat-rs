#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay the pinned legacy IMU typed-load projection and unchanged converter test."""
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
    def pkg(variable, package='sdformat'):
        return Path(subprocess.check_output(['pkg-config', '--variable=' + variable, package], env=env, text=True).strip()).resolve()
    libdir, includedir = pkg('libdir'), pkg('includedir')
    env['LD_LIBRARY_PATH'] = str(libdir)
    library = (libdir / 'libsdformat.so').resolve()
    assert digest(library) == digest(upstream / 'build/lib' / library.name)
    math_library = (libdir / 'libgz-math.so').resolve()
    assert digest(math_library) == digest(math / 'build/lib' / math_library.name)
    headers = ['Root', 'World', 'Model', 'Link', 'Joint', 'Sensor', 'Imu', 'Noise', 'ParserConfig', 'Element']
    for name in headers:
        assert digest(includedir / 'sdf' / (name + '.hh')) == digest(upstream / 'include/sdf' / (name + '.hh'))
    math_header = pkg('includedir', 'gz-math') / 'gz/math/Vector3.hh'
    assert digest(math_header) == digest(math / 'include/gz/math/Vector3.hh')
    sources = ['src/Converter.cc', 'src/Converter_TEST.cc', 'src/Imu.cc', 'src/Sensor.cc', 'src/Noise.cc',
               'src/Link.cc', 'src/parser.cc', 'src/Param.cc', 'sdf/1.4/imu.sdf', 'sdf/1.5/imu.sdf',
               'sdf/1.6/1_5.convert', 'sdf/1.12/imu.sdf', 'sdf/1.12/noise.sdf']
    data_path = ROOT / 'tests/legacy_imu_noise.json'
    data = json.loads(data_path.read_text())
    original_source = (upstream / 'src/Converter_TEST.cc').read_text().split('TEST(Converter, IMU_15_to_16)', 1)[1]
    original_xml = original_source.split('R"(', 1)[1].split(')";', 1)[0]
    fixture = ROOT / data['original_fixture']
    assert fixture.read_bytes() == original_xml.encode()
    compiler = Path(shutil.which('c++')).resolve()
    helper = ROOT / 'tests/legacy_imu_noise_reference.cc'
    original = upstream / 'build/bin/UNIT_Converter_TEST'
    protected = [Path(__file__), helper, fixture, compiler, library, math_library, math_header, original,
                 ROOT / 'tests/fixtures/legacy_imu_resource.sdf', *[upstream / name for name in sources],
                 *[includedir / 'sdf' / (name + '.hh') for name in headers]]
    before = {str(path): digest(path) for path in protected}
    output = ROOT / 'artifacts/legacy-imu-noise-native'
    output.mkdir(parents=True, exist_ok=True)
    assert shutil.disk_usage(output).free > 1024 ** 3, 'less than 1 GiB free'
    binary = output / 'reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], env=env, text=True))
    command = [str(compiler), '-std=c++17', '-Wall', '-Wextra', '-Werror', str(helper), '-o', str(binary),
               *flags, '-Wl,-rpath,' + str(libdir)]
    subprocess.run(command, check=True, env=env)
    libraries = linkage(binary, env)
    def query(xml, resource=None):
        arguments = [str(binary)] + ([str(ROOT / resource)] if resource else [])
        value = subprocess.run(arguments, input=xml, text=True, capture_output=True, env=env, timeout=10, check=True)
        return {'stdout': value.stdout, 'stderr': value.stderr}
    for row in data['cases']:
        actual = query(row['xml'], row.get('resource'))
        if args.capture:
            row['native'] = actual
        else:
            assert actual == row['native'], row['name']
    observations = {'original': query(original_xml)}
    old = json.loads((ROOT / 'tests/version15_unsupported.json').read_text())[1]
    assert old['id'] == 'legacy_imu_noise'
    observations['retained_unsupported_row'] = query(old['xml'])
    if args.capture:
        data.update(observations)
        data_path.write_text(json.dumps(data, indent=2) + '\n')
    else:
        assert all(data[name] == value for name, value in observations.items())
    report = output / 'original.xml'
    value = subprocess.run([str(original), '--gtest_filter=Converter.IMU_15_to_16', '--gtest_output=xml:' + str(report)],
                           capture_output=True, env=env, check=True, timeout=30)
    (output / 'original.stdout').write_bytes(value.stdout)
    (output / 'original.stderr').write_bytes(value.stderr)
    xml = ET.parse(report).getroot()
    assert xml.attrib['tests'] == '1' and xml.attrib['failures'] == '0'
    assert xml.attrib.get('disabled', '0') == xml.attrib.get('errors', '0') == '0'
    assert not list(xml.iter('skipped'))
    assert {row.attrib['classname'] + '.' + row.attrib['name'] for row in xml.iter('testcase')} == {'Converter.IMU_15_to_16'}
    provenance = {'upstream_sha': PIN, 'math_sha': MATH_PIN,
                  'native_sources': {name: digest(upstream / name) for name in sources},
                  'installed_headers': {name: digest(includedir / 'sdf' / (name + '.hh')) for name in headers},
                  'math_header_sha256': digest(math_header), 'compiler_sha256': digest(compiler),
                  'compiler_version': subprocess.check_output([str(compiler), '--version'], text=True, env=env),
                  'helper_sha256': digest(helper), 'reference_script_sha256': digest(Path(__file__)),
                  'native_libraries': libraries, 'original_libraries': linkage(original, env),
                  'original_binary_sha256': digest(original), 'fixture_sha256': digest(fixture),
                  'resource_sha256': digest(ROOT / 'tests/fixtures/legacy_imu_resource.sdf'),
                  'corpus_sha256': digest(data_path)}
    assert before == {str(path): digest(path) for path in protected}, 'native input changed during replay'
    reference = ROOT / 'reference/legacy-imu-noise.json'
    if args.capture:
        reference.write_text(json.dumps(provenance, indent=2) + '\n')
    else:
        assert provenance == json.loads(reference.read_text())
    (output / 'result.json').write_text(json.dumps({'cases': len(data['cases']), 'original_projection': observations['original'],
        'retained_row': observations['retained_unsupported_row'], 'unchanged_originals': 1,
        'command': command, 'provenance': provenance}, indent=2) + '\n')
    print(f"{len(data['cases'])} native legacy IMU controls, exact original/retained-row projections and unchanged converter original passed")


if __name__ == '__main__':
    main()
