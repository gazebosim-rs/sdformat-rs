#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Capture origin single-box/sphere automatic mass overrides, retaining direct and aggregate state."""
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
    for token, field in [('MASS_SPECIFIED', 'mass_specified'),
                         ('DIRECT_WARN_DEFAULT', 'direct_warn_default')]:
        flags = [line.split()[1:] for line in stdout.splitlines() if line.startswith(token + ' ')]
        assert len(flags) == 1 and len(flags[0]) == 1 and flags[0][0] in ('0', '1')
        value[field] = int(flags[0][0])
    return value



# One declared scalar-range refusal has no usable model/single-link DOM. Preserve
# its actual exit and diagnostics rather than inventing Root/Collision values.
LOAD_REFUSAL_XML = "<sdf version='1.12'><model name='m'><link name='l'><inertial auto='true'><mass>5e-324</mass></inertial><collision name='c'><density>1000</density><geometry><box><size>1 1 1</size></box></geometry></collision></link></model></sdf>"
LOAD_REFUSAL_STDOUT = 'LOAD 9 9 9 9 9 30\n'
LOAD_REFUSAL_STDERR = '\x1b[1;31mError [Param.cc:1288]\x1b[0m Error Code 34: Msg: Out of range. Unable to set value [5e-324 ] for key[mass].\n'


