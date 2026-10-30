#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay native world unknown-child observations and loaded-library evidence."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import tempfile

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('upstream', type=Path)
parser.add_argument('--write-reference', action='store_true')
args = parser.parse_args()
upstream = args.upstream.resolve()
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
subprocess.run(['python3', str(root / 'tools/world_known_schema_inventory.py'), str(upstream)], check=True)
revision = json.loads((root / 'reference/inventory.json').read_text())['upstream_sha']
sources = ['src/parser.cc', 'src/World.cc', 'src/Element.cc', 'src/Utils.cc', 'include/sdf/Root.hh', 'include/sdf/World.hh', 'include/sdf/Element.hh']
for name in sources:
    assert (upstream / name).read_bytes() == subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':' + name])
include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
for name in sources:
    if name.startswith('include/sdf/'):
        assert sha(include / 'sdf' / Path(name).name) == sha(upstream / name)
flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
libdir = subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()
fixture = root / 'tests/world_unqualified_extensions.json'
rows = json.loads(fixture.read_text()); assert len(rows) == 21 and len({row['id'] for row in rows}) == 21
out = root / 'artifacts/world-unqualified-reference'; out.mkdir(parents=True, exist_ok=True)
probe = root / 'tests/world_merge_extensions_reference.cc'
with tempfile.TemporaryDirectory(prefix='sdf-world-unqualified-') as directory:
    temp = Path(directory); binary, world, resource, maps = (temp / name for name in ('probe', 'world.sdf', 'resource.sdf', 'loaded.maps'))
    source = probe.read_text(); assert source.count('int main(int, char **argv)') == 1 and source.endswith('}\n')
    source = source.replace('int main(int, char **argv)', 'int main(int argc, char **argv)')
    source = source.replace('  sdf::Root root;', '  if (argc != 3) return 2;\n  sdf::Root root;')
    source = '#include <fstream>\n' + source[:-2] + '\n  std::ifstream maps("/proc/self/maps");\n  std::ofstream out(argv[2]); out << maps.rdbuf();\n  return maps && out ? 0 : 3;\n}\n'
    instrumented = temp / 'instrumented.cc'; instrumented.write_text(source); instrumented_sha = sha(instrumented)
    command = ['c++', '-std=c++17', str(instrumented), *flags, '-Wl,--disable-new-dtags', '-Wl,-rpath,' + libdir, '-o', str(binary)]
    subprocess.run(command, check=True)
    linked = {}
    for line in subprocess.check_output(['ldd', str(binary)], text=True).splitlines():
        words = line.split()
        if len(words) > 2 and words[1] == '=>' and Path(words[2]).is_file():
            path = Path(words[2]).resolve(); assert path.name not in linked; linked[path.name] = sha(path)
    native = next(name for name in linked if name.startswith('libsdformat.so.')); assert linked[native] == sha(upstream / 'build/lib' / native)
    resource.write_text("<sdf version='1.12'><model name='m'><model name='n'><link name='l'/></model><v:imported/></model></sdf>")
    loaded_hashes = None; raw = []
    for row in rows:
        world.write_text(row['xml'].replace('@A@', str(resource)))
        result = subprocess.run([str(binary), str(world), str(maps)], text=True, capture_output=True, check=True, timeout=10)
        normalize = lambda value: value.replace(str(resource), '@A@').replace(str(world), '@WORLD@')
        assert normalize(result.stdout) == row['stdout'] and normalize(result.stderr) == row['stderr'], row['id']
        loaded = set()
        for line in maps.read_bytes().split(b'\n'):
            fields = line.split(maxsplit=5)
            if len(fields) == 6 and fields[5].startswith(b'/'):
                path = Path(os.fsdecode(fields[5]))
                if path.is_file() and '.so' in path.name: loaded.add(path.resolve())
        hashes = {path.name: sha(path) for path in loaded}
        assert len(hashes) == len(loaded) and all(hashes.get(name) == value for name, value in linked.items())
        if loaded_hashes is None: loaded_hashes = hashes
        assert hashes == loaded_hashes
        raw.append({'id': row['id'], 'stdout': result.stdout, 'stderr': result.stderr})
    (out / 'native-observations.json').write_text(json.dumps(raw, indent=2, ensure_ascii=False) + '\n')
    (out / 'loaded.maps').write_bytes(maps.read_bytes()); binary_sha = sha(binary)
reference = {'upstream_sha': revision, 'source_sha256': {name: sha(upstream / name) for name in sources}, 'schema_inventory_sha256': sha(root / 'reference/world-known-schema-inventory.json'), 'probe_sha256': sha(probe), 'instrumented_source_sha256': instrumented_sha, 'generator_sha256': sha(Path(__file__)), 'corpus_sha256': sha(fixture), 'native_libraries_sha256': loaded_hashes, 'observations': len(rows)}
reference_path = root / 'reference/world-unqualified-extensions.json'
if args.write_reference: reference_path.write_text(json.dumps(reference, indent=2) + '\n')
else: assert json.loads(reference_path.read_text()) == reference
(out / 'provenance.json').write_text(json.dumps({'command': command, 'binary_sha256': binary_sha}, indent=2) + '\n')
print('21 native world observations and actual loaded-library evidence passed')
