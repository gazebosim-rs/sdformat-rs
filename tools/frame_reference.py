#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Compare sensor frame metadata with the pinned native DOM, preserving empty legacy values."""
import argparse
import hashlib
import json
from pathlib import Path
import shlex
import subprocess


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--output', type=Path, default=Path('artifacts/frame-reference.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    inventory = json.loads((root / 'reference/inventory.json').read_text())
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == inventory['upstream_sha'], 'native source revision mismatch'
    output = root / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    binary = output.parent / 'frame-reference'
    source = root / 'tests/frame_id_reference.cc'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    libdir = subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()
    subprocess.run(['c++', '-std=c++17', str(source), *flags, '-Wl,-rpath,' + libdir, '-o', str(binary)], check=True)
    observations = []
    corpus = root / 'tests/frame_id.json'
    for case in json.loads(corpus.read_text()):
        run = subprocess.run([str(binary)], input=case['xml'], text=True, capture_output=True, check=True)
        lines = run.stdout.splitlines()
        assert len(lines) == 4 and lines[0] == 'accepted', (case['id'], run.stdout, run.stderr)
        measured = dict(frame_id=bytes.fromhex(lines[1]).decode(),
                        gz_frame_id=bytes.fromhex(lines[3]).decode() if lines[2] == '1' else None)
        assert measured == {key: case[key] for key in measured}, (case['id'], measured)
        observations.append(dict(id=case['id'], **measured))
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    files = ['src/Sensor.cc', 'include/sdf/Sensor.hh', 'sdf/1.12/sensor.sdf']
    proof = dict(upstream_sha=revision, corpus_sha256=sha(corpus), probe_source_sha256=sha(source),
                 probe_binary_sha256=sha(binary), source_sha256={path: sha(upstream / path) for path in files},
                 linked_libraries=subprocess.check_output(['ldd', str(binary)], text=True), observations=observations)
    output.write_text(json.dumps(proof, indent=2) + '\n')
    print(str(len(observations)) + ' native frame metadata observations pass')


if __name__ == '__main__':
    main()
