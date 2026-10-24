#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Verify scalar/Pose stod and distinct vector behavior against the pinned native DOM."""
import argparse
import hashlib
import json
from pathlib import Path
import shlex
import subprocess


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--output', type=Path, default=Path('artifacts/scalar-reference.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == json.loads((root / 'reference/inventory.json').read_text())['upstream_sha']
    output = root / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    binary = output.parent / 'scalar-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    libdir = subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()
    source = root / 'tests/scalar_reference.cc'
    subprocess.run(['c++', '-std=c++17', str(source), *flags, '-Wl,-rpath,' + libdir, '-o', str(binary)], check=True)
    observations = []
    corpus = root / 'tests/scalar.json'
    for case in json.loads(corpus.read_text()):
        run = subprocess.run([str(binary)], input=case['xml'], text=True, capture_output=True, check=True)
        raw = run.stdout.strip()
        accepted = raw != 'error'
        bits = int(raw) if accepted else None
        assert (accepted, bits) == (case['native_accepted'], case['bits']), (case['id'], run.stdout, run.stderr)
        observations.append(dict(id=case['id'], accepted=accepted, bits=bits, supported=case['supported']))
    abi = output.parent / 'scalar-abi.c'
    abi.write_text("""#define _GNU_SOURCE
#include <stdlib.h>
#include <locale.h>
#include <stdint.h>
typedef double (*conversion)(const char *, char **, locale_t);
typedef locale_t (*creation)(int, const char *, locale_t);
typedef void (*cleanup)(locale_t);
_Static_assert(__builtin_types_compatible_p(__typeof__(&strtod_l), conversion), "strtod_l ABI");
_Static_assert(__builtin_types_compatible_p(__typeof__(&newlocale), creation), "newlocale ABI");
_Static_assert(__builtin_types_compatible_p(__typeof__(&freelocale), cleanup), "freelocale ABI");
_Static_assert(sizeof(double) == sizeof(uint64_t), "f64 storage");
""")
    subprocess.run(['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', '-fsyntax-only', str(abi)], check=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    files = ['src/Param.cc', 'src/Sensor.cc', 'src/Imu.cc', 'include/sdf/Param.hh']
    linked = subprocess.check_output(['ldd', str(binary)], text=True)
    native_library = next(Path(line.split('=>', 1)[1].split()[0])
                          for line in linked.splitlines() if line.strip().startswith('libsdformat.so.'))
    assert sha(native_library) == sha(upstream / 'build/lib' / native_library.name), 'installed/build native library mismatch'
    for path in files:
        expected = subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':' + path])
        assert hashlib.sha256(expected).hexdigest() == sha(upstream / path), path
    proof = dict(upstream_sha=revision, corpus_sha256=sha(corpus), probe_source_sha256=sha(source),
                 probe_binary_sha256=sha(binary), native_library_sha256=sha(native_library), source_sha256={path: sha(upstream / path) for path in files},
                 stdlib_header_sha256=sha(Path('/usr/include/stdlib.h')), abi_check_sha256=sha(abi),
                 fault_shim_sha256=sha(root / 'tests/locale_failure.c'),
                 libc_version=subprocess.check_output(['ldd', '--version'], text=True).splitlines()[0],
                 observations=observations)
    output.write_text(json.dumps(proof, indent=2) + '\n')
    print(str(len(observations)) + ' actual native observations; two nonfinite cases remain explicit unsupported gaps')


if __name__ == '__main__':
    main()
