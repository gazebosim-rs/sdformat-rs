#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay Collision constructor, genuine originals and Rust XML exchanges."""
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys

root = Path(__file__).resolve().parents[1]
upstream = Path(sys.argv[1]).resolve()
reference = json.loads((root / 'reference/collision-xml.json').read_text())
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
assert subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip() == reference['upstream_sha']
for path, expected in reference['native_sources'].items():
    assert sha(upstream / path) == expected
include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
for header in ('Collision.hh', 'Geometry.hh', 'Surface.hh'):
    assert sha(include / 'sdf' / header) == sha(upstream / 'include/sdf' / header)
math = upstream.parent / 'gz-math'
assert subprocess.check_output(['git', '-C', str(math), 'rev-parse', 'HEAD'], text=True).strip() == reference['math_upstream_sha']
libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()).resolve()
flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
out = root / 'artifacts/collision-xml-reference'
out.mkdir(parents=True, exist_ok=True)
binary = out / 'native'
subprocess.run(['c++', '-std=c++17', str(root / 'tests/collision_xml_reference.cc'), *flags, '-Wl,--disable-new-dtags', f'-Wl,-rpath,{libdir}', '-o', str(binary)], check=True)
libraries = {}
for line in subprocess.check_output(['ldd', str(binary)], text=True).splitlines():
    words = line.split()
    if '=>' in words and len(words) > 2 and Path(words[2]).is_file():
        path = Path(words[2]).resolve(); libraries[path.name] = sha(path)
assert libraries == reference['libraries']
producer = subprocess.run([str(binary), 'ctor'], capture_output=True, check=True)
assert not producer.stderr
fixture = root / 'tests/fixtures/collision_to_element_native.xml'
assert producer.stdout == fixture.read_bytes()
assert sha(fixture) == reference['fixtures'][str(fixture.relative_to(root))]
exports = out / 'exports'; exports.mkdir(exist_ok=True)
for file in exports.glob('*.xml'): file.unlink()
env = dict(os.environ, SDF_COLLISION_XML_EXPORT=str(exports))
subprocess.run(['cargo', 'test', '--offline', '--locked', '--test', 'collision_xml'], cwd=root, env=env, check=True)
assert {p.name for p in exports.glob('*.xml')} == {'original.xml', 'original_density.xml'}
for file, args in [('original.xml', []), ('original_density.xml', ['density'])]:
    result = subprocess.run([str(binary), *args], input=(exports / file).read_bytes(), capture_output=True, check=True)
    assert result.stdout == b'ok\n' and not result.stderr
original_binary = upstream / 'build/bin/UNIT_Collision_TEST'
assert sha(original_binary) == reference['native_test_binary_sha256']
native_env = dict(os.environ, LD_LIBRARY_PATH=str(upstream / 'build/lib') + ':' + str(libdir))
original_libraries = {}
for line in subprocess.check_output(['ldd', str(original_binary)], env=native_env, text=True).splitlines():
    words = line.split()
    if '=>' in words and len(words) > 2 and Path(words[2]).is_file():
        path = Path(words[2]).resolve(); original_libraries[path.name] = sha(path)
assert original_libraries == reference['native_test_libraries']
originals = subprocess.run([str(original_binary), '--gtest_filter=DOMCollision.ToElement:DOMCollision.ToElementErrorOutput'], env=native_env, capture_output=True, text=True, check=True, timeout=30)
assert '[  PASSED  ] 2 tests.' in originals.stdout and not originals.stderr
print('native constructor fixture, two Rust exchanges and both native originals passed')
