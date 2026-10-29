#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay typed Contact XML and both directions of native exchange."""
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
    parser.add_argument('--output', type=Path, default=Path('artifacts/contact-xml-reference/result.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = (root / args.output).resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    provenance = json.loads((root / 'reference/contact-xml.json').read_text())
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == provenance['upstream_sha']
    includedir = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    assert sha(includedir / 'sdf/Sensor.hh') == sha(upstream / 'include/sdf/Sensor.hh')
    assert sha(includedir / 'sdf/Element.hh') == sha(upstream / 'include/sdf/Element.hh')
    for path, expected in provenance['native_sources'].items():
        assert sha(upstream / path) == expected, path
    math = upstream.parent / 'gz-math'
    assert subprocess.check_output(['git', '-C', str(math), 'rev-parse', 'HEAD'], text=True).strip() == provenance['math_upstream_sha']
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], text=True).strip())
    for path, expected in provenance['math_sources'].items():
        assert sha(math / path) == expected, path
        assert sha(math_include / path.removeprefix('include/')) == expected, path
    libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()).resolve()
    binary = output.parent / 'contact-xml-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    command = ['c++', '-std=c++17', str(root / 'tests/contact_xml_reference.cc'), *flags,
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
    rows = json.loads((root / 'tests/contact_xml.json').read_text())
    for row in rows:
        result = subprocess.run([str(binary), row['mode']], input=row['xml'], capture_output=True,
                                text=True, check=True, timeout=10)
        assert result.stdout.split() == row['tokens'], (row['id'], result.stdout, result.stderr)
    producer = provenance['native_producer']
    result = subprocess.run([str(binary), 'producer'], input=producer['input'], capture_output=True,
                            text=True, check=True, timeout=10)
    assert result.stdout.split() == producer['tokens']
    exports = output.parent / 'rust-outputs'
    exports.mkdir(exist_ok=True)
    subprocess.run(['cargo', 'test', '--locked', '--test', 'contact_xml', '--test', 'contact_sensor'], cwd=root,
                   env=dict(os.environ, SDF_CONTACT_XML_EXPORTS=str(exports)), check=True, capture_output=True)
    import struct
    as_float = lambda value: struct.unpack('d', struct.pack('Q', int(value)))[0]
    exchanges = 0
    checks = [(row['id'], row['mode'], row['tokens']) for row in rows]
    checks.append(('edited-authoritative', 'config', next(row['tokens'] for row in rows if row['id'] == 'typed_edits')))
    for name, mode, expected in checks:
        native = subprocess.run([str(binary), mode], input=(exports / (name + '.xml')).read_bytes(),
                                capture_output=True, check=True, timeout=10)
        actual = native.stdout.decode().split()
        count_index = 1 if mode == 'config' else 17
        size = (3 if mode == 'config' else 19) + int(expected[count_index])
        assert len(actual) == len(expected) == size
        for index, (observed, wanted) in enumerate(zip(actual, expected)):
            if mode == 'sensor' and 10 <= index < 17:
                assert abs(as_float(observed) - as_float(wanted)) <= 1e-3, (name, index)
            else:
                assert observed == wanted, (name, index)
        exchanges += 1
    original = upstream / 'build/bin/INTEGRATION_link_dom'
    environment = dict(os.environ)
    environment['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(Path.home() / 'local/lib') + ':' + environment.get('LD_LIBRARY_PATH', '')
    result = subprocess.run([str(original), '--gtest_filter=DOMLink.Sensors'], env=environment,
                            capture_output=True, text=True, check=True, timeout=30)
    assert '[  PASSED  ] 1 test.' in result.stdout
    output.write_text(json.dumps(dict(upstream=revision, command=command, libraries=libraries,
        corpus_cases=len(rows), rust_to_native_exchanges=exchanges, native_producer=producer,
        native_stdout=result.stdout, native_stderr=result.stderr), indent=2) + '\n')
    print(f'Passed {len(rows)} native Contact inputs, {exchanges} actual Load exchanges, producer limitation and unchanged DOMLink.Sensors')


if __name__ == '__main__':
    main()
