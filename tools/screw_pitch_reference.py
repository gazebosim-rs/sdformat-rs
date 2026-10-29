#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Reproduce native screw-pitch unit observations and the original fixture test."""
import argparse
import hashlib
import json
from pathlib import Path
import shlex
import subprocess


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--output', type=Path, default=Path('artifacts/screw-pitch-reference/result.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = (root / args.output).resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == json.loads((root / 'reference/inventory.json').read_text())['upstream_sha']
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    includedir = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    assert sha(includedir / 'sdf/Joint.hh') == sha(upstream / 'include/sdf/Joint.hh')
    fixture = root / 'tests/fixtures/joint_screw_thread_pitch.sdf'
    assert fixture.read_bytes() == (upstream / 'test/sdf/joint_screw_thread_pitch.sdf').read_bytes()
    libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()).resolve()
    binary = output.parent / 'screw-pitch-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    command = ['c++', '-std=c++17', str(root / 'tests/screw_pitch_reference.cc'), *flags,
               '-Wl,--disable-new-dtags', f'-Wl,-rpath,{libdir}', '-o', str(binary)]
    subprocess.run(command, check=True)
    rows = json.loads((root / 'tests/screw_pitch.json').read_text())
    for row in rows:
        result = subprocess.run([str(binary)], input=row['xml'], capture_output=True, text=True, check=True, timeout=10)
        assert result.stdout.split() == row['tokens'], (row['id'], result.stdout, result.stderr)
    native = upstream / 'build/bin/INTEGRATION_joint_dom'
    result = subprocess.run([str(native), '--gtest_filter=DOMJoint.ScrewThreadPitch'], capture_output=True, text=True, check=True, timeout=30)
    assert '[  PASSED  ] 1 test.' in result.stdout
    linkage = subprocess.check_output(['ldd', str(binary)], text=True)
    libraries = {}
    for line in linkage.splitlines():
        words = line.split()
        if '=>' in words and len(words) > 2 and Path(words[2]).is_file():
            path = Path(words[2]).resolve()
            libraries[str(path)] = sha(path)
    assert any(Path(path).name.startswith('libsdformat') and Path(path).parent == libdir for path in libraries)
    output.write_text(json.dumps(dict(upstream=revision, command=command, binary_sha256=sha(binary),
        libraries=libraries, corpus_cases=len(rows), fixture_sha256=sha(fixture),
        native_stdout=result.stdout, native_stderr=result.stderr), indent=2) + '\n')
    print(f'Passed {len(rows)} native pitch cases and DOMJoint.ScrewThreadPitch')


if __name__ == '__main__':
    main()
