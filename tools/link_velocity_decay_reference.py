# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay Element-only Link velocity_decay metadata against the pinned SDK."""
import argparse
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import tempfile
import xml.etree.ElementTree as ET

from verify import isolated_git_context
from world_scene_reference import PIN, digest, linkage

ROOT = Path(__file__).resolve().parents[1]
PHYSICS_PIN = 'b44cd0d0ea6606b6c98320c0c87a5670d4fc6be6'
SIM_PIN = '71c60db31fddd5c98d2aa689e9a2cccb3548a0a3'


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--write-provenance', action='store_true')
    args = parser.parse_args()
    upstream = args.upstream.resolve()
    physics = upstream.parent / 'gz-physics'
    sim = upstream.parent / 'gz-sim'
    with isolated_git_context(ROOT):
        for repository, pin in [(upstream, PIN), (physics, PHYSICS_PIN), (sim, SIM_PIN)]:
            assert subprocess.check_output(['git', '-C', str(repository), 'rev-parse', 'HEAD'], text=True).strip() == pin
            subprocess.run(['git', '-C', str(repository), 'diff', '--quiet', pin, '--'], check=True)
    env = dict(os.environ)
    for name in ['LD_PRELOAD', 'LD_AUDIT', 'LD_LIBRARY_PATH']:
        env.pop(name, None)
    libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip())
    includedir = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    env['LD_LIBRARY_PATH'] = str(libdir)
    library = (libdir / 'libsdformat.so').resolve()
    assert digest(library) == digest(upstream / 'build/lib' / library.name)
    headers = [includedir / 'sdf' / (name + '.hh') for name in ['Root', 'Model', 'Link', 'Element']]
    for header in headers:
        assert digest(header) == digest(upstream / 'include/sdf' / header.name)
    assert 'velocity_decay' not in (upstream / 'src/Link.cc').read_text()
    assert 'VelocityDecay' not in (upstream / 'include/sdf/Link.hh').read_text()
    source_names = ['src/Link.cc', 'src/Param.cc', 'src/parser.cc', 'src/parser_urdf.cc',
                    'test/integration/urdf_gazebo_extensions.cc', 'test/integration/urdf_gazebo_extensions.urdf',
                    'test/sdf/bad_syntax_double.sdf', *[f'sdf/1.{v}/link.sdf' for v in range(4, 13)]]
    sources = [upstream / name for name in source_names]
    physics_sources = [physics / 'dartsim/src/SDFFeatures.cc', physics / 'mujoco/src/SDFFeatures.cc', sim / 'src/systems/physics/Physics.cc']
    for source in physics_sources:
        assert 'velocity_decay' not in source.read_text() and 'VelocityDecay' not in source.read_text()
    helper = ROOT / 'tests/link_velocity_decay_reference.cc'
    corpus = ROOT / 'tests/link_velocity_decay.json'
    fixture = ROOT / 'tests/fixtures/link_velocity_decay_native.sdf'
    bad_fixture = ROOT / 'tests/fixtures/bad_syntax_double.sdf'
    assert bad_fixture.read_bytes() == (upstream / 'test/sdf/bad_syntax_double.sdf').read_bytes()
    original = upstream / 'build/bin/INTEGRATION_urdf_gazebo_extensions'
    compiler = Path(shutil.which('c++')).resolve()
    protected = [Path(__file__), compiler, library, helper, corpus, fixture, bad_fixture, original, *headers, *sources, *physics_sources]
    before = {str(path): digest(path) for path in protected}
    data = json.loads(corpus.read_text())
    with tempfile.TemporaryDirectory(prefix='sdf-link-decay-native-') as temporary:
        binary = Path(temporary) / 'link-decay-reference'
        flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
        subprocess.run([str(compiler), '-std=c++17', '-Wall', '-Wextra', '-Werror', str(helper), '-o', str(binary), *flags, '-Wl,-rpath,' + str(libdir)], check=True, env=env)
        libraries = linkage(binary, env)
        def check(row, xml, mode=None):
            result = subprocess.run([str(binary)] + ([mode] if mode else []), input=xml, capture_output=True, text=True, env=env)
            assert (result.returncode, result.stdout, result.stderr) == (row['exit'], row['stdout'], row['stderr'])
        for row in data['cases']:
            check(row, row['xml'])
        urdf = (upstream / 'test/integration/urdf_gazebo_extensions.urdf').read_text()
        projected = subprocess.run([str(binary), '--project'], input=urdf, capture_output=True, text=True, env=env, check=True)
        assert projected.stdout == fixture.read_text() == data['original_projection']['xml']
        check(data['original_urdf'], urdf)
        check(data['original_projection'], fixture.read_text())
        assert data['original_urdf']['stdout'] == data['original_projection']['stdout']
        check(data['clone'], data['clone']['xml'], '--clone')
        check(data['constructor'], '', '--ctor')
        check(data['bad_syntax'], bad_fixture.read_text())
        output = ROOT / 'artifacts/link-velocity-decay-native'
        output.mkdir(parents=True, exist_ok=True)
        result = subprocess.run([str(original), '--gtest_filter=SDFParser.UrdfGazeboExtensionURDFTest', '--gtest_output=xml:' + str(output / 'original.xml')], capture_output=True, env=env)
        (output / 'original.stdout').write_bytes(result.stdout)
        (output / 'original.stderr').write_bytes(result.stderr)
        report = ET.parse(output / 'original.xml').getroot()
        assert result.returncode == 0 and report.attrib['tests'] == '1' and report.attrib['failures'] == '0'
        assert report.attrib.get('disabled', '0') == '0' and report.attrib.get('errors', '0') == '0'
        cases = list(report.iter('testcase'))
        assert len(cases) == 1 and cases[0].attrib['result'] == 'completed'
        assert not list(report.iter('skipped'))
        provenance = {
            'upstream_sha': PIN,
            'native_sources': {name: digest(upstream / name) for name in source_names},
            'installed_headers': {path.name: digest(path) for path in headers},
            'physics_sha': PHYSICS_PIN, 'sim_sha': SIM_PIN,
            'inspected_physics_sources': {str(path.relative_to(upstream.parent)): digest(path) for path in physics_sources},
            'compiler_sha256': digest(compiler), 'native_libraries': libraries,
            'helper_sha256': digest(helper), 'corpus_sha256': digest(corpus), 'fixture_sha256': digest(fixture),
            'bad_fixture_sha256': digest(bad_fixture), 'reference_script_sha256': digest(Path(__file__)),
            'original_sha256': digest(original), 'original_libraries': linkage(original, env),
        }
    assert before == {str(path): digest(path) for path in protected}, 'native inputs changed during replay'
    destination = ROOT / 'reference/link-velocity-decay.json'
    if args.write_provenance:
        destination.write_text(json.dumps(provenance, indent=2) + '\n')
    else:
        assert provenance == json.loads(destination.read_text()), 'native provenance changed'
    print(f"{len(data['cases'])} Element-only decay controls, exact native URDF/projection, clone and unchanged original passed")


if __name__ == '__main__':
    main()