def retained_observation(row, child, out, loaded, binary):
    name = row['name']
    assert child['name'] == name
    stdout = (out / (name + '.stdout')).read_text()
    stderr = (out / (name + '.stderr')).read_text()
    maps = out / (name + '.maps')
    assert not child['timeout'], 'native timeout is never an expected observation'
    if child['exit_code'] != 0:
        assert name == 'minimum_subnormal_mass' and row['mass_text'] == '5e-324'
        assert row['xml'] == LOAD_REFUSAL_XML and row['uri'] == 'mass-override.sdf'
        assert row['rust_error'] == 'InvalidValue' and child['exit_code'] == 2
        assert stdout == LOAD_REFUSAL_STDOUT and stderr == LOAD_REFUSAL_STDERR
        assert not maps.exists() and not maps.is_symlink(), 'the early DOM refusal produced no maps'
        return dict(name=name, native_load_refusal=True, exit_code=2,
                    load_errors=[9, 9, 9, 9, 9, 30], maps_recorded=False,
                    stdout=stdout, stderr=stderr)
    assert maps.is_file()
    for line in maps.read_text().splitlines():
        fields = line.split(maxsplit=5)
        if len(fields) == 6 and fields[5].startswith('/'):
            path = Path(fields[5]).resolve()
            if path != binary:
                assert loaded.get(str(path)) == digest(path), ('unqualified mapping', path)
    if name in ('box_old_corpus', 'sphere_old_corpus'):
        source = row['source']
        prior = next(value for value in json.loads((ROOT / 'tests' / source['fixture']).read_text())['cases']
                     if value['name'] == source['case'])
        legacy = ''.join(line + '\n' for line in stdout.splitlines()
                         if line.startswith(('LOAD', 'VALUES', 'RESOLVE')))
        assert legacy == prior['stdout'] and stderr == prior['stderr'], 'exact retained Root input output changed'
    return dict(name=name, exit_code=0, maps_recorded=True,
                **observation(stdout), stdout=stdout, stderr=stderr)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--capture', action='store_true')
    parser.add_argument('--finish-retained', action='store_true')
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
    headers = ['Root', 'Model', 'Link', 'Collision', 'Element', 'ParserConfig', 'Param', 'Box', 'Sphere', 'Cylinder', 'Capsule']
    for name in headers:
        assert digest(includedir / 'sdf' / (name + '.hh')) == digest(upstream / 'include/sdf' / (name + '.hh'))
    math_include = pkg('includedir', 'gz-math') / 'gz/math'
    math_headers = ['Inertial', 'MassMatrix3', 'Matrix3', 'Vector3', 'Quaternion', 'Pose3', 'Box', 'Sphere', 'Cylinder', 'Capsule']
    for name in math_headers:
        assert digest(math_include / (name + '.hh')) == digest(math / 'include/gz/math' / (name + '.hh'))
    sources = [upstream / name for name in ['src/Collision.cc', 'src/Link.cc', 'src/Param.cc', 'src/Link_TEST.cc', 'src/Box.cc', 'src/Sphere.cc', 'src/Cylinder.cc', 'src/Capsule.cc']]
    schema_names = ['root', 'model', 'link', 'inertial', 'collision', 'pose', 'box_shape', 'sphere_shape', 'cylinder_shape', 'capsule_shape']
    schemas = []
    for name in schema_names:
        selected = upstream / 'sdf/1.12' / (name + '.sdf')
        installed = pkg('prefix') / 'share/sdformat/1.12' / (name + '.sdf')
        assert digest(selected) == digest(installed)
        schemas.extend([selected, installed])
    helper = ROOT / 'tests/auto_mass_override_reference.cc'
    cases_path = ROOT / 'tests/auto_mass_override_cases.json'
    output_path = ROOT / 'tests/auto_mass_override_native.json'
    cases = json.loads(cases_path.read_text())['cases']
    assert len(cases) == 24 and len({row['name'] for row in cases}) == 24
    inherited = []
    for row in cases:
        assert row['xml'] == row['base_xml'].replace(
            "<inertial auto='true'/>",
            "<inertial auto='true'><mass>" + row['mass_text'] + "</mass></inertial>")
        assert row['base_xml'].count("<inertial auto='true'/>") == 1
        if 'source' not in row:
            continue
        source = row['source']
        fixture = ROOT / 'tests' / source['fixture']
        prior = next(value for value in json.loads(fixture.read_text())['cases']
                     if value['name'] == source['case'])
        expected = prior['xml']
        if source.get('generated'):
            expected = expected.replace("<inertial auto='true'/>",
                                        "<inertial auto='true'><mass>1</mass></inertial>")
        assert row['xml'] == expected
        old = ROOT / 'tests' / source['test']
        assert row['uri'] in old.read_text()
        inherited.extend([fixture, old])
    inherited.extend([ROOT / 'tests/auto_box_inertial.json', ROOT / 'tests/auto_sphere_inertial.json'])
    compiler = Path('/usr/bin/x86_64-linux-gnu-g++-11').resolve(strict=True)
    assert digest(compiler) == '02ba98cc5feefe173cfb8c28c98089817737800537dc7189138ed66b07cf56ec', 'selected established compiler identity changed'
    original_binary = upstream / 'build/bin/UNIT_Link_TEST'
    protected = [original_binary, Path(__file__), ROOT / 'tools/auto_translation_pose_reference.py',
                 ROOT / 'tools/auto_default_density_reference.py',
                 ROOT / 'tools/world_scene_reference.py', ROOT / 'tools/verify.py',
                 helper, cases_path, compiler, *libraries, *sources, *schemas,
                 *[includedir / 'sdf' / (name + '.hh') for name in headers],
                 *[math_include / (name + '.hh') for name in math_headers],
                 *[math / ('include/gz/math/detail/' + name + '.hh') for name in ['Box', 'Sphere', 'Cylinder', 'Capsule']],
                 ROOT / 'tests/auto_cylinder_inertial.rs']
    protected.extend(inherited)
    if not args.capture:
        protected.append(output_path)
    out = ROOT / 'artifacts/auto-mass-override-native'
    finish = args.finish_retained
    if finish:
        assert args.capture and out.is_dir()
        assert not (out / 'result.json').exists()
        assert not (out / 'original-command.json').exists(), 'original must not run twice'
        assert not json.loads(output_path.read_text())['observations']
        before = json.loads((out / 'protected-inputs.json').read_text())
        assert set(before) == {str(path) for path in protected}
        copy = out / 'captured-driver.py'
        assert digest(copy) == before[str(Path(__file__))]
        for name, expected in before.items():
            assert digest(copy if name == str(Path(__file__)) else Path(name)) == expected
        command = json.loads((out / 'compile-command.json').read_text())
        loaded = json.loads((out / 'native-libraries.json').read_text())
        dependency_hashes = json.loads((out / 'dependencies.json').read_text())
        for name, expected in loaded.items():
            assert digest(Path(name)) == expected
        children = json.loads((out / 'children.json').read_text())
        assert [row['name'] for row in children] == [row['name'] for row in cases]
        capture_inputs = [path for path in out.iterdir() if path.is_file()]
        retained_hashes = {str(path): digest(path) for path in capture_inputs}
        finish_driver = digest(Path(__file__))
    else:
        before = {str(path): digest(path) for path in protected}
        out.mkdir(parents=True, exist_ok=False)
        children = []
    assert shutil.disk_usage(out).free > 1280 * 1024**2
    binary = out / 'reference'
    if not finish:
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
        for row in cases:
            assert re.fullmatch('[a-z0-9_]+', row['name'])
            maps = out / (row['name'] + '.maps')
            try:
                child = subprocess.run([str(binary)], input=row['xml'], text=True, capture_output=True,
                                       env=dict(env, GZ_MASS_OVERRIDE_MAPS=str(maps)), timeout=10)
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
    assert len(children) == len(cases)
    assert all(row['exit_code'] == 0 or
               (row['name'] == 'minimum_subnormal_mass' and row['exit_code'] == 2)
               for row in children)
    observed = [retained_observation(row, child, out, loaded, binary)
                for row, child in zip(cases, children)]
    assert len(observed) == len(cases)
    assert [row['name'] for row in observed if row.get('native_load_refusal')] == ['minimum_subnormal_mass']
    assert sum(row['maps_recorded'] for row in observed) == 23
    original_argv = [str(original_binary), '--gtest_filter=DOMLink.ResolveAutoInertialsWithMassAndDefaultDensity',
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
    assert len(tests) == 1 and tests[0].attrib['name'] == 'ResolveAutoInertialsWithMassAndDefaultDensity'
    assert tests[0].attrib.get('status') == 'run' and tests[0].attrib.get('result') == 'completed'
    assert not suites.findall('.//failure') and not suites.findall('.//skipped')
    if finish:
        assert retained_hashes == {name: digest(Path(name)) for name in retained_hashes}, 'retained capture changed'
        assert digest(Path(__file__)) == finish_driver, 'finish source changed'
        assert before == {name: digest(out / 'captured-driver.py' if name == str(Path(__file__)) else Path(name))
                          for name in before}, 'original capture inputs changed'
    else:
        assert before == {str(path): digest(Path(path)) for path in before}, 'reference inputs changed'
    assert dependency_hashes == {path: digest(Path(path)) for path in dependency_hashes}, 'expanded compiler inputs changed'
    provenance = dict(upstream_sha=PIN, math_sha=MATH_PIN, command=command,
                      protected=before, binary_sha256=digest(binary),
                      dependencies=dependency_hashes,
                      native_libraries=loaded, children=children,
                      original=dict(argv=original_argv, **original_result, libraries=original_libraries),
                      mode='finish_retained_capture_original_once' if finish else 'complete_capture',
                      actual_observer_maps=23,
                      controlled_load_refusals=1,
                      capture_driver_copy=str(out / 'captured-driver.py') if finish else None,
                      finish_driver_sha256=finish_driver if finish else None)
    if finish:
        provenance['retained_capture_sha256'] = retained_hashes
    (out / 'result.json').write_text(json.dumps(provenance, indent=2) + '\n')
    value = dict(copyright='Copyright (C) 2026 Open Source Robotics Foundation',
                 license='Apache-2.0', scope='Actual native observations; no failed child or input omitted', observations=observed)
    if args.capture:
        output_path.write_text(json.dumps(value, indent=2) + '\n')
    else:
        assert value == json.loads(output_path.read_text())
    print(f"{len(observed)} native direct/aggregate mass-override observations retained")


if __name__ == '__main__':
    main()
