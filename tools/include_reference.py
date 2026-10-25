#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Compare include provenance/plugins with the pinned native DOM and callback."""
import argparse
import hashlib
import json
from pathlib import Path
import shlex
import subprocess


def observe(raw):
    result = dict(requests=[], plugins=[], include=None)
    presence, values, plugins = {}, {}, []
    for line in raw.splitlines():
        parts = line.split('\t')
        key = parts[0]
        if key == 'accepted':
            result[key] = parts[1] == '1'
        elif key == 'model_static':
            result['is_static'] = bytes.fromhex(parts[1]).decode() == 'true'
        elif key == 'exception':
            result[key] = bytes.fromhex(parts[1]).decode()
        elif key == 'request':
            result['requests'].append(bytes.fromhex(parts[1]).decode())
        elif key in ('model_plugin', 'include_plugin'):
            target = result['plugins'] if key == 'model_plugin' else plugins
            target.append([bytes.fromhex(s).decode() for s in parts[1:]])
        elif key.startswith('has_'):
            presence[key[4:]] = parts[1] == '1'
        elif key == 'include':
            result['include_present'] = parts[1] == '1'
        elif key == 'pose_bits':
            values[key] = [int(s) for s in parts[1:]]
        elif key.startswith('include_'):
            values[key[8:]] = bytes.fromhex(parts[1]).decode()
        else:
            result[key] = bytes.fromhex(parts[1]).decode()
    if result.pop('include_present', False):
        result['include'] = dict(uri=values['uri'], name=values.get('name'),
            is_static=values['static'] == 'true' if presence['static'] else None,
            placement_frame=values.get('placement_frame'), pose_bits=values.get('pose_bits'),
            pose_relative_to=result.pop('pose_relative_to', None), plugins=plugins)
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--output', type=Path, default=Path('artifacts/include/reference.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == json.loads((root / 'reference/inventory.json').read_text())['upstream_sha']
    output = root / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    binary = output.parent / 'include-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    libdir = subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()
    source = root / 'tests/include_reference.cc'
    subprocess.run(['c++', '-std=c++17', str(source), *flags, '-Wl,-rpath,' + libdir, '-o', str(binary)], check=True)
    corpus = root / 'tests/include.json'
    observations = []
    for case in json.loads(corpus.read_text()):
        resource = root / 'tests/fixtures' / case.get('resource', 'include_resource.sdf')
        run = subprocess.run([str(binary), str(resource)], input=case['xml'], text=True, capture_output=True, check=True)
        measured = observe(run.stdout)
        assert measured == case['expected'], (case['id'], measured, case['expected'])
        observations.append(dict(id=case['id'], **measured))
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    files = ['src/parser.cc', 'src/Model.cc', 'src/Element.cc', 'sdf/1.12/model.sdf', 'sdf/1.12/world.sdf', 'sdf/1.12/plugin.sdf',
             'test/integration/model_dom.cc', 'test/integration/nested_model.cc']
    for path in files:
        pinned = subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':' + path])
        assert hashlib.sha256(pinned).hexdigest() == sha(upstream / path), path
    linked = subprocess.check_output(['ldd', str(binary)], text=True)
    library = next(Path(line.split('=>', 1)[1].split()[0])
                   for line in linked.splitlines() if line.strip().startswith('libsdformat.so.'))
    assert sha(library) == sha(upstream / 'build/lib' / library.name), 'native build/install library mismatch'
    proof = dict(upstream_sha=revision, corpus_sha256=sha(corpus), probe_source_sha256=sha(source),
        probe_binary_sha256=sha(binary), native_library_sha256=sha(library), source_sha256={path: sha(upstream / path) for path in files},
        resources_sha256={case.get('resource', 'include_resource.sdf'): sha(root / 'tests/fixtures' / case.get('resource', 'include_resource.sdf')) for case in json.loads(corpus.read_text())}, observations=observations)
    output.write_text(json.dumps(proof, indent=2) + '\n')
    print(str(len(observations)) + ' actual native include metadata observations pass')


if __name__ == '__main__':
    main()
