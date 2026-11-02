#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Capture bounded native translation cases, retaining direct and aggregate state."""
import argparse
import json
import math
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import struct

from auto_default_density_reference import MATH_PIN, native_linkage
from verify import isolated_git_context
from world_scene_reference import PIN, digest

ROOT = Path(__file__).resolve().parents[1]


def observation(stdout):
    lines = stdout.splitlines()
    def entries(prefix):
        return [list(map(int, line.removeprefix(prefix).split()))
                for line in lines if line.startswith(prefix)]
    values = entries('VALUES ')
    status = entries('ROOT_STATUS ')
    direct = entries('COLLISION_VALUES ')
    direct_status = entries('COLLISION_STATUS ')
    assert len(values) == len(status) == 2 and all(len(row) == 18 for row in values)
    assert len(direct) == len(direct_status) == 1 and len(direct[0]) == 14
    assert all(len(row) == 2 and all(v in (0, 1) for v in row) for row in status + direct_status)
    assert all(0 <= bit < 2**64 for row in values + direct for bit in row)
    assert all(all(flag in (0, 1) for flag in row[:2] + row[-2:]) for row in values)
    for row, state in zip([value[2:16] for value in values] + direct, status + direct_status):
        finite = all(math.isfinite(struct.unpack('<d', struct.pack('<Q', bit))[0]) for bit in row)
        assert finite == bool(state[0]), 'finite status differs from recorded float bits'
    def errors(label):
        found = [line for line in lines if line == label or line.startswith(label + ' ')]
        assert len(found) == 1
        codes = list(map(int, found[0].split()[1:]))
        assert all(code >= 0 for code in codes)
        return codes
    return dict(root_values=values, root_status=status, collision_values=direct[0],
                collision_status=direct_status[0], load_errors=errors('LOAD'),
                resolve_errors=errors('RESOLVE'), collision_errors=errors('COLLISION_ERRORS'))


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
    math_headers = ['Inertial', 'MassMatrix3', 'Vector3', 'Quaternion', 'Pose3']
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
    helper = ROOT / 'tests/auto_translation_pose_reference.cc'
    cases_path = ROOT / 'tests/auto_translation_pose_cases.json'
    output_path = ROOT / 'tests/auto_translation_pose_native.json'
    cases = json.loads(cases_path.read_text())['cases']
    compiler = Path(shutil.which('c++')).resolve()
    protected = [Path(__file__), ROOT / 'tools/auto_default_density_reference.py',
                 ROOT / 'tools/world_scene_reference.py', ROOT / 'tools/verify.py',
                 helper, cases_path, compiler, *libraries, *sources, *schemas,
                 *[includedir / 'sdf' / (name + '.hh') for name in headers],
                 *[math_include / (name + '.hh') for name in math_headers]]
    for row in cases:
        if 'old_corpus' in row:
            protected.append(ROOT / row['old_corpus'])
    if not args.capture:
        protected.append(output_path)
    before = {str(path): digest(path) for path in protected}
    out = ROOT / 'artifacts/auto-translation-native'
    out.mkdir(parents=True, exist_ok=True)
    assert shutil.disk_usage(out).free > 1280 * 1024**2
    binary = out / 'reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], env=env, text=True))
    command = [str(compiler), '-std=c++17', '-Wall', '-Wextra', '-Werror', str(helper),
               '-MD', '-MF', str(out / 'reference.d'), '-o', str(binary), *flags, '-Wl,-rpath,' + str(libdir)]
    with (out / 'compile.stdout').open('wb') as stdout, (out / 'compile.stderr').open('wb') as stderr:
        subprocess.run(command, check=True, env=env, stdout=stdout, stderr=stderr, timeout=60)
    loaded = native_linkage(binary, env)
    children, observed = [], []
    for row in cases:
        assert re.fullmatch('[a-z0-9_]+', row['name'])
        maps = out / (row['name'] + '.maps')
        try:
            child = subprocess.run([str(binary)], input=row['xml'], text=True, capture_output=True,
                                   env=dict(env, GZ_TRANSLATION_MAPS=str(maps)), timeout=10)
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
    dependencies = shlex.split((out / 'reference.d').read_text().replace('\\\n', ' ').split(':', 1)[1])
    provenance = dict(upstream_sha=PIN, math_sha=MATH_PIN, command=command,
                      protected=before, binary_sha256=digest(binary),
                      dependencies={str(Path(path).resolve()): digest(Path(path)) for path in dependencies},
                      native_libraries=loaded, children=children)
    (out / 'result.json').write_text(json.dumps(provenance, indent=2) + '\n')
    value = dict(copyright='Copyright (C) 2026 Open Source Robotics Foundation',
                 license='Apache-2.0', scope='Actual native observations; no failed child or input omitted', observations=observed)
    if args.capture:
        output_path.write_text(json.dumps(value, indent=2) + '\n')
    else:
        assert value == json.loads(output_path.read_text())
    print(f"{len(observed)} native direct/aggregate translation observations retained")


if __name__ == '__main__':
    main()
