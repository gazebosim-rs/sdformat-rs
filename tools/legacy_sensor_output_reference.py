#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay legacy raw-kind and native classifier output against the native SDK."""
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
    parser.add_argument('--output', type=Path, default=Path('artifacts/legacy-sensor-output-reference/result.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = (root / args.output).resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    provenance = json.loads((root / 'reference/legacy-sensor-output.json').read_text())
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == provenance['upstream_sha']
    includedir = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    assert sha(includedir / 'sdf/Sensor.hh') == sha(upstream / 'include/sdf/Sensor.hh')
    for path, expected in provenance['native_sources'].items():
        assert sha(upstream / path) == expected, path
    math = upstream.parent / 'gz-math'
    assert subprocess.check_output(['git', '-C', str(math), 'rev-parse', 'HEAD'], text=True).strip() == provenance['math_upstream_sha']
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], text=True).strip())
    for path, expected in provenance['math_sources'].items():
        assert sha(math / path) == expected, path
        assert sha(math_include / path.removeprefix('include/')) == expected, path
    libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()).resolve()
    binary = output.parent / 'legacy-sensor-output-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    command = ['c++', '-std=c++17', str(root / 'tests/legacy_sensor_output_reference.cc'), *flags,
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
    rows = json.loads((root / 'tests/legacy_sensor_output.json').read_text())
    for row in rows:
        result = subprocess.run([str(binary)], input=row['xml'], capture_output=True,
                                text=True, check=True, timeout=10)
        assert result.stdout.split() == row['tokens'], (row['id'], result.stdout, result.stderr)
    exports = output.parent / 'rust-outputs'
    exports.mkdir(exist_ok=True)
    for path in exports.glob('*.xml'):
        path.unlink()
    with (output.parent / 'rust-tests.log').open('w') as log:
        subprocess.run(['cargo', 'test', '--locked', '--test', 'legacy_sensor_output', '--test', 'navsat_xml', '--test', 'lidar_xml'], cwd=root,
                       env=dict(os.environ, SDF_LEGACY_SENSOR_OUTPUTS=str(exports)),
                       check=True, stdout=log, stderr=subprocess.STDOUT)
    import struct
    as_float = lambda value: struct.unpack('d', struct.pack('Q', int(value)))[0]
    exchanges = 0
    expected_by_id = {row['id']: row['tokens'] for row in rows if row['tokens'][0] == 'OK'}
    checks = [(name, tokens) for name, tokens in expected_by_id.items()]
    for kind in ['gps','ray','gpu_ray']:
        checks.append((kind+'-edited-authoritative', expected_by_id[kind+'_edited']))
        boundary = 'gps_old_boundary' if kind == 'gps' else kind+'_default'
        checks.append((kind+'-boundary-authoritative', expected_by_id[boundary]))
    for name, expected in checks:
        native = subprocess.run([str(binary)], input=(exports / (name + '.xml')).read_bytes(),
                                capture_output=True, check=True, timeout=10)
        actual = native.stdout.decode().split()
        assert len(actual) == len(expected)
        assert len(expected) in (38,50)
        for index, (observed, wanted) in enumerate(zip(actual, expected)):
            if 11 <= index < 18:
                assert abs(as_float(observed) - as_float(wanted)) <= 1e-3, (name, index)
            else:
                assert observed == wanted, (name, index)
        exchanges += 1
    original = upstream / 'build/bin/UNIT_Sensor_TEST'
    result = subprocess.run([str(original), '--gtest_filter=DOMSensor.Type'], capture_output=True,
                            text=True, check=True, timeout=30)
    assert '[  PASSED  ] 1 test.' in result.stdout
    output.write_text(json.dumps(dict(upstream=revision, command=command, libraries=libraries,
        corpus_cases=len(rows), rust_to_native_exchanges=exchanges,
        native_stdout=result.stdout, native_stderr=result.stderr), indent=2) + '\n')
    print(f'Passed {len(rows)} native legacy-kind inputs, {exchanges} actual outputs and unchanged DOMSensor.Type')


if __name__ == '__main__':
    main()
