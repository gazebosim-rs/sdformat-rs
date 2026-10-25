#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Verify native terrain metadata and the unchanged geometry Shapes case."""
import argparse
import hashlib
import json
import os
import xml.etree.ElementTree as ET
from pathlib import Path
import shlex
import subprocess


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--output', type=Path, default=Path('artifacts/terrain/reference.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == json.loads((root / 'reference/inventory.json').read_text())['upstream_sha']
    output = root / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    binary = output.parent / 'terrain-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    libdir = subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()
    source = root / 'tests/terrain_reference.cc'
    subprocess.run(['c++', '-std=c++17', str(source), *flags, '-Wl,-rpath,' + libdir, '-o', str(binary)], check=True)
    observations = []
    corpus = root / 'tests/terrain.json'
    for case in json.loads(corpus.read_text()):
        run = subprocess.run([str(binary)], input=case['xml'], text=True, capture_output=True, check=True)
        raw = run.stdout.strip()
        accepted = raw != 'error'
        tokens = raw.split()
        assert (accepted, tokens) == (case['accepted'], case['tokens']), (case['id'], run.stdout, run.stderr)
        observations.append(dict(id=case['id'], accepted=accepted, tokens=tokens))
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    files = ['src/Heightmap.cc', 'src/Polyline.cc', 'src/Geometry.cc', 'src/Param.cc',
             'src/Utils.cc', 'src/parser.cc', 'include/sdf/Heightmap.hh',
             'include/sdf/Polyline.hh', 'include/sdf/Geometry.hh',
             'sdf/1.12/heightmap_shape.sdf', 'sdf/1.12/polyline_shape.sdf',
             'test/integration/geometry_dom.cc', 'test/sdf/shapes.sdf']
    linked = subprocess.check_output(['ldd', str(binary)], text=True)
    native_library = next(Path(line.split('=>', 1)[1].split()[0])
                          for line in linked.splitlines() if line.strip().startswith('libsdformat.so.'))
    assert sha(native_library) == sha(upstream / 'build/lib' / native_library.name), 'installed/build native library mismatch'
    for path in files:
        expected = subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':' + path])
        assert hashlib.sha256(expected).hexdigest() == sha(upstream / path), path
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], text=True).strip())
    math_headers = [math_include / 'gz/math' / name for name in ('Vector2.hh', 'Vector3.hh', 'Helpers.hh')]
    math_source = upstream.parent / 'gz-math'
    math_revision = subprocess.check_output(['git', '-C', str(math_source), 'rev-parse', 'HEAD'], text=True).strip()
    for header in math_headers:
        relative = 'include/gz/math/' + header.name
        expected = subprocess.check_output(['git', '-C', str(math_source), 'show', math_revision + ':' + relative])
        assert hashlib.sha256(expected).hexdigest() == sha(header) == sha(math_source / relative), header.name
    fixture = root / 'tests/fixtures/shapes.sdf'
    assert sha(fixture) == sha(upstream / 'test/sdf/shapes.sdf')
    original = upstream / 'build/bin/INTEGRATION_geometry_dom'
    xml = output.parent / 'original.xml'
    env = dict(os.environ)
    env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(Path.home() / 'local/lib') + ':' + env.get('LD_LIBRARY_PATH', '')
    run = subprocess.run([str(original), '--gtest_filter=DOMGeometry.Shapes',
                          '--gtest_output=xml:' + str(xml)], env=env, check=True, text=True, capture_output=True)
    (output.parent / 'original.log').write_text(run.stdout + run.stderr)
    result = ET.parse(xml).getroot()
    assert result.attrib['tests'] == '1' and result.attrib['failures'] == '0'
    proof = dict(upstream_sha=revision, corpus_sha256=sha(corpus), probe_source_sha256=sha(source),
                 probe_binary_sha256=sha(binary), native_library_sha256=sha(native_library),
                 source_sha256={path: sha(upstream / path) for path in files},
                 math_upstream_sha=math_revision, math_header_sha256={path.name: sha(path) for path in math_headers},
                 fixture_sha256=sha(fixture), original_binary_sha256=sha(original),
                 original_tests=1, original_failures=0, observations=observations)
    output.write_text(json.dumps(proof, indent=2) + '\n')
    print(str(len(observations)) + ' actual native terrain observations and the original Shapes case match')


if __name__ == '__main__':
    main()
