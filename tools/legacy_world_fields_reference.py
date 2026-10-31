# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay pinned legacy world-vector conversion and both unchanged originals."""
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
        for repository, pin in [(upstream, PIN), (math, MATH_PIN)]:
            assert subprocess.check_output(['git', '-C', str(repository), 'rev-parse', 'HEAD'], text=True).strip() == pin
            subprocess.run(['git', '-C', str(repository), 'diff', '--quiet', pin, '--'], check=True)
    env = dict(os.environ)
    for name in ['LD_PRELOAD', 'LD_AUDIT', 'LD_LIBRARY_PATH']:
        env.pop(name, None)
    libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], env=env, text=True).strip())
    includedir = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], env=env, text=True).strip())
    env['LD_LIBRARY_PATH'] = str(libdir)
    library = (libdir / 'libsdformat.so').resolve()
    assert digest(library) == digest(upstream / 'build/lib' / library.name)
    headers = ['Root', 'World', 'Physics', 'Element', 'SDFImpl', 'parser']
    for header in headers:
        assert digest(includedir / 'sdf' / (header + '.hh')) == digest(upstream / 'include/sdf' / (header + '.hh'))
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], env=env, text=True).strip())
    math_header = math_include / 'gz/math/Vector3.hh'
    assert digest(math_header) == digest(math / 'include/gz/math/Vector3.hh')
    math_library = (libdir / 'libgz-math.so').resolve()
    assert digest(math_library) == digest(math / 'build/lib' / math_library.name)
    sources = ['src/Converter.cc', 'src/parser.cc', 'src/Param.cc', 'src/Root.cc', 'src/World.cc', 'src/Physics.cc',
               'sdf/1.4/physics.sdf', 'sdf/1.5/physics.sdf', 'sdf/1.6/1_5.convert', 'sdf/1.12/world.sdf',
               'test/integration/converter.cc']
    corpus = ROOT / 'tests/legacy_world_fields.json'
    data = json.loads(corpus.read_text())
    original_text = (upstream / 'test/integration/converter.cc').read_text()
    inline = original_text.split('std::string xmlString = R"(', 1)[1].split(')";', 1)[0]
    inline = inline.replace(')" + _version + R"(', '{version}')
    for version, name in data['original_fixtures'].items():
        assert (ROOT / name).read_bytes() == inline.replace('{version}', version).encode()
    helper = ROOT / 'tests/legacy_world_fields_reference.cc'
    compiler = Path(shutil.which('c++')).resolve()
    original = upstream / 'build/bin/INTEGRATION_converter'
    protected = [Path(__file__), compiler, library, math_library, math_header, helper, original, *[upstream / name for name in sources],
                 *[includedir / 'sdf' / (name + '.hh') for name in headers],
                 *[ROOT / name for name in data['original_fixtures'].values()]]
    before = {str(path): digest(path) for path in protected}
    output = ROOT / 'artifacts/legacy-world-fields-native'
    output.mkdir(parents=True, exist_ok=True)
    binary = output / 'reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], env=env, text=True))
    command = [str(compiler), '-std=c++17', '-Wall', '-Wextra', '-Werror', str(helper), '-o', str(binary), *flags, '-Wl,-rpath,' + str(libdir)]
    subprocess.run(command, check=True, env=env)
    libraries = linkage(binary, env)
    def query(xml, mode=()):
        value = subprocess.run([str(binary), *mode], input=xml, text=True, capture_output=True, env=env, timeout=10)
        assert value.returncode == 0, value.stderr
        return {'exit': value.returncode, 'stdout': value.stdout, 'stderr': value.stderr}
    for row in data['cases']:
        actual = query(row['xml'])
        if args.capture:
            row['native'] = actual
        else:
            assert actual == row['native'], row['name']
    originals = {}
    for version, path in data['original_fixtures'].items():
        actual = {mode: query((ROOT / path).read_text(), ['--convert'] if mode == 'convert' else []) for mode in ['root', 'convert']}
        if args.capture:
            originals[version] = actual
        else:
            assert actual == data['originals'][version]
    if args.capture:
        data['originals'] = originals
        corpus.write_text(json.dumps(data, indent=2) + '\n')
    report_path = output / 'originals.xml'
    cases = ['ConverterIntegration.ParserStringConverter', 'ConverterIntegration.ParserStringConverterFrom14']
    value = subprocess.run([str(original), '--gtest_filter=' + ':'.join(cases), '--gtest_output=xml:' + str(report_path)], capture_output=True, env=env, timeout=30)
    (output / 'originals.stdout').write_bytes(value.stdout)
    (output / 'originals.stderr').write_bytes(value.stderr)
    report = ET.parse(report_path).getroot()
    assert value.returncode == 0 and report.attrib['tests'] == '2' and report.attrib['failures'] == '0'
    assert report.attrib.get('disabled', '0') == report.attrib.get('errors', '0') == '0'
    assert not list(report.iter('skipped'))
    assert {row.attrib['classname'] + '.' + row.attrib['name'] for row in report.iter('testcase')} == set(cases)
    assert not value.stderr
    provenance = {'upstream_sha': PIN, 'math_sha': MATH_PIN, 'math_header_sha256': digest(math_header),
                  'native_sources': {name: digest(upstream / name) for name in sources},
                  'installed_headers': {name: digest(includedir / 'sdf' / (name + '.hh')) for name in headers},
                  'compiler_sha256': digest(compiler), 'native_libraries': libraries,
                  'helper_sha256': digest(helper), 'corpus_sha256': digest(corpus),
                  'fixtures': {name: digest(ROOT / name) for name in data['original_fixtures'].values()},
                  'reference_script_sha256': digest(Path(__file__)),
                  'original': {'binary_sha256': digest(original), 'libraries': linkage(original, env)}}
    assert before == {str(path): digest(path) for path in protected}, 'native inputs changed during replay'
    destination = ROOT / 'reference/legacy-world-fields.json'
    if args.capture:
        destination.write_text(json.dumps(provenance, indent=2) + '\n')
    else:
        assert provenance == json.loads(destination.read_text())
    (output / 'result.json').write_text(json.dumps({'native_cases': len(data['cases']), 'unchanged_originals': 2,
                                                  'command': command, 'provenance': provenance}, indent=2) + '\n')
    print(f"{len(data['cases'])} pinned native controls and two unchanged originals passed")


if __name__ == '__main__':
    main()
