#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Check native Light input-origin lifecycle without pointer-equivalence claims."""
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
    parser.add_argument('--output', type=Path, default=Path('artifacts/light-origin-reference/result.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = (root / args.output).resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    provenance = json.loads((root / 'reference/light-origin.json').read_text())
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
    binary = output.parent / 'light-origin-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    command = ['c++', '-std=c++17', str(root / 'tests/light_origin_reference.cc'), *flags,
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
    result = subprocess.run([str(binary)], capture_output=True, text=True, check=True, timeout=10)
    assert result.stdout == provenance['native_expected'], result.stdout
    assert not result.stderr, result.stderr
    subprocess.run(['cargo', 'test', '--locked', '--test', 'light_origin', '--test', 'light_roundtrip_originals'], cwd=root, check=True)
    native = upstream / 'build/bin/UNIT_Light_TEST'
    env = os.environ.copy()
    env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(libdir)
    original = subprocess.run([str(native), '--gtest_filter=DOMLight.ToElement:DOMLight.ToElementErrorOutput'], env=env, capture_output=True, text=True, check=True, timeout=30)
    assert '[  PASSED  ] 2 tests.' in original.stdout, original.stdout
    assert not original.stderr, original.stderr
    output.write_text(json.dumps({'upstream_sha':revision, 'libraries':libraries, 'native_origin':result.stdout, 'unchanged_originals':2}, indent=2)+'\n')
    print('Passed native constructed/loaded Light origin controls and two unchanged originals')

if __name__ == '__main__':
    main()
