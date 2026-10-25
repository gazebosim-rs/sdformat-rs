#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Verify the bounded 1.5 model subset and original include fixture provenance."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import xml.etree.ElementTree as ET


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--output', type=Path, default=Path('artifacts/version15-reference.json'))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    revision = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == json.loads((root / 'reference/inventory.json').read_text())['upstream_sha']
    output = root / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    paths = ['sdf/1.6/1_5.convert', 'sdf/1.7/1_6.convert', 'sdf/1.8/1_7.convert',
             'src/Converter.cc', 'src/Model.cc', 'src/parser.cc', 'test/integration/nested_model.cc',
             'test/integration/model/box/model.sdf', 'test/integration/model/test_model/model.sdf']
    for path in paths:
        original = subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':' + path])
        assert hashlib.sha256(original).hexdigest() == sha(upstream / path), path
    source = (upstream / 'test/integration/nested_model.cc').read_text()
    start = source.index('TEST(NestedModel, IncludeElements)')
    start = source.index('stream << R"(', start) + len('stream << R"(')
    original_xml = source[start:source.index(')";', start)]
    assert (root / 'tests/fixtures/include_elements.sdf').read_text() == original_xml
    fixtures = {'tests/fixtures/include_elements_box.sdf': 'test/integration/model/box/model.sdf',
                'tests/fixtures/include_elements_model.sdf': 'test/integration/model/test_model/model.sdf'}
    for local, original in fixtures.items():
        assert sha(root / local) == sha(upstream / original), local
    native = output.parent / 'version15-differential.json'
    subprocess.run(['python3', str(root / 'tools/reference_probe.py'), str(upstream),
                    '--output', str(native)], check=True, capture_output=True)
    corpus = json.loads(native.read_text())
    observations = [row for row in corpus['results'] if row['id'].startswith('v15_')]
    assert len(observations) == 62
    binary = upstream / 'build/bin/INTEGRATION_nested_model'
    xml = output.parent / 'version15-original.xml'
    env = dict(os.environ)
    env['LD_LIBRARY_PATH'] = str(upstream / 'build/lib') + ':' + str(Path.home() / 'local/lib') + ':' + env.get('LD_LIBRARY_PATH', '')
    run = subprocess.run([str(binary), '--gtest_filter=NestedModel.IncludeElements',
                          '--gtest_output=xml:' + str(xml)], check=True, env=env, capture_output=True, text=True)
    (output.parent / 'version15-original.log').write_text(run.stdout + run.stderr)
    result = ET.parse(xml).getroot()
    assert result.attrib['tests'] == '1' and result.attrib['failures'] == '0'
    binding = next((upstream / 'build/lib').glob('sdformat.cpython-*.so'))
    linked = subprocess.check_output(['ldd', str(binding)], env=env, text=True)
    library = next(Path(line.split('=>', 1)[1].split()[0])
                   for line in linked.splitlines() if line.strip().startswith('libsdformat.so.'))
    gap_corpus = root / 'tests/version15_unsupported.json'
    gap_probe = """
import json,sys
from pathlib import Path
import sdformat
rows=[]
for case in json.loads(Path(sys.argv[1]).read_text()):
    document=sdformat.Root()
    document.load_sdf_string(case['xml'])
    row=dict(id=case['id'],accepted=True)
    if 'gravity' in case:
        row['gravity']=list(document.world_by_index(0).gravity())
        assert row['gravity']==case['gravity']
    else:
        imu=document.model().link_by_index(0).sensor_by_index(0).imu_sensor()
        row['noise_means']=[getattr(imu,group+'_'+axis+'_noise')().mean() for group in ('angular_velocity','linear_acceleration') for axis in ('x','y','z')]
        assert row['noise_means']==case['noise_means']
    rows.append(row)
print(json.dumps(rows))
"""
    env['PYTHONPATH'] = str(upstream / 'build/lib') + ':' + str(Path.home() / 'local/lib/python')
    gaps = json.loads(subprocess.check_output(['/usr/bin/python3', '-c', gap_probe, str(gap_corpus)], env=env, text=True))
    proof = dict(upstream_sha=revision, source_sha256={p: sha(upstream / p) for p in paths},
                 corpus_sha256=corpus['corpus_sha256'], observations=observations,
                 source_fixture_sha256=sha(root / 'tests/fixtures/include_elements.sdf'),
                 resources_sha256={local: sha(root / local) for local in fixtures},
                 original_binary_sha256=sha(binary), original_tests=1, original_failures=0,
                 binding_sha256=sha(binding), native_library_sha256=sha(library),
                 probe_script_sha256=sha(root / 'tools/reference_probe.py'),
                 explicit_unsupported_sha256=sha(gap_corpus), native_supported_rust_unsupported=gaps)
    output.write_text(json.dumps(proof, indent=2) + '\n')
    print('62 native conversion-chain controls and exact original IncludeElements passed')


if __name__ == '__main__':
    main()
