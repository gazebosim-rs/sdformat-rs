# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay complete native custom-sensor observations without rewriting XML prefixes."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--write', action='store_true')
    args = parser.parse_args()
    source = args.upstream.resolve()
    assert subprocess.check_output(['git', '-C', str(source), 'rev-parse', 'HEAD'], text=True).strip() == '1249805d9fceeff94ca1fe63e26ab9c051cea24e'
    fixture = ROOT / 'tests/fixtures/environmental_sensor.sdf.in'
    cases = [('original_world', fixture.read_text())]
    fragments = [
        ('missing', "<sensor name='s' type='custom'/>"),
        ('empty', "<sensor name='s' type='custom' gz:type=''/>") ,
        ('arbitrary', "<sensor name='s' type='custom' gz:type='arbitrary/type'><custom_field a='one'><nested>value&amp;text</nested><!--note--><![CDATA[tail]]></custom_field><custom_field/><another:field/></sensor>"),
        ('no_subtype_fields', "<sensor name='s' type='custom'><custom_field/><custom_field/></sensor>"),
        ('decoded', "<sensor name='s' type='custom' gz:type='a&amp;b'><field>text</field></sensor>"),
        ('space', "<sensor name='s' type='custom' gz:type='  a  b  '/>"),
        ('whitespace_references', "<sensor name='s' type='custom' gz:type='&#9;a&#13;b&#10;'/>") ,
        ('known_payload_strict_difference', "<sensor name='s' type='custom'><imu/></sensor>"),
        ('duplicate_attribute', "<sensor name='s' type='custom' gz:type='a' gz:type='b'/>") ,
        ('malformed_child', "<sensor name='s' type='custom'><field></other></sensor>"),
    ]
    cases.extend((name, '<sdf version="1.6"><world name="w"><model name="model_with_sensor"><link name="link">' + fragment + '</link></model></world></sdf>') for name, fragment in fragments)
    libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip())
    includedir = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
    for header in ['Sensor.hh', 'Element.hh', 'Root.hh']:
        assert digest(includedir / 'sdf' / header) == digest(source / 'include/sdf' / header)
    selected_library = (libdir / 'libsdformat.so').resolve()
    assert digest(selected_library) == digest(source / 'build/lib' / selected_library.name)
    env = dict(os.environ, LD_LIBRARY_PATH=str(libdir))
    with tempfile.TemporaryDirectory(prefix='sdf-custom-native-') as temporary:
        temporary = Path(temporary)
        binary = temporary / 'native'
        flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
        subprocess.run(['c++', '-std=c++17', str(ROOT / 'tests/custom_sensor_reference.cc'), '-o', str(binary), *flags, '-Wl,-rpath,' + str(libdir)], check=True)
        linked = subprocess.check_output(['ldd', str(binary)], env=env, text=True)
        libraries = {}
        for line in linked.splitlines():
            words = line.split()
            if '=>' in words and len(words) > 2 and words[2].startswith('/'):
                path = Path(words[2]).resolve()
                assert path.name not in libraries
                libraries[path.name] = digest(path)
        assert any('libsdformat' in path for path in libraries)
        observations = []
        for name, xml in cases:
            path = temporary / (name + '.sdf')
            path.write_text(xml)
            result = subprocess.run([str(binary), str(path)], env=env, capture_output=True)
            observations.append({'name': name, 'xml': xml, 'exit': result.returncode, 'stdout': result.stdout.decode(), 'stderr': result.stderr.decode()})
        record = {
            'upstream_sha': '1249805d9fceeff94ca1fe63e26ab9c051cea24e',
            'sim_fixture_revision': '71c60db31fddd5c98d2aa689e9a2cccb3548a0a3',
            'fixture_sha256': digest(fixture),
            'helper_sha256': digest(ROOT / 'tests/custom_sensor_reference.cc'),
            'native_sources': {name: digest(source / name) for name in ['src/Sensor.cc', 'src/Element.cc', 'src/Param.cc', 'src/parser.cc', 'sdf/1.12/sensor.sdf']},
            'installed_headers': {name: digest(includedir / 'sdf' / name) for name in ['Sensor.hh', 'Element.hh', 'Root.hh']},
            'native_libraries': libraries,
            'observations': observations,
        }
    path = ROOT / 'tests/custom_sensor_native.json'
    if args.write:
        path.write_text(json.dumps(record, indent=2) + '\n')
    else:
        assert record == json.loads(path.read_text()), 'native observations or provenance changed'
    print(f'{len(observations)} complete native observations verified; custom family known-payload strict difference retained')

if __name__ == '__main__':
    main()
