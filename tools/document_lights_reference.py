#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay native document Light schema, pose and fixture-projection observations."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--output', type=Path, default=Path('artifacts/document-lights-reference/result.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = (root / args.output).resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    provenance = json.loads((root / 'reference/document-lights.json').read_text())
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == provenance['upstream_sha']
    includedir = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    assert sha(includedir / 'sdf/Light.hh') == sha(upstream / 'include/sdf/Light.hh')
    for path, expected in provenance['native_sources'].items():
        assert sha(upstream / path) == expected, path
    math = upstream.parent / 'gz-math'
    assert subprocess.check_output(['git', '-C', str(math), 'rev-parse', 'HEAD'], text=True).strip() == provenance['math_upstream_sha']
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], text=True).strip())
    for path, expected in provenance['math_sources'].items():
        assert sha(math / path) == expected, path
        assert sha(math_include / path.removeprefix('include/')) == expected, path
    libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()).resolve()
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    binaries = []
    for source in provenance['generators']:
        binary = output.parent / Path(source).stem
        subprocess.run(['c++', '-std=c++17', str(root / source), *flags,
                        '-Wl,--disable-new-dtags', f'-Wl,-rpath,{libdir}', '-o', str(binary)], check=True)
        libraries = {}
        for line in subprocess.check_output(['ldd', str(binary)], text=True).splitlines():
            words = line.split()
            if '=>' in words and len(words) > 2 and Path(words[2]).is_file():
                path = Path(words[2]).resolve()
                libraries[path.name] = sha(path)
        assert libraries == provenance['libraries']
        binaries.append(binary)
    for name, expected in provenance['fixtures'].items():
        assert sha(root / name) == expected, name
    data = json.loads((root / 'tests/document_lights.json').read_text())
    peer, profiles = binaries
    def query(mode, xml):
        value = subprocess.run([str(peer), mode], input=xml, capture_output=True,
                               text=True, check=True, timeout=10)
        return dict(lines=[line.split() for line in value.stdout.splitlines()], stderr=value.stderr)
    for row in data['fields']:
        assert query('--light', row['xml']) == row['native'], row['id']
    for key in ['exact_old_documents', 'typed_documents', 'document_shapes']:
        for row in data[key]:
            assert query('--single-document', row['xml']) == row['native'], row
    row = data['offset_document']
    assert query('--offset-document', row['xml']) == row['native']
    projection = project_fixture((upstream / 'test/sdf/world_complete.sdf').read_text())
    assert projection.encode() == (root / 'tests/fixtures/light_documents_original.sdf').read_bytes()
    assert query('--document', projection) == data['projection']
    assert query('--document', (upstream / 'test/sdf/world_complete.sdf').read_text()) == data['projection']
    value = subprocess.run([str(profiles)], capture_output=True, text=True, check=True, timeout=10)
    assert dict(stdout=value.stdout, stderr=value.stderr) == data['raw_schema_profiles']
    subprocess.run(['cargo', 'test', '--locked', '--test', 'document_lights', '--test', 'document_lights_original'], cwd=root, check=True)
    env = os.environ.copy()
    env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(libdir)
    original = subprocess.run([str(upstream / 'build/bin/INTEGRATION_light_dom'), '--gtest_filter=DOMWorld.LoadLights'], env=env, capture_output=True, text=True, check=True, timeout=30)
    assert '[  PASSED  ] 1 test.' in original.stdout, original.stdout
    assert not original.stderr, original.stderr
    output.write_text(json.dumps(dict(schema_cases=len(data['fields']), document_controls=11,
        projection_matches_full_native=True, unchanged_originals=1, libraries=libraries), indent=2)+'\n')
    print('Passed 42 schema cases, document controls, full/projection equivalence and unchanged LoadLights original')


def project_fixture(text):
    def element(needle, start=0):
        begin = text.index(needle, start)
        tag = needle[1:].split()[0].rstrip('>')
        end = text.index('</' + tag + '>', begin) + len(tag) + 3
        return text[begin:end], end
    def opening(needle):
        begin = text.index(needle)
        return text[begin:text.index('>', begin) + 1]
    point, _ = element('<light name="point_light"')
    spot, after_spot = element('<light name="spot_light"')
    directional, _ = element('<light name="directional_light"')
    frame1, _ = element('<frame name="frame1"')
    frame2, _ = element('<frame name="frame2"')
    link_light, _ = element('<light name="spot_light"', after_spot)
    prefix = text[:text.index('>', text.index('<world name="default"')) + 1]
    model_pose = '<pose relative_to="frame1"/>'
    assert model_pose in text
    return '\n'.join([prefix, point, spot, directional, frame1,
        opening('<model name="model1"'), model_pose, frame2,
        opening('<link name="link"'), link_light, '</link>', '</model>', '</world>', '</sdf>', ''])

if __name__ == '__main__':
    main()
