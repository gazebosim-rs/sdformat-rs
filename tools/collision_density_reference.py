#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay native Collision density presence, scalar and setter boundaries."""
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
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = root / 'artifacts/collision-density-reference'
    output.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    reference = json.loads((root / 'reference/collision-density.json').read_text())
    assert subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip() == reference['upstream_sha']
    for path, expected in reference['native_sources'].items():
        assert sha(upstream / path) == expected, path
    include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    assert sha(include / 'sdf/Collision.hh') == sha(upstream / 'include/sdf/Collision.hh')
    math = upstream.parent / 'gz-math'
    assert subprocess.check_output(['git', '-C', str(math), 'rev-parse', 'HEAD'], text=True).strip() == reference['math_upstream_sha']
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], text=True).strip())
    for path, expected in reference['math_sources'].items():
        assert sha(math / path) == expected, path
        assert sha(math_include / path.removeprefix('include/')) == expected, path
    libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()).resolve()
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    binary = output / 'collision-density-reference'
    subprocess.run(['c++', '-std=c++17', str(root / 'tests/collision_density_reference.cc'), *flags,
                    '-Wl,--disable-new-dtags', f'-Wl,-rpath,{libdir}', '-o', str(binary)], check=True)
    libraries = {}
    for line in subprocess.check_output(['ldd', str(binary)], text=True).splitlines():
        words = line.split()
        if '=>' in words and len(words) > 2 and Path(words[2]).is_file():
            path = Path(words[2]).resolve(); libraries[path.name] = sha(path)
    assert libraries == reference['libraries']
    def query(xml='', arguments=()):
        value = subprocess.run([str(binary), *arguments], input=xml, capture_output=True,
                               text=True, check=True, timeout=10)
        return dict(stdout=value.stdout, stderr=value.stderr)
    data=json.loads((root/'tests/collision_density.json').read_text())
    assert len(data)==14
    for row in data:
        assert query(row['xml'])==dict(stdout=row['stdout'],stderr=row['stderr']),row['id']
    setters=json.loads((root/'tests/collision_density_setters.json').read_text())
    assert query(arguments=['setters'])==setters
    subprocess.run(['cargo','test','--offline','--locked','--test','collision_density','--test','collision_ownership_originals','--test','link_ownership_originals'],cwd=root,check=True)
    result=dict(native_cases=len(data),native_setter_values=10,rust_checks=15,libraries=libraries)
    (output/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(result)

if __name__=='__main__':main()
