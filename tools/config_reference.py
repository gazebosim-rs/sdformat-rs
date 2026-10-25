#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay pure model.config observations through the pinned native filesystem API."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import resource
import shlex
import subprocess
import xml.etree.ElementTree as ET


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--output', type=Path, default=Path('artifacts/config/reference.json'))
    args = parser.parse_args()
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = root / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == json.loads((root / 'reference/inventory.json').read_text())['upstream_sha']
    source = root / 'tests/config_reference.cc'
    binary = output.parent / 'config-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    libdir = subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()
    subprocess.run(['c++', '-std=c++17', str(source), *flags, '-Wl,-rpath,' + libdir, '-o', str(binary)], check=True)
    sdk_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    assert sha(sdk_include / 'sdf/parser.hh') == sha(upstream / 'include/sdf/parser.hh')
    corpus = root / 'tests/model_config.json'
    observations = []
    for case in json.loads(corpus.read_text()):
        run = subprocess.run([str(binary), str(output.parent / 'input')], input=case['xml'], text=True, capture_output=True)
        actual = dict(returncode=run.returncode, stdout=run.stdout, stderr=run.stderr)
        assert actual == {key: case[key] for key in actual}, (case['id'], actual)
        observations.append(dict(id=case['id'], **actual))
    files = ['src/parser.cc', 'include/sdf/parser.hh', 'test/integration/model_versions.cc']
    fixtures = {'tests/fixtures/config_versions.config': 'test/integration/model/cococan/model.config',
                'tests/fixtures/config_no_version.config': 'test/integration/model/cococan_noversiontag/model.config'}
    files.extend(fixtures.values())
    for path in files:
        expected = subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':' + path])
        assert hashlib.sha256(expected).hexdigest() == sha(upstream / path), path
    for local, original in fixtures.items():
        assert sha(root / local) == sha(upstream / original), local
    math_source = upstream.parent / 'gz-math'
    math_revision = subprocess.check_output(['git', '-C', str(math_source), 'rev-parse', 'HEAD'], text=True).strip()
    math_files = ['src/SemanticVersion.cc', 'include/gz/math/SemanticVersion.hh']
    for path in math_files:
        expected = subprocess.check_output(['git', '-C', str(math_source), 'show', math_revision + ':' + path])
        assert hashlib.sha256(expected).hexdigest() == sha(math_source / path), path
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], text=True).strip())
    assert sha(math_include / 'gz/math/SemanticVersion.hh') == sha(math_source / math_files[1])
    linked = subprocess.check_output(['ldd', str(binary)], text=True)
    libraries = {}
    for line in linked.splitlines():
        if '=>' in line and any(line.strip().startswith(name) for name in ('libsdformat.so.', 'libgz-math.so.', 'libtinyxml2.so.', 'libstdc++.so.', 'libc.so.')):
            path = Path(line.split('=>', 1)[1].split()[0])
            libraries[path.name] = sha(path)
            if path.name.startswith('libsdformat.so.'):
                assert sha(path) == sha(upstream / 'build/lib' / path.name)
    original = upstream / 'build/bin/INTEGRATION_model_versions'
    report = output.parent / 'original.xml'
    env = dict(os.environ)
    env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(Path.home() / 'local/lib') + ':' + env.get('LD_LIBRARY_PATH', '')
    run = subprocess.run([str(original), '--gtest_filter=ModelVersionsTest.NoVersionTag_ModelFilePath:ModelVersionsTest.Correct_ModelFilePath', '--gtest_output=xml:' + str(report)], env=env, text=True, capture_output=True, check=True)
    (output.parent / 'original.log').write_text(run.stdout + run.stderr)
    result = ET.parse(report).getroot()
    assert result.attrib['tests'] == '2' and result.attrib['failures'] == '0'
    proof = dict(upstream_sha=revision, corpus_sha256=sha(corpus), probe_source_sha256=sha(source),
                 probe_binary_sha256=sha(binary), linked_libraries_sha256=libraries,
                 source_sha256={path: sha(upstream / path) for path in files},
                 math_upstream_sha=math_revision, math_source_sha256={path: sha(math_source / path) for path in math_files},
                 fixture_sha256={path: sha(root / path) for path in fixtures},
                 original_binary_sha256=sha(original), original_tests=2, original_failures=0,
                 explicit_xml_gap_ids=[case['id'] for case in json.loads(corpus.read_text()) if 'gap' in case],
                 safe_warning_ids=['missing_name_future', 'empty_name'],
                 safe_error_ids=[case['id'] for case in json.loads(corpus.read_text()) if case['stdout'].startswith('exception') and case['id'] != 'empty_name'],
                 observations=observations)
    output.write_text(json.dumps(proof, indent=2) + '\n')
    print(f'{len(observations)} native config observations and two exact originals verified; safe error/warning boundary differences explicit')


if __name__ == '__main__':
    main()
