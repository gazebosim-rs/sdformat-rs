#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Capture bounded native combined-pose cases, retaining direct and aggregate state."""
import argparse
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess

from auto_default_density_reference import MATH_PIN, native_linkage
from verify import isolated_git_context
from world_scene_reference import PIN, digest

ROOT = Path(__file__).resolve().parents[1]


def observation(stdout):
    from auto_translation_pose_reference import observation as base_observation
    value = base_observation(stdout)
    rows = [line.split()[1:] for line in stdout.splitlines()
            if line.startswith('COLLISION_MOI ')]
    assert len(rows) == 1 and len(rows[0]) == 9
    matrix = list(map(int, rows[0]))
    assert all(0 <= bit < 2**64 for bit in matrix)
    value['collision_moi'] = matrix
    return value


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
    for key in ['LD_PRELOAD', 'LD_AUDIT', 'LD_LIBRARY_PATH', 'SDF_PATH', 'CPATH',
                'CPLUS_INCLUDE_PATH', 'LIBRARY_PATH', 'CXXFLAGS', 'CPPFLAGS', 'LDFLAGS',
                'PKG_CONFIG_SYSROOT_DIR']:
        env.pop(key, None)
    env['LC_ALL'] = 'C'
    def pkg(variable, package='sdformat'):
        return Path(subprocess.check_output(['pkg-config', '--variable=' + variable, package], env=env, text=True).strip()).resolve()
    libdir, includedir = pkg('libdir'), pkg('includedir')
    env['LD_LIBRARY_PATH'] = str(libdir)
    libraries = [(libdir / 'libsdformat.so').resolve(), (libdir / 'libgz-math.so').resolve()]
    for library, repository in zip(libraries, [upstream, math]):
        assert digest(library) == digest(repository / 'build/lib' / library.name)
    headers = ['Root', 'Model', 'Link', 'Collision', 'Element', 'ParserConfig']
    for name in headers:
        assert digest(includedir / 'sdf' / (name + '.hh')) == digest(upstream / 'include/sdf' / (name + '.hh'))
    math_include = pkg('includedir', 'gz-math') / 'gz/math'
    math_headers = ['Inertial', 'MassMatrix3', 'Matrix3', 'Vector3', 'Quaternion', 'Pose3']
    for name in math_headers:
        assert digest(math_include / (name + '.hh')) == digest(math / 'include/gz/math' / (name + '.hh'))
    sources = [upstream / name for name in ['src/Collision.cc', 'src/Link.cc', 'src/Box.cc', 'src/Sphere.cc']]
    schema_names = ['root', 'model', 'link', 'inertial', 'collision', 'pose', 'box_shape', 'sphere_shape']
    schemas = []
    for name in schema_names:
        selected = upstream / 'sdf/1.12' / (name + '.sdf')
        installed = pkg('prefix') / 'share/sdformat/1.12' / (name + '.sdf')
        assert digest(selected) == digest(installed)
        schemas.extend([selected, installed])
    helper = ROOT / 'tests/auto_rigid_pose_reference.cc'
    cases_path = ROOT / 'tests/auto_rigid_pose_cases.json'
    output_path = ROOT / 'tests/auto_rigid_pose_native.json'
    cases = json.loads(cases_path.read_text())['cases']
    assert len(cases) == 24 and len({row['name'] for row in cases}) == 24
    for row in cases:
        assert ('<pose>' + row['pose_text'] + '</pose>') in row['xml']
    compiler = Path('/usr/bin/x86_64-linux-gnu-g++-11').resolve(strict=True)
    assert digest(compiler) == '02ba98cc5feefe173cfb8c28c98089817737800537dc7189138ed66b07cf56ec', 'selected established compiler identity changed'
    protected = [Path(__file__), ROOT / 'tools/auto_translation_pose_reference.py',
                 ROOT / 'tools/auto_default_density_reference.py',
                 ROOT / 'tools/world_scene_reference.py', ROOT / 'tools/verify.py',
                 helper, cases_path, compiler, *libraries, *sources, *schemas,
                 *[includedir / 'sdf' / (name + '.hh') for name in headers],
                 *[math_include / (name + '.hh') for name in math_headers]]
    for row in cases:
        if 'inherited_numeric_case' in row:
            filename, name = row['inherited_numeric_case'].rsplit(':', 1)
            protected.append(ROOT / filename)
            previous = next(item for item in json.loads((ROOT / filename).read_text())['cases'] if item['name'] == name)
            assert row['xml'] == previous['xml'].replace('<pose>0 0 0 0.1 0.2 0.3</pose>', '<pose>1 0 0 0.1 0.2 0.3</pose>')
            prior_native_path = ROOT / 'tests/auto_rotation_pose_native.json'
            protected.append(prior_native_path)
            previous_native = next(item for item in json.loads(prior_native_path.read_text())['observations'] if item['name'] == name)
            assert row['rust_error'] == 'Unsupported'
            assert previous.get('rust_error') == 'Unsupported' or not all(previous_native['root_status'][1])
        if 'source_corpus' in row:
            protected.append(ROOT / row['source_corpus'])
            protected.append(ROOT / 'tests/auto_rotation_pose.rs')
            prior = next(item for item in json.loads((ROOT / row['source_corpus']).read_text())['cases']
                         if item['name'] == row['source_case'])
            assert row['xml'] == prior['xml'].replace('<pose>0 0 0 0 0 0</pose>', row['held_pose'])
            assert row['held_pose'] in (ROOT / 'tests/auto_rotation_pose.rs').read_text()
        if 'old_corpus' in row:
            protected.append(ROOT / row['old_corpus'])
    if not args.capture:
        protected.append(output_path)
    before = {str(path): digest(path) for path in protected}
    out = ROOT / 'artifacts/auto-rigid-native'
    out.mkdir(parents=True, exist_ok=False)
    assert shutil.disk_usage(out).free > 1280 * 1024**2
    binary = out / 'reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], env=env, text=True))
    command = [str(compiler), '-std=c++17', '-Wall', '-Wextra', '-Werror', str(helper),
               '-MD', '-MF', str(out / 'reference.d'), '-o', str(binary), *flags, '-Wl,-rpath,' + str(libdir)]
    (out / 'protected-inputs.json').write_text(json.dumps(before, indent=2) + '\n')
    (out / 'compile-command.json').write_text(json.dumps(command, indent=2) + '\n')
    with (out / 'compile.stdout').open('wb') as stdout, (out / 'compile.stderr').open('wb') as stderr:
        subprocess.run(command, check=True, env=env, stdout=stdout, stderr=stderr, timeout=60)
    loaded = native_linkage(binary, env)
    (out / 'native-libraries.json').write_text(json.dumps(loaded, indent=2) + '\n')
    dependencies = shlex.split((out / 'reference.d').read_text().replace('\\\n', ' ').split(':', 1)[1])
    dependency_hashes = {str(Path(path).resolve()): digest(Path(path)) for path in dependencies}
    (out / 'dependencies.json').write_text(json.dumps(dependency_hashes, indent=2) + '\n')
    children, observed = [], []
    for row in cases:
        assert re.fullmatch('[a-z0-9_]+', row['name'])
        maps = out / (row['name'] + '.maps')
        try:
            child = subprocess.run([str(binary)], input=row['xml'], text=True, capture_output=True,
                                   env=dict(env, GZ_RIGID_MAPS=str(maps)), timeout=10)
            stdout, stderr, code = child.stdout, child.stderr, child.returncode
            timeout = False
        except subprocess.TimeoutExpired as error:
            stdout = (error.stdout or b'').decode()
            stderr = (error.stderr or b'').decode()
            code, timeout = None, True
        (out / (row['name'] + '.stdout')).write_text(stdout)
        (out / (row['name'] + '.stderr')).write_text(stderr)
        children.append(dict(name=row['name'], exit_code=code, timeout=timeout))
        (out / 'children.json').write_text(json.dumps(children, indent=2) + '\n')
        if code != 0 or timeout:
            continue
        for line in maps.read_text().splitlines():
            fields = line.split(maxsplit=5)
            if len(fields) == 6 and fields[5].startswith('/'):
                path = Path(fields[5]).resolve()
                if path != binary:
                    assert loaded.get(str(path)) == digest(path), ('unqualified mapping', path)
        if 'old_corpus' in row:
            prior = next(item for item in json.loads((ROOT / row['old_corpus']).read_text())['cases'] if item['name'] == row['old_name'])
            assert row['xml'] == prior['xml']
            projection = ''.join(line + '\n' for line in stdout.splitlines()
                                 if line == 'LOAD' or line.startswith(('LOAD ', 'VALUES ', 'RESOLVE')))
            assert projection == prior['stdout'] and stderr == prior['stderr']
        observed.append(dict(name=row['name'], **observation(stdout), stdout=stdout, stderr=stderr))
    assert len(children) == len(cases) and all(row['exit_code'] == 0 and not row['timeout'] for row in children), 'native failures/timeouts retained; no qualification'
    assert len(observed) == len(cases)
    assert before == {str(path): digest(Path(path)) for path in before}, 'reference inputs changed'
    assert dependency_hashes == {path: digest(Path(path)) for path in dependency_hashes}, 'expanded compiler inputs changed'
    provenance = dict(upstream_sha=PIN, math_sha=MATH_PIN, command=command,
                      protected=before, binary_sha256=digest(binary),
                      dependencies=dependency_hashes,
                      native_libraries=loaded, children=children)
    (out / 'result.json').write_text(json.dumps(provenance, indent=2) + '\n')
    value = dict(copyright='Copyright (C) 2026 Open Source Robotics Foundation',
                 license='Apache-2.0', scope='Actual native observations; no failed child or input omitted', observations=observed)
    if args.capture:
        output_path.write_text(json.dumps(value, indent=2) + '\n')
    else:
        assert value == json.loads(output_path.read_text())
    print(f"{len(observed)} native direct/aggregate rigid-pose observations retained")


if __name__ == '__main__':
    main()
