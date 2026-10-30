#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay valid material metadata and actual XML exchanges."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    output = root / 'artifacts/material-xml-reference'
    output.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    reference = json.loads((root / 'reference/material-xml.json').read_text())
    assert subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip() == reference['upstream_sha']
    for path, expected in reference['native_sources'].items():
        assert sha(upstream / path) == expected, path
    include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    assert sha(include / 'sdf/Material.hh') == sha(upstream / 'include/sdf/Material.hh')
    math = upstream.parent / 'gz-math'
    assert subprocess.check_output(['git', '-C', str(math), 'rev-parse', 'HEAD'], text=True).strip() == reference['math_upstream_sha']
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], text=True).strip())
    for path, expected in reference['math_sources'].items():
        assert sha(math / path) == expected, path
        assert sha(math_include / path.removeprefix('include/')) == expected, path
    libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()).resolve()
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    binary = output / 'material-xml-reference'
    subprocess.run(['c++', '-std=c++17', str(root / 'tests/material_xml_reference.cc'), *flags,
                    '-Wl,--disable-new-dtags', f'-Wl,-rpath,{libdir}', '-o', str(binary)], check=True)
    libraries = {}
    for line in subprocess.check_output(['ldd', str(binary)], text=True).splitlines():
        words = line.split()
        if '=>' in words and len(words) > 2 and Path(words[2]).is_file():
            path = Path(words[2]).resolve(); libraries[path.name] = sha(path)
    assert libraries == reference['libraries']
    def query(xml='', arguments=()):
        value = subprocess.run([str(binary), *arguments], input=xml, capture_output=True,
                               text=True, check=True, timeout=10)
        return dict(stdout=value.stdout, stderr=value.stderr)
    data=json.loads((root/'tests/material_xml.json').read_text())
    for mode,name in [('--fixture-metal','material_metal_native.xml'),('--fixture-specular','material_specular_native.xml'),('--default','material_default_native.xml')]:
        value=query(arguments=[mode]);assert value==data['constructors'][mode]
        assert not value['stderr'];assert '\nEMIT 1\nXML_BEGIN\n' in value['stdout']
        fixture=root/'tests/fixtures'/name
        xml=value['stdout'].split('XML_BEGIN\n',1)[1].split('XML_END\n',1)[0]
        assert xml.encode()==fixture.read_bytes()
        assert sha(fixture)==reference['fixtures'][str(fixture.relative_to(root))]
    for row in data['rows']:assert query(row['xml'])==row['native'],row['id']
    exports=output/'exports';exports.mkdir(exist_ok=True)
    for path in exports.glob('*.xml'):path.unlink()
    subprocess.run(['cargo','test','--offline','--locked','--test','material_xml'],cwd=root,env=dict(os.environ,SDF_MATERIAL_XML_EXPORT=str(exports)),check=True)
    expected={row['id']+'.xml' for row in data['rows'] if row['export']}
    assert {p.name for p in exports.glob('*.xml')}==expected
    for row in data['rows']:
        if row['export']:
            emitted=query((exports/(row['id']+'.xml')).read_text())
            assert emitted['stdout']==row['native']['stdout'],row['id']
            assert not emitted['stderr'],(row['id'],emitted['stderr'])
    env=dict(os.environ,LD_LIBRARY_PATH=str(upstream/'build/lib')+':'+str(libdir))
    for case in ['DOMMaterial.ToElement','DOMMaterial.ToElementErrorOutput']:
        value=subprocess.run([str(upstream/'build/bin/UNIT_Material_TEST'),'--gtest_filter='+case],env=env,capture_output=True,text=True,check=True,timeout=30)
        assert '[  PASSED  ] 1 test.' in value.stdout,value.stdout
        assert not value.stderr,value.stderr
    result=dict(native_cases=len(data['rows']),exact_native_fixtures=3,actual_rust_to_native_exchanges=len(expected),unchanged_native_originals=2,constructor_domain_rust_original_equivalence=False,libraries=libraries)
    (output/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(result)

if __name__=='__main__':main()
