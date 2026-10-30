#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay valid heightmap metadata and actual XML exchanges."""
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
    output = root / 'artifacts/heightmap-xml-reference'
    output.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    reference = json.loads((root / 'reference/heightmap-xml.json').read_text())
    assert subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip() == reference['upstream_sha']
    for path, expected in reference['native_sources'].items():
        assert sha(upstream / path) == expected, path
    include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    assert sha(include / 'sdf/Heightmap.hh') == sha(upstream / 'include/sdf/Heightmap.hh')
    math = upstream.parent / 'gz-math'
    assert subprocess.check_output(['git', '-C', str(math), 'rev-parse', 'HEAD'], text=True).strip() == reference['math_upstream_sha']
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], text=True).strip())
    for path, expected in reference['math_sources'].items():
        assert sha(math / path) == expected, path
        assert sha(math_include / path.removeprefix('include/')) == expected, path
    libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()).resolve()
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    binary = output / 'heightmap-xml-reference'
    subprocess.run(['c++', '-std=c++17', str(root / 'tests/heightmap_xml_reference.cc'), *flags,
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
    data = json.loads((root / 'tests/heightmap_xml.json').read_text())
    for mode, name in [('--fixture','heightmap_to_element_native.xml')]:
        value=query(arguments=[mode]); assert not value['stderr']
        assert value['stdout'].startswith('EMIT 1\nXML_BEGIN\n')
        xml=value['stdout'].split('XML_BEGIN\n',1)[1].split('XML_END\n',1)[0]
        fixture=root/'tests/fixtures'/name
        assert xml.encode()==fixture.read_bytes()
        assert sha(fixture)==reference['fixtures'][str(fixture.relative_to(root))]
    assert query(arguments=['--empty'])==data['empty_native_producer']
    for row in data['rows']: assert query(row['xml'])==row['native'],row['id']
    for row in data['geometry_controls']:
        mode='--document' if row['xml'].startswith('<sdf') else '--geometry'
        assert query(row['xml'],[mode])==row['native'],row['id']
    exports=output/'exports'; exports.mkdir(exist_ok=True)
    for path in exports.glob('*.xml'): path.unlink()
    env=dict(os.environ,SDF_HEIGHTMAP_XML_EXPORT=str(exports))
    subprocess.run(['cargo','test','--offline','--locked','--test','heightmap_xml','--test','heightmap_xml_original','--test','geometry_xml'],cwd=root,env=env,check=True)
    expected={row['id']+'.xml' for row in data['rows'] if row['export']}
    assert {p.name for p in exports.glob('*.xml')}==expected
    for row in data['rows']:
        if row['export']: assert query((exports/(row['id']+'.xml')).read_text())==row['native'],row['id']
    env['LD_LIBRARY_PATH']=str(upstream/'build/lib')+':'+str(libdir)
    for case in ['DOMHeightmap.ToElement','DOMHeightmap.ToElementErrorOutput']:
        value=subprocess.run([str(upstream/'build/bin/UNIT_Heightmap_TEST'),'--gtest_filter='+case],env=env,capture_output=True,text=True,check=True,timeout=30)
        assert '[  PASSED  ] 1 test.' in value.stdout,value.stdout
        assert not value.stderr,value.stderr
    result=dict(native_cases=len(data['rows']),exact_native_fixtures=1,geometry_controls=2,actual_rust_to_native_exchanges=len(expected),unchanged_originals=2,libraries=libraries)
    (output/'result.json').write_text(json.dumps(result,indent=2)+'\n')
    print(result)

if __name__ == '__main__': main()
