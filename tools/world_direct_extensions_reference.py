#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay direct-world extension observations and focused Rust metadata checks."""
import hashlib
import json
from pathlib import Path
import shlex
import subprocess
import sys
import tempfile

root = Path(__file__).resolve().parents[1]
upstream = Path(sys.argv[1]).resolve()
reference = json.loads((root / 'reference/world-merge-extensions.json').read_text())
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
assert subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip() == reference['upstream_sha']
for name, expected in reference['native_sources'].items():
    assert sha(upstream / name) == expected
include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
for name in ('Root.hh', 'World.hh', 'Element.hh'):
    assert sha(include / 'sdf' / name) == sha(upstream / 'include/sdf' / name)
libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()).resolve()
flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
rows = json.loads((root / 'tests/world_direct_extensions.json').read_text())
assert len(rows) == 13 and len({row['id'] for row in rows}) == 13
with tempfile.TemporaryDirectory(prefix='sdf-world-merge-extensions-') as directory:
    temp = Path(directory)
    binary = temp / 'probe'
    subprocess.run(['c++', '-std=c++17', str(root / 'tests/world_merge_extensions_reference.cc'), *flags,
                    '-Wl,--disable-new-dtags', f'-Wl,-rpath,{libdir}', '-o', str(binary)], check=True)
    libraries = {}
    for line in subprocess.check_output(['ldd', str(binary)], text=True).splitlines():
        words = line.split()
        if '=>' in words and len(words) > 2 and Path(words[2]).is_file():
            path = Path(words[2]).resolve(); libraries[path.name] = sha(path)
    assert libraries == reference['libraries']
    a, b, world = (temp / name for name in ('a.sdf', 'b.sdf', 'world.sdf'))
    for row in rows:
        a.write_text("<sdf version='1.12'><model name='m'><model name='n'><link name='l'/></model><v:dup k='M'/><v:imported/></model></sdf>")
        world.write_text(row['xml'].replace('@A@', str(a)))
        value = subprocess.run([str(binary), str(world)], capture_output=True, text=True, check=True, timeout=10)
        stderr = value.stderr.replace(str(a), '@A@').replace(str(b), '@B@').replace(str(world), '@WORLD@')
        assert value.stdout == row['stdout'] and stderr == row['stderr'], row['id']
subprocess.run(['cargo', 'test', '--offline', '--locked', '--test', 'world_direct_extensions'], cwd=root, check=True)
print('13 native direct-world observations and focused Rust tests passed')
