#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Capture bounded native origin-capsule cases, retaining direct and aggregate state."""
import argparse
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import xml.etree.ElementTree as ET

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
    headers = ['Root', 'Model', 'Link', 'Collision', 'Element', 'ParserConfig', 'Capsule']
    for name in headers:
        assert digest(includedir / 'sdf' / (name + '.hh')) == digest(upstream / 'include/sdf' / (name + '.hh'))
    math_include = pkg('includedir', 'gz-math') / 'gz/math'
    math_headers = ['Inertial', 'MassMatrix3', 'Matrix3', 'Vector3', 'Quaternion', 'Pose3', 'Capsule']
    for name in math_headers:
        assert digest(math_include / (name + '.hh')) == digest(math / 'include/gz/math' / (name + '.hh'))
    sources = [upstream / name for name in ['src/Collision.cc', 'src/Link.cc', 'src/Capsule.cc', 'src/Capsule_TEST.cc']]
    schema_names = ['root', 'model', 'link', 'inertial', 'collision', 'pose', 'capsule_shape']
    schemas = []
    for name in schema_names:
        selected = upstream / 'sdf/1.12' / (name + '.sdf')
        installed = pkg('prefix') / 'share/sdformat/1.12' / (name + '.sdf')
        assert digest(selected) == digest(installed)
        schemas.extend([selected, installed])
    helper = ROOT / 'tests/auto_capsule_inertial_reference.cc'
    cases_path = ROOT / 'tests/auto_capsule_inertial_cases.json'
    output_path = ROOT / 'tests/auto_capsule_inertial_native.json'
    cases = json.loads(cases_path.read_text())['cases']
    assert len(cases) == 24 and len({row['name'] for row in cases}) == 24
    for row in cases:
        pose = row['pose_text']
        if pose is None:
            assert '<pose' not in row['xml']
        else:
            assert ('<pose/>' if pose == '' else '<pose>' + pose + '</pose>') in row['xml']
    compiler = Path('/usr/bin/x86_64-linux-gnu-g++-11').resolve(strict=True)
    assert digest(compiler) == '02ba98cc5feefe173cfb8c28c98089817737800537dc7189138ed66b07cf56ec', 'selected established compiler identity changed'
    original_binary = upstream / 'build/bin/UNIT_Capsule_TEST'
    protected = [original_binary, Path(__file__), ROOT / 'tools/auto_translation_pose_reference.py',
                 ROOT / 'tools/auto_default_density_reference.py',
                 ROOT / 'tools/world_scene_reference.py', ROOT / 'tools/verify.py',
                 helper, cases_path, compiler, *libraries, *sources, *schemas,
                 *[includedir / 'sdf' / (name + '.hh') for name in headers],
                 *[math_include / (name + '.hh') for name in math_headers],
                 math / 'include/gz/math/detail/Capsule.hh',
                 ROOT / 'tests/auto_cylinder_inertial.rs']
    old_cases = ROOT / 'tests/auto_cylinder_inertial_cases.json'
    protected.append(old_cases)
    previous = next(row for row in json.loads(old_cases.read_text())['cases'] if row['name'] == 'retained_negative')
    retained = next(row for row in cases if row['name'] == 'retained_mixed_refusal')
    assert retained['xml'] == previous['xml'].replace('cylinder>', 'capsule>')
    assert retained['uri'] == 'other-geometry.sdf'
    assert '<capsule><radius>1</radius><length>2</length></capsule>' in (ROOT / 'tests/auto_cylinder_inertial.rs').read_text()
    if not args.capture:
        protected.append(output_path)
    before = {str(path): digest(path) for path in protected}
    out = ROOT / 'artifacts/auto-capsule-native'
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
                                   env=dict(env, GZ_CAPSULE_MAPS=str(maps)), timeout=10)
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
        observed.append(dict(name=row['name'], **observation(stdout), stdout=stdout, stderr=stderr))
    assert len(children) == len(cases) and all(row['exit_code'] == 0 and not row['timeout'] for row in children), 'native failures/timeouts retained; no qualification'
    assert len(observed) == len(cases)
    original_argv = [str(original_binary), '--gtest_filter=DOMCapsule.CalculateInertial',
                     '--gtest_output=xml:' + str(out / 'original.xml')]
    (out / 'original-command.json').write_text(json.dumps(original_argv, indent=2) + '\n')
    original_libraries = native_linkage(original_binary, env)
    for library in libraries:
        assert original_libraries.get(str(library)) == digest(library)
    (out / 'original-libraries.json').write_text(json.dumps(original_libraries, indent=2) + '\n')
    try:
        original = subprocess.run(original_argv, text=True, capture_output=True,
                                  env=dict(env, SDF_INSTALL_PREFIX=str(pkg('prefix'))), timeout=20)
        original_stdout, original_stderr, original_code = original.stdout, original.stderr, original.returncode
        original_timeout = False
    except subprocess.TimeoutExpired as error:
        original_stdout = (error.stdout or b'').decode()
        original_stderr = (error.stderr or b'').decode()
        original_code, original_timeout = None, True
    (out / 'original.stdout').write_text(original_stdout)
    (out / 'original.stderr').write_text(original_stderr)
    original_result = dict(exit_code=original_code, timeout=original_timeout)
    (out / 'original-result.json').write_text(json.dumps(original_result, indent=2) + '\n')
    assert original_code == 0 and not original_timeout, 'unchanged original failed; outputs retained'
    suites = ET.parse(out / 'original.xml').getroot()
    assert int(suites.attrib['tests']) == 1 and int(suites.attrib['failures']) == 0
    tests = suites.findall('.//testcase')
    assert len(tests) == 1 and tests[0].attrib['name'] == 'CalculateInertial'
    assert tests[0].attrib.get('status') == 'run' and tests[0].attrib.get('result') == 'completed'
    assert not suites.findall('.//failure') and not suites.findall('.//skipped')
    assert before == {str(path): digest(Path(path)) for path in before}, 'reference inputs changed'
    assert dependency_hashes == {path: digest(Path(path)) for path in dependency_hashes}, 'expanded compiler inputs changed'
    provenance = dict(upstream_sha=PIN, math_sha=MATH_PIN, command=command,
                      protected=before, binary_sha256=digest(binary),
                      dependencies=dependency_hashes,
                      native_libraries=loaded, children=children,
                      original=dict(argv=original_argv, **original_result, libraries=original_libraries))
    (out / 'result.json').write_text(json.dumps(provenance, indent=2) + '\n')
    value = dict(copyright='Copyright (C) 2026 Open Source Robotics Foundation',
                 license='Apache-2.0', scope='Actual native observations; no failed child or input omitted', observations=observed)
    if args.capture:
        output_path.write_text(json.dumps(value, indent=2) + '\n')
    else:
        assert value == json.loads(output_path.read_text())
    print(f"{len(observed)} native direct/aggregate capsule observations retained")


if __name__ == '__main__':
    main()
