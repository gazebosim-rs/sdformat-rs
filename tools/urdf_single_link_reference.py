# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay the bounded URDF import controls and both unchanged native originals."""
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
MATH_PIN = '1316ed334f4d5bda7be59aae07cd6e8ff31cfe13'


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--write-provenance', action='store_true')
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
    libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True, env=env).strip())
    includedir = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True, env=env).strip())
    env['LD_LIBRARY_PATH'] = str(libdir)
    library = (libdir / 'libsdformat.so').resolve()
    assert digest(library) == digest(upstream / 'build/lib' / library.name)
    names = ['Root', 'Model', 'Link', 'Collision', 'Visual', 'Material', 'Geometry', 'Element', 'Surface', 'SemanticPose', 'Box', 'Sphere', 'Cylinder']
    headers = [includedir / 'sdf' / (name + '.hh') for name in names]
    for header in headers:
        assert digest(header) == digest(upstream / 'include/sdf' / header.name)
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], text=True, env=env).strip())
    math_library = (libdir / 'libgz-math.so').resolve()
    assert digest(math_library) == digest(math / 'build/lib' / math_library.name)
    math_headers = [math_include / 'gz/math' / (name + '.hh') for name in ['Pose3', 'Quaternion', 'Vector3', 'Inertial', 'MassMatrix3']]
    for header in math_headers:
        assert digest(header) == digest(math / 'include/gz/math' / header.name)
    urdf_headers = [Path('/usr/include/urdf_model') / (name + '.h') for name in ['pose', 'utils', 'link']]
    source_names = ['src/parser_urdf.cc', 'src/parser_urdf.hh', 'src/parser_urdf_TEST.cc', 'src/parser.cc',
                    'src/Root.cc', 'src/Model.cc', 'src/Link.cc', 'src/Material.cc', 'src/Surface.cc',
                    'src/Visual.cc', 'src/Collision.cc', 'src/Sphere.cc', 'src/Box.cc', 'src/Cylinder.cc',
                    'src/FrameSemantics.cc', 'src/SemanticPose.cc', 'test/integration/urdf_to_sdf.cc']
    fixtures = ['tests/fixtures/urdf_single_link_original.urdf', 'tests/fixtures/urdf_basic_original.urdf']
    for fixture, file, case in [(fixtures[0], 'src/parser_urdf_TEST.cc', 'TEST(URDFParser, ParseWhitespace)'),
                                (fixtures[1], 'test/integration/urdf_to_sdf.cc', 'TEST(URDF2SDF, ValidBasicURDF)')]:
        raw = (upstream / file).read_text().split(case, 1)[1].split(')";', 1)[0].split('R"(', 1)[1]
        assert (ROOT / fixture).read_bytes() == raw.encode()
    helper = ROOT / 'tests/urdf_single_link_reference.cc'
    corpus = ROOT / 'tests/urdf_single_link.json'
    originals = [('UNIT_parser_urdf_TEST', 'URDFParser.ParseWhitespace'), ('INTEGRATION_urdf_to_sdf', 'URDF2SDF.ValidBasicURDF')]
    compiler = Path(shutil.which('c++')).resolve()
    metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--offline', '--locked', '--format-version=1'], cwd=ROOT, text=True, env=env))
    package, = [p for p in metadata['packages'] if p['name'] == 'urdf-rs']
    assert package['version'] == '0.10.0' and package['license'] == 'Apache-2.0'
    crate = Path(package['manifest_path']).parent
    crate_sources = [crate / name for name in ['Cargo.toml.orig', 'LICENSE', 'src/funcs.rs', 'src/deserialize.rs']]
    protected = [Path(__file__), compiler, library, math_library, helper, corpus, ROOT / 'Cargo.toml', ROOT / 'Cargo.lock',
                 *headers, *math_headers, *urdf_headers, *crate_sources, *[ROOT / name for name in fixtures],
                 *[upstream / name for name in source_names], *[upstream / 'build/bin' / name for name, _ in originals]]
    before = {str(path): digest(path) for path in protected}
    data = json.loads(corpus.read_text())
    output = ROOT / 'artifacts/urdf-single-link-native'
    output.mkdir(parents=True, exist_ok=True)
    original_evidence = {}
    with tempfile.TemporaryDirectory(prefix='sdf-urdf-native-') as temporary:
        binary = Path(temporary) / 'urdf-reference'
        flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True, env=env))
        subprocess.run([str(compiler), '-std=c++17', '-Wall', '-Wextra', '-Werror', str(helper), '-o', str(binary), *flags, '-Wl,-rpath,' + str(libdir)], check=True, env=env)
        libraries = linkage(binary, env)
        for row in data['cases']:
            result = subprocess.run([str(binary)], input=row['xml'], capture_output=True, text=True, env=env)
            assert (result.returncode, result.stdout, result.stderr) == (row['exit'], row['stdout'], row['stderr']), row['name']
        for name, case in originals:
            original = upstream / 'build/bin' / name
            report_path = output / (name + '.xml')
            result = subprocess.run([str(original), '--gtest_filter=' + case, '--gtest_output=xml:' + str(report_path)], capture_output=True, env=env)
            (output / (name + '.stdout')).write_bytes(result.stdout)
            (output / (name + '.stderr')).write_bytes(result.stderr)
            report = ET.parse(report_path).getroot()
            assert result.returncode == 0 and report.attrib['tests'] == '1' and report.attrib['failures'] == '0'
            assert report.attrib.get('disabled', '0') == '0' and report.attrib.get('errors', '0') == '0'
            cases = list(report.iter('testcase'))
            assert len(cases) == 1 and cases[0].attrib['result'] == 'completed' and not list(report.iter('skipped'))
            original_evidence[name] = {'sha256': digest(original), 'native_libraries': linkage(original, env)}
        provenance = {
            'upstream_sha': PIN, 'math_sha': MATH_PIN,
            'native_sources': {name: digest(upstream / name) for name in source_names},
            'installed_headers': {path.name: digest(path) for path in headers},
            'math_headers': {path.name: digest(path) for path in math_headers},
            'native_urdfdom_headers': {path.name: digest(path) for path in urdf_headers},
            'compiler_sha256': digest(compiler), 'native_libraries': libraries,
            'helper_sha256': digest(helper), 'corpus_sha256': digest(corpus),
            'fixtures': {name: digest(ROOT / name) for name in fixtures},
            'reference_script_sha256': digest(Path(__file__)), 'originals': original_evidence,
            'rust_urdf_crate': {'version': package['version'], 'license': package['license'],
                               'sources': {str(path.relative_to(crate)): digest(path) for path in crate_sources},
                               'cargo_lock_sha256': digest(ROOT / 'Cargo.lock')},
        }
    assert before == {str(path): digest(path) for path in protected}, 'reference inputs changed during replay'
    destination = ROOT / 'reference/urdf-single-link.json'
    if args.write_provenance:
        destination.write_text(json.dumps(provenance, indent=2) + '\n')
    else:
        assert provenance == json.loads(destination.read_text()), 'reference provenance changed'
    print(f"{len(data['cases'])} pinned URDF controls, exact original fixtures and both unchanged originals passed")


if __name__ == '__main__':
    main()
