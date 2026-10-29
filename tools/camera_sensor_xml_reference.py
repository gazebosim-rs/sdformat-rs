#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay bounded canonical Camera Sensor envelopes against the native SDK."""
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
    parser.add_argument('--output', type=Path, default=Path('artifacts/camera-sensor-xml-reference/result.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = (root / args.output).resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    provenance = json.loads((root / 'reference/camera-sensor-xml.json').read_text())
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == provenance['upstream_sha']
    includedir = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    assert sha(includedir / 'sdf/Camera.hh') == sha(upstream / 'include/sdf/Camera.hh')
    for path, expected in provenance['native_sources'].items():
        assert sha(upstream / path) == expected, path
    math = upstream.parent / 'gz-math'
    assert subprocess.check_output(['git', '-C', str(math), 'rev-parse', 'HEAD'], text=True).strip() == provenance['math_upstream_sha']
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], text=True).strip())
    for path, expected in provenance['math_sources'].items():
        assert sha(math / path) == expected, path
        assert sha(math_include / path.removeprefix('include/')) == expected, path
    libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()).resolve()
    binary = output.parent / 'camera-sensor-xml-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    command = ['c++', '-std=c++17', str(root / 'tests/camera_sensor_xml_reference.cc'), *flags,
               '-Wl,--disable-new-dtags', f'-Wl,-rpath,{libdir}', '-o', str(binary)]
    subprocess.run(command, check=True)
    linkage = subprocess.check_output(['ldd', str(binary)], text=True)
    libraries = {}
    for line in linkage.splitlines():
        words = line.split()
        if '=>' in words and len(words) > 2 and Path(words[2]).is_file():
            path = Path(words[2]).resolve()
            libraries[path.name] = sha(path)
    assert libraries == provenance['libraries']
    fixture = root / 'tests/fixtures/camera_to_element_native.xml'
    assert sha(fixture) == provenance['fixture_sha256']
    rows = json.loads((root / 'tests/camera_sensor_xml.json').read_text())
    for row in rows:
        options = ['--document'] if row.get('mode') == 'document' else []
        result = subprocess.run([str(binary), *options], input=row['xml'], capture_output=True,
                                text=True, check=True, timeout=10)
        assert result.stdout.split() == row['tokens'], (row['id'], result.stdout, result.stderr)
    exports = output.parent / 'rust-outputs'
    exports.mkdir(exist_ok=True)
    for path in exports.glob('*.xml'):
        path.unlink()
    with (output.parent / 'rust-tests.log').open('w') as log:
        subprocess.run(['cargo', 'test', '--locked', '--test', 'camera_sensor_xml',
                        '--test', 'camera_base_xml', '--test', 'camera_base_xml_original'], cwd=root,
                       env=dict(os.environ, SDF_CAMERA_SENSOR_XML_EXPORTS=str(exports)),
                       check=True, stdout=log, stderr=subprocess.STDOUT)
    import struct
    as_float = lambda value: struct.unpack('d', struct.pack('Q', int(value)))[0]
    exchanges = 0
    expected_by_id = {row['id']: row['tokens'] for row in rows if row['tokens'][0] == 'OK'}
    checks = [(name, tokens) for name, tokens in expected_by_id.items()]
    checks.extend([
        ('edited-authoritative', expected_by_id['typed_edited']),
        ('full-document-authoritative', expected_by_id['full_document_original_boundary']),
    ])
    for name, expected in checks:
        native = subprocess.run([str(binary)], input=(exports / (name + '.xml')).read_bytes(),
                                capture_output=True, check=True, timeout=10)
        actual = native.stdout.decode().split()
        assert len(actual) == len(expected) == 84
        for index, (observed, wanted) in enumerate(zip(actual, expected)):
            if 10 <= index < 17 or 32 <= index < 39:
                assert abs(as_float(observed) - as_float(wanted)) <= 1e-3, (name, index)
            else:
                assert observed == wanted, (name, index)
        exchanges += 1
    original = upstream / 'build/bin/UNIT_Camera_TEST'
    result = subprocess.run([str(original), '--gtest_filter=DOMCamera.ToElement'], capture_output=True, text=True, check=True, timeout=30)
    assert '[  PASSED  ] 1 test.' in result.stdout
    assert not result.stderr, result.stderr
    output.write_text(json.dumps(dict(upstream=revision, command=command, libraries=libraries,
        corpus_cases=len(rows), fixture_sha256=sha(fixture), rust_to_native_exchanges=exchanges,
        native_stdout=result.stdout, native_stderr=result.stderr), indent=2) + '\n')
    print(f'Passed {len(rows)} native Camera Sensor cases, {exchanges} actual exchanges and unchanged Camera.ToElement')



if __name__ == '__main__':
    main()
