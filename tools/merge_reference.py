#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Verify model-merge semantic observations and exact native include originals."""
import argparse
import hashlib
import json
import os
import xml.etree.ElementTree as ET
from pathlib import Path
import shlex
import subprocess
import struct


def convert(tokens):
    it = iter(tokens)

    def tag(expected):
        assert next(it) == expected

    def integer():
        return int(next(it))

    def string():
        h = next(it)
        return '' if h == '-' else bytes.fromhex(h).decode()

    def values(n):
        return [struct.unpack('d', struct.pack('Q', integer()))[0] for _ in range(n)]

    def semantic():
        errors = integer()
        return dict(errors=errors, pose=values(7) if errors == 0 else None)

    def model():
        tag('M')
        result = dict(name=string(), is_static=bool(integer()), canonical=string(), relative_to=string(), raw_pose=values(7), resolved=semantic())
        tag('L')
        links = []
        for _ in range(integer()):
            link = dict(name=string(), relative_to=string(), raw_pose=values(7), resolved=semantic())
            tag('X')
            link['collisions'] = [dict(name=string(), relative_to=string(), raw_pose=values(7), resolved=semantic()) for _ in range(integer())]
            links.append(link)
        result['links'] = links
        tag('F')
        frames = []
        for _ in range(integer()):
            frame = dict(name=string(), attached_to=string(), relative_to=string(), raw_pose=values(7), resolved=semantic())
            frame['body_errors'] = integer()
            frame['body'] = string()
            frames.append(frame)
        result['frames'] = frames
        tag('J')
        joints = []
        for _ in range(integer()):
            joint = dict(name=string(), parent=string(), child=string(), relative_to=string(), raw_pose=values(7), resolved=semantic())
            axes = []
            for _ in range(2):
                if integer():
                    axis = dict(expressed_in=string(), xyz=values(3))
                    axis['errors'] = integer()
                    axis['resolved'] = values(3) if axis['errors'] == 0 else None
                    axes.append(axis)
                else:
                    axes.append(None)
            joint['axes'] = axes
            joints.append(joint)
        result['joints'] = joints
        tag('P')
        result['plugins'] = [dict(name=string(), filename=string()) for _ in range(integer())]
        tag('C')
        result['models'] = [model() for _ in range(integer())]
        return result
    errors = integer()
    if errors:
        return None
    result = model()
    assert list(it) == []
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--output', type=Path, default=Path('artifacts/merge/reference.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == json.loads((root / 'reference/inventory.json').read_text())['upstream_sha']
    output = root / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    binary = output.parent / 'merge-reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    libdir = subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()
    source = root / 'tests/merge_reference.cc'
    subprocess.run(['c++', '-std=c++17', str(source), *flags, '-Wl,-rpath,' + libdir, '-o', str(binary)], check=True)
    observations = []
    corpus = root / 'tests/merge.json'
    for case in json.loads(corpus.read_text()):
        resource = output.parent / 'resource.sdf'
        resource.write_text(case['resource'])
        (output.parent / 'grandchild.sdf').write_text(case.get('grandchild', ''))
        run = subprocess.run([str(binary), str(resource)], input=case['xml'], text=True, capture_output=True, check=True)
        tokens = run.stdout.split()
        accepted = tokens[0] == '0'
        model = convert(tokens)
        assert (accepted, model) == (case['accepted'], case['model']), (case['id'], run.stdout, run.stderr)
        observations.append(dict(id=case['id'], accepted=accepted, model=model))
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    files = ['src/parser.cc', 'src/Model.cc', 'src/FrameSemantics.cc', 'src/Param.cc',
             'src/Types.cc', 'include/sdf/Model.hh', 'include/sdf/Frame.hh',
             'include/sdf/Link.hh', 'include/sdf/Joint.hh', 'include/sdf/Plugin.hh',
             'sdf/1.12/model.sdf', 'sdf/1.12/world.sdf', 'test/integration/includes.cc']
    linked = subprocess.check_output(['ldd', str(binary)], text=True)
    native_library = next(Path(line.split('=>', 1)[1].split()[0])
                          for line in linked.splitlines() if line.strip().startswith('libsdformat.so.'))
    assert sha(native_library) == sha(upstream / 'build/lib' / native_library.name), 'installed/build native library mismatch'
    for path in files:
        expected = subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':' + path])
        assert hashlib.sha256(expected).hexdigest() == sha(upstream / path), path
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], text=True).strip())
    math_headers = [math_include / 'gz/math' / name for name in ('Vector3.hh', 'Quaternion.hh', 'Helpers.hh')]
    math_source = upstream.parent / 'gz-math'
    math_revision = subprocess.check_output(['git', '-C', str(math_source), 'rev-parse', 'HEAD'], text=True).strip()
    for header in math_headers:
        relative = 'include/gz/math/' + header.name
        expected = subprocess.check_output(['git', '-C', str(math_source), 'show', math_revision + ':' + relative])
        assert hashlib.sha256(expected).hexdigest() == sha(header) == sha(math_source / relative), header.name
    original = upstream / 'build/bin/INTEGRATION_includes'
    xml = output.parent / 'original.xml'
    env = dict(os.environ)
    env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(Path.home() / 'local/lib') + ':' + env.get('LD_LIBRARY_PATH', '')
    run = subprocess.run([str(original), '--gtest_filter=IncludesTest.MergeInclude:IncludesTest.MergeIncludePlacementFrame',
                          '--gtest_output=xml:' + str(xml)], env=env, check=True, text=True, capture_output=True)
    (output.parent / 'original.log').write_text(run.stdout + run.stderr)
    result = ET.parse(xml).getroot()
    assert result.attrib['tests'] == '2' and result.attrib['failures'] == '0'
    proof = dict(upstream_sha=revision, corpus_sha256=sha(corpus), probe_source_sha256=sha(source),
                 probe_binary_sha256=sha(binary), native_library_sha256=sha(native_library),
                 source_sha256={path: sha(upstream / path) for path in files},
                 math_upstream_sha=math_revision, math_header_sha256={path.name: sha(path) for path in math_headers},
                 original_binary_sha256=sha(original),
                 original_tests=2, original_failures=0, observations=observations,
                 matching_observations=len(observations)-sum('gap' in row for row in json.loads(corpus.read_text())),
                 explicit_gap_ids=[row['id'] for row in json.loads(corpus.read_text()) if 'gap' in row])
    output.write_text(json.dumps(proof, indent=2) + '\n')
    print(str(len(observations)) + ' actual native merge observations and both original native include cases verified; Rust schema/legacy gaps remain explicit')


if __name__ == '__main__':
    main()
