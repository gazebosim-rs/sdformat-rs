#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Compare canonical DOM link selection with the pinned native DOM, independently of attachment semantics."""
import argparse
import hashlib
import json
from pathlib import Path
import shlex
import subprocess
import xml.etree.ElementTree as ET


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--output', type=Path, default=Path('artifacts/canonical-reference.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    inventory = json.loads((root / 'reference/inventory.json').read_text())
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == inventory['upstream_sha'], 'native source revision mismatch'
    output = root / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    binary = output.parent / 'canonical-reference'
    source = root / 'tests/canonical_reference.cc'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    libdir = subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()
    subprocess.run(['c++', '-std=c++17', str(source), *flags, '-Wl,-rpath,' + libdir, '-o', str(binary)], check=True)
    observations = []
    corpus = root / 'tests/canonical.json'
    for case in json.loads(corpus.read_text()):
        run = subprocess.run([str(binary)], input=case['xml'], text=True, capture_output=True, check=True)
        lines = run.stdout.splitlines()
        assert len(lines) == 3, (case['id'], run.stdout, run.stderr)
        measured = dict(name=bytes.fromhex(lines[1]).decode(),
                        link=bytes.fromhex(lines[2]).decode() if lines[0] == '1' else None)
        assert measured == {key: case[key] for key in measured}, (case['id'], measured)
        observations.append(dict(id=case['id'], **measured))
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    files = ['src/Model.cc', 'include/sdf/Model.hh']
    proof = dict(upstream_sha=revision, corpus_sha256=sha(corpus), probe_source_sha256=sha(source),
                 probe_binary_sha256=sha(binary), source_sha256={path: sha(upstream / path) for path in files},
                 linked_libraries=subprocess.check_output(['ldd', str(binary)], text=True), observations=observations)
    native = upstream / 'build/bin/INTEGRATION_model_dom'
    report = output.parent / 'canonical-original.xml'
    case = 'DOMRoot.LoadNestedExplicitCanonicalLink'
    run = subprocess.run([str(native), '--gtest_filter=' + case, '--gtest_output=xml:' + str(report)],
                         text=True, capture_output=True, check=True)
    tests = [(suite.attrib['name'] + '.' + test.attrib['name'], test)
             for suite in ET.parse(report).getroot() for test in suite.findall('testcase')]
    assert len(tests) == 1 and tests[0][0] == case and tests[0][1].attrib.get('status') == 'run'
    assert not list(tests[0][1]), (run.stdout, run.stderr)
    proof.update(original_native_test=case, original_native_binary_sha256=sha(native),
                 original_source_sha256=sha(upstream / 'test/integration/model_dom.cc'))
    output.write_text(json.dumps(proof, indent=2) + '\n')
    print(str(len(observations)) + ' native canonical DOM observations pass')


if __name__ == '__main__':
    main()
