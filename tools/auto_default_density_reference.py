#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay native default-density automatic inertia without changing old goldens."""
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


def native_linkage(binary, env):
    text = subprocess.check_output(['ldd', str(binary)], env=env, text=True)
    assert 'not found' not in text
    paths = {Path(word).resolve() for line in text.splitlines()
             for word in line.split() if word.startswith('/')}
    assert any(path.name.startswith('libsdformat.so') for path in paths)
    return {str(path): digest(path) for path in sorted(paths)}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--capture', action='store_true')
    args = parser.parse_args()
    upstream = args.upstream.resolve()
    math = upstream.parent / 'gz-math'
    with isolated_git_context(ROOT):
        for repository, revision in [(upstream, PIN), (math, MATH_PIN)]:
            assert subprocess.check_output(
                ['git', '-C', str(repository), 'rev-parse', 'HEAD'], text=True).strip() == revision
            subprocess.run(['git', '-C', str(repository), 'diff', '--quiet', revision, '--'], check=True)
    env = dict(os.environ)
    for name in ['LD_PRELOAD', 'LD_AUDIT', 'LD_LIBRARY_PATH', 'SDF_PATH',
                 'CPATH', 'CPLUS_INCLUDE_PATH', 'LIBRARY_PATH', 'CXXFLAGS',
                 'CPPFLAGS', 'LDFLAGS', 'PKG_CONFIG_SYSROOT_DIR']:
        env.pop(name, None)
    env['LC_ALL'] = 'C'

    def pkg(variable, package='sdformat'):
        return Path(subprocess.check_output(
            ['pkg-config', '--variable=' + variable, package], env=env, text=True).strip()).resolve()

    libdir, includedir = pkg('libdir'), pkg('includedir')
    env['LD_LIBRARY_PATH'] = str(libdir)
    library = (libdir / 'libsdformat.so').resolve()
    math_library = (libdir / 'libgz-math.so').resolve()
    assert digest(library) == digest(upstream / 'build/lib' / library.name)
    assert digest(math_library) == digest(math / 'build/lib' / math_library.name)
    headers = ['Root', 'World', 'Model', 'Link', 'Collision', 'Element', 'ParserConfig']
    for name in headers:
        assert digest(includedir / 'sdf' / (name + '.hh')) == digest(upstream / 'include/sdf' / (name + '.hh'))
    sources = ['src/Link.cc', 'src/Link_TEST.cc', 'src/Collision.cc', 'src/Collision_TEST.cc',
               'src/Box.cc', 'src/Sphere.cc', 'src/Param.cc',
               *[f'sdf/1.{version}/inertial.sdf' for version in range(4, 13)],
               *[f'sdf/1.{version + 1}/1_{version}.convert' for version in range(4, 12)],
               'sdf/1.12/root.sdf', 'sdf/1.12/model.sdf', 'sdf/1.12/link.sdf',
               'sdf/1.12/collision.sdf', 'sdf/1.12/box_shape.sdf', 'sdf/1.12/sphere_shape.sdf']
    schema_dir = pkg('prefix') / 'share/sdformat'
    schemas = [name.removeprefix('sdf/') for name in sources if name.startswith('sdf/')]
    for name in schemas:
        assert digest(schema_dir / name) == digest(upstream / 'sdf' / name)
    math_include = pkg('includedir', 'gz-math') / 'gz/math'
    math_headers = ['Inertial', 'MassMatrix3', 'Vector3', 'Quaternion', 'Pose3',
                    'Box', 'detail/Box', 'Sphere', 'detail/Sphere', 'Material']
    for name in math_headers:
        assert digest(math_include / (name + '.hh')) == digest(math / 'include/gz/math' / (name + '.hh'))
    compiler = Path(shutil.which('c++')).resolve()
    helper = ROOT / 'tests/auto_default_density_reference.cc'
    data_path = ROOT / 'tests/auto_default_density.json'
    old_data_path = ROOT / 'tests/auto_box_inertial.json'
    data = json.loads(data_path.read_text())
    old = {row['name']: row for row in json.loads(old_data_path.read_text())['cases']}
    original = upstream / 'build/bin/UNIT_Link_TEST'
    protected = [Path(__file__), helper, old_data_path, compiler, library, math_library, original,
                 *[upstream / name for name in sources], *[schema_dir / name for name in schemas],
                 *[includedir / 'sdf' / (name + '.hh') for name in headers],
                 *[math_include / (name + '.hh') for name in math_headers]]
    if not args.capture:
        protected.append(data_path)
    before = {str(path): digest(path) for path in protected}
    output = ROOT / 'artifacts/auto-default-density-native'
    output.mkdir(parents=True, exist_ok=True)
    assert shutil.disk_usage(output).free > 1280 * 1024**2, 'less than 1.25 GiB free'
    binary = output / 'reference'
    flags = shlex.split(subprocess.check_output(
        ['pkg-config', '--cflags', '--libs', 'sdformat'], env=env, text=True))
    command = [str(compiler), '-std=c++17', '-Wall', '-Wextra', '-Werror', str(helper),
               '-MD', '-MF', str(output / 'reference.d'), '-o', str(binary),
               *flags, '-Wl,-rpath,' + str(libdir)]
    with (output / 'compile.stdout').open('wb') as stdout, (output / 'compile.stderr').open('wb') as stderr:
        subprocess.run(command, check=True, env=env, stdout=stdout, stderr=stderr, timeout=60)
    loaded = native_linkage(binary, env)
    outcomes = []
    for row in data['cases']:
        argv = [str(binary)] + (['strict'] if row['strict_warning'] else [])
        maps = output / (row['name'] + '.maps')
        value = subprocess.run(argv, input=row['xml'], text=True, capture_output=True,
                               env=dict(env, GZ_DENSITY_MAPS=str(maps)), timeout=10)
        (output / (row['name'] + '.stdout')).write_text(value.stdout)
        (output / (row['name'] + '.stderr')).write_text(value.stderr)
        outcomes.append({'name': row['name'], 'argv': argv, 'exit_code': value.returncode})
        (output / 'children.json').write_text(json.dumps(outcomes, indent=2) + '\n')
        assert value.returncode == 0, row['name']
        for line in maps.read_text().splitlines():
            fields = line.split(maxsplit=5)
            if len(fields) == 6 and fields[5].startswith('/'):
                path = Path(fields[5]).resolve()
                if path != binary:
                    assert loaded.get(str(path)) == digest(path), ('unqualified mapping', path)
        if row.get('old_name'):
            prior = old[row['old_name']]
            assert row['xml'] == prior['xml']
            projection = ''.join(line + '\n' for line in value.stdout.splitlines()
                                 if not line.startswith('DENSITY '))
            assert projection == prior['stdout'] and value.stderr == prior['stderr']
        observed = {'stdout': value.stdout, 'stderr': value.stderr}
        if args.capture:
            row.update(observed)
        else:
            assert observed == {key: row[key] for key in observed}, row['name']
    report = output / 'original.xml'
    with (output / 'original.stdout').open('wb') as stdout, (output / 'original.stderr').open('wb') as stderr:
        subprocess.run([str(original), '--gtest_filter=DOMLink.ResolveAutoInertialsWithDifferentDensity',
                        '--gtest_output=xml:' + str(report)], env=env, check=True,
                       stdout=stdout, stderr=stderr, timeout=30)
    xml = ET.parse(report).getroot()
    assert xml.attrib['tests'] == '1' and xml.attrib['failures'] == '0'
    assert xml.attrib.get('disabled', '0') == xml.attrib.get('errors', '0') == '0'
    assert not list(xml.iter('skipped'))
    assert {row.attrib['classname'] + '.' + row.attrib['name'] for row in xml.iter('testcase')} == {
        'DOMLink.ResolveAutoInertialsWithDifferentDensity'}
    dependencies = shlex.split((output / 'reference.d').read_text().replace('\\\n', ' ').split(':', 1)[1])
    if args.capture:
        data_path.write_text(json.dumps(data, indent=2) + '\n')
    provenance = {'upstream_sha': PIN, 'math_sha': MATH_PIN,
                  'native_sources': {name: digest(upstream / name) for name in sources},
                  'installed_headers': {name: digest(includedir / 'sdf' / (name + '.hh')) for name in headers},
                  'installed_schemas': {name: digest(schema_dir / name) for name in schemas},
                  'math_headers': {name: digest(math_include / (name + '.hh')) for name in math_headers},
                  'compiler_sha256': digest(compiler), 'helper_sha256': digest(helper),
                  'compiler_inputs': {
                      ('tests/auto_default_density_reference.cc' if Path(name).resolve() == helper
                       else str(Path(name).resolve())): digest(Path(name)) for name in dependencies},
                  'native_libraries': loaded, 'original_libraries': linkage(original, env),
                  'original_binary_sha256': digest(original), 'old_corpus_sha256': digest(old_data_path),
                  'corpus_sha256': digest(data_path),
                  'reference_script_sha256': digest(Path(__file__))}
    assert before == {path: digest(Path(path)) for path in before}, 'native input changed'
    assert loaded == native_linkage(binary, env), 'native library closure changed'
    reference = ROOT / 'reference/auto-default-density.json'
    if args.capture:
        reference.write_text(json.dumps(provenance, indent=2) + '\n')
    else:
        assert provenance == json.loads(reference.read_text())
    (output / 'result.json').write_text(json.dumps({
        'cases': len(outcomes), 'unchanged_native_originals': 1,
        'scope': 'Direct native default-density characterization; no Rust original or warning-policy parity',
        'command': command, 'elf_sha256': digest(binary), 'provenance': provenance}, indent=2) + '\n')
    print(f"{len(outcomes)} native default-density controls and unchanged original passed")


if __name__ == '__main__':
    main()
