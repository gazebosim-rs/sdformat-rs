#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay native-valid Empty metadata, strict differences and actual exchanges."""
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
    output = root / 'artifacts/geometry-empty-reference'
    output.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    reference = json.loads((root / 'reference/geometry-empty.json').read_text())
    assert subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip() == reference['upstream_sha']
    for path, expected in reference['native_sources'].items():
        assert sha(upstream / path) == expected, path
    include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    assert sha(include / 'sdf/Geometry.hh') == sha(upstream / 'include/sdf/Geometry.hh')
    math = upstream.parent / 'gz-math'
    assert subprocess.check_output(['git', '-C', str(math), 'rev-parse', 'HEAD'], text=True).strip() == reference['math_upstream_sha']
    math_include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-math'], text=True).strip())
    for path, expected in reference['math_sources'].items():
        assert sha(math / path) == expected, path
        assert sha(math_include / path.removeprefix('include/')) == expected, path
    libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()).resolve()
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
    binary = output / 'geometry-empty-reference'
    subprocess.run(['c++', '-std=c++17', str(root / 'tests/geometry_empty_reference.cc'), *flags,
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
    data=json.loads((root/'tests/geometry_empty.json').read_text())
    value=query(arguments=['constructor']); assert value==data['constructor']
    xml=value['stdout'].split('XML_BEGIN\n',1)[1].split('XML_END\n',1)[0]
    fixture=root/'tests/fixtures/geometry_empty_native.xml';assert fixture.read_bytes()==xml.encode()
    assert sha(fixture)==reference['fixtures'][str(fixture.relative_to(root))]
    for row in data['rows']:assert query(row['xml'],[row['mode']])==dict(stdout=row['stdout'],stderr=row['stderr']),row['id']
    exports=output/'exports';exports.mkdir(exist_ok=True)
    for path in exports.iterdir():
        if path.suffix in ['.xml','.mode']:path.unlink()
    env=dict(os.environ,SDF_EMPTY_GEOMETRY_EXPORT=str(exports))
    subprocess.run(['cargo','test','--offline','--locked','--test','geometry_empty','--test','geometry_xml','--test','visual_xml','--test','semantics'],cwd=root,env=env,check=True)
    expected={r['id']+'.xml' for r in data['rows'] if r['export']};assert {p.name for p in exports.glob('*.xml')}==expected
    for row in data['rows']:
        if not row['export']:continue
        name=row['id'];mode=(exports/(name+'.mode')).read_text();assert mode==('visual' if row['mode']=='visual' else 'geometry')
        assert query((exports/(name+'.xml')).read_text(),[mode])==dict(stdout=row['stdout'],stderr=row['stderr']),name
    result=dict(native_cases=len(data['rows']),exact_native_fixtures=1,actual_geometry_or_visual_exchanges=len(expected),strict_native_differences=sum(not r['export'] for r in data['rows']),libraries=libraries)
    (output/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(result)

if __name__=='__main__':main()
