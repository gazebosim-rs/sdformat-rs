#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay world Light includes; retain the native empty-name abort as a failure."""
import argparse
import json
import os
from pathlib import Path
import resource
import shlex
import shutil
import subprocess
import xml.etree.ElementTree as ET

from verify import isolated_git_context
from world_scene_reference import PIN, digest, linkage

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--capture', action='store_true')
    parser.add_argument('--urdf-libdir', type=Path, default=Path('/opt/ros/jazzy/lib/x86_64-linux-gnu'))
    args = parser.parse_args()
    upstream = args.upstream.resolve()
    light_proof = json.loads((ROOT / 'reference/document-lights.json').read_text())
    math = upstream.parent / 'gz-math'
    with isolated_git_context(ROOT):
        assert subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip() == PIN
        subprocess.run(['git', '-C', str(upstream), 'diff', '--quiet', PIN, '--'], check=True)
        assert subprocess.check_output(['git', '-C', str(math), 'rev-parse', 'HEAD'], text=True).strip() == light_proof['math_upstream_sha']
    env = dict(os.environ)
    for name in ['LD_PRELOAD', 'LD_AUDIT', 'LD_LIBRARY_PATH', 'SDF_PATH']:
        env.pop(name, None)
    def pkg(variable):
        return Path(subprocess.check_output(['pkg-config', '--variable=' + variable, 'sdformat'], env=env, text=True).strip()).resolve()
    libdir, includedir = pkg('libdir'), pkg('includedir')
    urdf_libdir = args.urdf_libdir.resolve()
    assert digest(urdf_libdir / 'liburdfdom_model.so.4.0') == light_proof['libraries']['liburdfdom_model.so.4.0']
    env['LD_LIBRARY_PATH'] = str(libdir) + ':' + str(urdf_libdir)
    library = (libdir / 'libsdformat.so').resolve()
    assert digest(library) == digest(upstream / 'build/lib' / library.name)
    headers = ['Light', 'World', 'Root', 'Element', 'ParserConfig', 'SemanticPose']
    for name in headers:
        assert digest(includedir / 'sdf' / (name + '.hh')) == digest(upstream / 'include/sdf' / (name + '.hh'))
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], env=env, text=True).strip()).resolve()
    for name, expected in light_proof['math_sources'].items():
        assert digest(math / name) == expected
        assert digest(math_include / name.removeprefix('include/')) == expected
    schema_dir = pkg('prefix') / 'share/sdformat'
    schemas = [f'1.{version}/light.sdf' for version in range(4,13)] + [f'1.{version+1}/1_{version}.convert' for version in range(4,12)] + ['1.12/root.sdf','1.12/world.sdf','1.12/pose.sdf']
    for name in schemas:
        assert digest(schema_dir / name) == digest(upstream / 'sdf' / name)
    sources = ['src/parser.cc', 'src/Light.cc', 'src/World.cc', 'src/Root.cc',
               'src/Element.cc', 'src/Param.cc', 'src/SemanticPose.cc', 'src/FrameSemantics.cc',
               'test/integration/includes.cc', 'test/sdf/includes.sdf',
               'test/integration/model/test_light/model.sdf']
    sources += ['sdf/' + name for name in schemas]
    fixtures = ['tests/fixtures/included_light_original.sdf', 'tests/fixtures/include_lights_full_original.sdf',
                'tests/fixtures/include_lights_original.sdf']
    assert (ROOT / fixtures[0]).read_bytes() == (upstream / 'test/integration/model/test_light/model.sdf').read_bytes()
    assert (ROOT / fixtures[1]).read_bytes() == (upstream / 'test/sdf/includes.sdf').read_bytes()
    full = (ROOT / fixtures[1]).read_text()
    start = full.index('    <include>\n      <uri>test_light</uri>')
    end = full.index('    <include>\n      <uri>test_actor</uri>')
    projection = full[:full.index('    <include>')] + full[start:end] + '  </world>\n</sdf>\n'
    assert (ROOT / fixtures[2]).read_text() == projection
    data_path = ROOT / 'tests/include_lights.json'
    rows = json.loads(data_path.read_text())
    assert len(rows) == 23
    output = ROOT / 'artifacts/include-light-native'
    output.mkdir(parents=True, exist_ok=True)
    assert shutil.disk_usage(output).free > 1024 ** 3
    compiler = Path(shutil.which('c++')).resolve()
    helper = ROOT / 'tests/include_light_reference.cc'
    original = upstream / 'build/bin/INTEGRATION_includes'
    protected = [Path(__file__), helper, data_path, compiler, library, original,
                 *[upstream / name for name in sources], *[ROOT / name for name in fixtures],
                 *[includedir / 'sdf' / (name + '.hh') for name in headers],
                 *[math / name for name in light_proof['math_sources']],
                 *[math_include / name.removeprefix('include/') for name in light_proof['math_sources']],
                 *[schema_dir / name for name in schemas]]
    before = {str(p): digest(p) for p in protected}
    binary = output / 'reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], env=env, text=True))
    command = [str(compiler), '-std=c++17', '-Wall', '-Wextra', '-Werror', str(helper),
               *flags, '-Wl,--disable-new-dtags', '-Wl,-rpath,' + str(libdir), '-o', str(binary)]
    subprocess.run(command, check=True, env=env)
    assert linkage(binary, env) == light_proof['libraries']
    for executable in [binary, original]:
        for line in subprocess.check_output(['ldd', str(executable)], env=env, text=True).splitlines():
            words = line.split()
            if '=>' in words and len(words) > 2 and words[2].startswith('/'):
                path = Path(words[2]).resolve()
                before[str(path)] = digest(path)
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    observations = []
    for row in rows:
        source = output / row['resource']
        source.write_text(row['resource_xml'])
        run = subprocess.run([str(binary), str(source)], input=row['xml'], text=True,
                             capture_output=True, env=env, timeout=10)
        measured = {'exit_code': run.returncode,
                    'stdout': run.stdout.replace(str(source).encode().hex(), '@RESOURCE@'),
                    'stderr': run.stderr}
        assert measured == row['native'], (row['id'], measured, row['native'])
        observations.append({'id': row['id'], **measured})
    assert [row['id'] for row in observations if row['exit_code']] == ['emptyname']
    report = output / 'original.xml'
    run = subprocess.run([str(original), '--gtest_filter=IncludesTest.Includes',
                          '--gtest_output=xml:' + str(report)], env=env, capture_output=True, timeout=30)
    (output / 'original.stdout').write_bytes(run.stdout)
    (output / 'original.stderr').write_bytes(run.stderr)
    assert run.returncode == 0
    xml = ET.parse(report).getroot()
    assert xml.attrib['tests'] == '1' and xml.attrib['failures'] == '0'
    assert xml.attrib.get('disabled', '0') == xml.attrib.get('errors', '0') == '0'
    assert not list(xml.iter('skipped'))
    assert {r.attrib['classname'] + '.' + r.attrib['name'] for r in xml.iter('testcase')} == {'IncludesTest.Includes'}
    provenance = {'upstream_sha': PIN, 'native_sources': {name: digest(upstream / name) for name in sources},
                  'installed_headers': {name: digest(includedir / 'sdf' / (name + '.hh')) for name in headers},
                  'math_upstream_sha': light_proof['math_upstream_sha'], 'math_headers': light_proof['math_sources'],
                  'installed_schemas': {name: digest(schema_dir / name) for name in schemas},
                  'compiler_sha256': digest(compiler), 'compiler_version': subprocess.check_output([str(compiler), '--version'], env=env, text=True),
                  'helper_sha256': digest(helper), 'script_sha256': digest(Path(__file__)),
                  'helper_libraries': linkage(binary, env), 'original_libraries': linkage(original, env),
                  'original_binary_sha256': digest(original), 'fixtures': {name: digest(ROOT / name) for name in fixtures},
                  'corpus_sha256': digest(data_path)}
    assert before == {name: digest(Path(name)) for name in before}
    reference = ROOT / 'reference/include-lights.json'
    if args.capture:
        reference.write_text(json.dumps(provenance, indent=2) + '\n')
    else:
        assert provenance == json.loads(reference.read_text())
    (output / 'result.json').write_text(json.dumps({'command': command, 'observations': observations,
        'native_normal_exits': 22, 'known_native_aborts': 1, 'unchanged_originals': 1,
        'provenance': provenance}, indent=2) + '\n')
    print('23 native observations replayed: 22 normal exits and one preserved empty-name abort; unchanged Includes original passed')


if __name__ == '__main__':
    main()
