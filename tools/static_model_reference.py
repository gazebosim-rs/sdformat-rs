#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay static-model graph sinks and one unchanged nested-static original."""
import argparse
import hashlib
import json
import os
import shlex
from pathlib import Path
import subprocess
import xml.etree.ElementTree as ET


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--output', type=Path, default=Path('artifacts/static-model/reference.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = root / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    disposition = json.loads((root / 'reference/static_model_disposition.json').read_text())
    for path, change in disposition['files'].items():
        restored = (root / path).read_text()
        for edit in change['edits']:
            assert edit['after'] and edit['after'] in restored, path
            restored = restored.replace(edit['after'], edit['before'], 1)
        assert hashlib.sha256(restored.encode()).hexdigest() == change['parent_sha256'], path
    for path, expected in disposition['unchanged_input_sha256'].items():
        assert sha(root / path) == expected, path
    rows = json.loads((root / 'tests/static_model.json').read_text())
    retained = {row['id']: hashlib.sha256(row['xml'].encode()).hexdigest()
                for row in rows if row['id'].startswith('old_')}
    assert retained == disposition['retained_xml_sha256']
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == json.loads((root / 'reference/inventory.json').read_text())['upstream_sha']
    headers = ['Root.hh', 'World.hh', 'Model.hh', 'Link.hh', 'Frame.hh', 'Joint.hh',
               'SemanticPose.hh', 'Element.hh', 'Error.hh', 'Types.hh', 'InterfaceElements.hh',
               'InterfaceFrame.hh', 'InterfaceJoint.hh', 'InterfaceLink.hh', 'InterfaceModel.hh']
    files = ['src/FrameSemantics.cc', 'src/FrameSemantics.hh', 'src/ScopedGraph.hh',
             'src/Utils.hh', 'src/Model.cc', 'src/Frame.cc', 'src/Joint.cc',
             'src/FrameSemantics_TEST.cc', 'test/sdf/model_nested_static_model.sdf']
    files += ['include/sdf/' + name for name in headers]
    fixtures = ['model_nested_static_model.sdf']
    for path in files:
        expected = subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':' + path])
        assert hashlib.sha256(expected).hexdigest() == sha(upstream / path), path
    for name in fixtures:
        assert sha(root / 'tests/fixtures' / name) == sha(upstream / 'test/sdf' / name)
    include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    for name in headers:
        assert sha(include / 'sdf' / name) == sha(upstream / 'include/sdf' / name)
    binary = upstream / 'build/bin/UNIT_FrameSemantics_TEST'
    env = dict(os.environ)
    env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(Path.home() / 'local/lib') + ':' + env.get('LD_LIBRARY_PATH', '')
    env.pop('SDF_PATH', None)
    probe_source = root / 'tests/static_model_reference.cc'
    probe = output.parent / 'static-model-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], env=env, text=True))
    libdir = subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], env=env, text=True).strip()
    subprocess.run(['c++', '-std=c++17', '-Wall', '-Wextra', '-Werror', '-I' + str(upstream / 'src'), str(probe_source),
                    str(upstream / 'src/FrameSemantics.cc'), *flags,
                    '-Wl,-rpath,' + libdir, '-o', str(probe)], env=env, check=True)
    corpus = root / 'tests/static_model.json'
    observations = []
    for row in json.loads(corpus.read_text()):
        run = subprocess.run([str(probe)], input=row['xml'], text=True,
                             capture_output=True, check=True, env=env)
        assert run.stdout.split() == row['tokens'], (row['id'], run.stdout, run.stderr)
        observations.append(dict(id=row['id'], tokens=run.stdout.split()))
    originals = []
    for case in ['ModelWithoutLinksWithNestedStaticModel']:
        report = output.parent / (case + '.xml')
        run = subprocess.run([str(binary), '--gtest_filter=NestedFrameSemantics.' + case, '--gtest_output=xml:' + str(report)], env=env, text=True, capture_output=True, check=True)
        (output.parent / (case + '.log')).write_text(run.stdout + run.stderr)
        result = ET.parse(report).getroot()
        assert result.attrib['tests'] == '1' and result.attrib['failures'] == '0'
        assert int(result.attrib.get('errors', '0')) == 0
        assert int(result.attrib.get('disabled', '0')) == 0
        cases = result.findall('.//testcase')
        assert len(cases) == 1 and cases[0].attrib['name'] == case
        assert cases[0].attrib.get('status') == 'run' and cases[0].find('skipped') is None
        originals.append(dict(case='NestedFrameSemantics.' + case, tests=1, failures=0, skipped=0))
    libraries = {}
    for line in subprocess.check_output(['ldd', str(binary)], env=env, text=True).splitlines():
        if '=>' in line and any(line.strip().startswith(name) for name in ('libsdformat.so.', 'libgz-math.so.', 'libstdc++.so.', 'libc.so.')):
            path = Path(line.split('=>', 1)[1].split()[0])
            libraries[path.name] = sha(path)
            if path.name.startswith('libsdformat.so.'):
                assert sha(path) == sha(upstream / 'build/lib' / path.name)
    math_source = upstream.parent / 'gz-math'
    math_revision = subprocess.check_output(['git', '-C', str(math_source), 'rev-parse', 'HEAD'], text=True).strip()
    math_files = ['include/gz/math/' + name for name in ['Pose3.hh', 'Quaternion.hh', 'Vector3.hh']]
    math_files += [str(path.relative_to(math_source)) for path in sorted((math_source / 'include/gz/math/graph').glob('*.hh'))]
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], text=True).strip())
    for path in math_files:
        expected = subprocess.check_output(['git', '-C', str(math_source), 'show', math_revision + ':' + path])
        assert hashlib.sha256(expected).hexdigest() == sha(math_source / path), path
        assert sha(math_include / Path(path).relative_to('include')) == sha(math_source / path), path
    proof = dict(upstream_sha=revision, corpus_sha256=sha(corpus), probe_source_sha256=sha(probe_source),
                 probe_binary_sha256=sha(probe), observations=observations,
                 disposition_sha256=sha(root / 'reference/static_model_disposition.json'), source_sha256={path: sha(upstream / path) for path in files},
                 fixture_sha256={name: sha(root / 'tests/fixtures' / name) for name in fixtures},
                 original_binary_sha256=sha(binary), linked_libraries_sha256=libraries,
                 math_upstream_sha=math_revision, math_header_sha256={path: sha(math_source / path) for path in math_files},
                 originals=originals, qualification='Fresh helper plus immutable FrameSemantics.cc compilation; existing pinned original test binary and installed SDK/library/header identity. Root DOM graph uses the installed library, while direct graph queries use the freshly compiled source. No fresh full native rebuild or reproducible image claim.')
    output.write_text(json.dumps(proof, indent=2) + '\n')
    print('61 native static-model observations and one unchanged original verified')


if __name__ == '__main__':
    main()
