#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Reproduce conservative known SDF schema and conversion names."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import xml.etree.ElementTree as ET

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('upstream', type=Path)
parser.add_argument('--write', action='store_true')
args = parser.parse_args()
upstream = args.upstream.resolve()
revision = json.loads((root / 'reference/inventory.json').read_text())['upstream_sha']
assert subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip() == revision
versions = ('1.4', '1.5', '1.6', '1.7', '1.8', '1.9', '1.10', '1.11', '1.12')
paths = []
for version in versions:
    entries = subprocess.check_output(['git', '-C', str(upstream), 'ls-tree', '-r', '--name-only', revision, '--', 'sdf/' + version], text=True).splitlines()
    paths.extend(name for name in entries if name.endswith(('.sdf', '.convert')))
assert len(paths) == 547 and len(set(paths)) == len(paths)
names = set()
inputs = {}
for name in sorted(paths):
    original = subprocess.check_output(['git', '-C', str(upstream), 'show', revision + ':' + name])
    assert (upstream / name).read_bytes() == original, name
    inputs[name] = hashlib.sha256(original).hexdigest()
    xml = ET.fromstring(original)
    for element in xml.iter():
        if name.endswith('.sdf'):
            values = [element.attrib['name']] if element.tag in ('element', 'attribute') and 'name' in element.attrib else []
        else:
            values = [element.attrib[key] for key in ('name', 'element', 'attribute', 'descendant_name') if key in element.attrib]
        for value in values:
            for part in re.split(r'::|/', value):
                if part and not part.startswith('@'):
                    assert re.fullmatch(r'[A-Za-z_][A-Za-z0-9_.-]*', part), (name, part)
                    assert part == part.lower(), (name, part)
                    names.add(part)
asset = root / 'data/world_known_schema_names.txt'
content = '# Copyright (C) 2026 Open Source Robotics Foundation\n# SPDX-License-Identifier: Apache-2.0\n# Generated from pinned SDFormat schemas and conversion tables.\n' + ''.join(name + '\n' for name in sorted(names))
metadata = {'upstream_sha': revision, 'input_sha256': inputs, 'name_count': len(names), 'names_sha256': hashlib.sha256(content.encode()).hexdigest(), 'generator_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
reference = root / 'reference/world-known-schema-inventory.json'
if args.write:
    asset.write_text(content)
    reference.write_text(json.dumps(metadata, indent=2) + '\n')
else:
    assert asset.read_text() == content
    assert json.loads(reference.read_text()) == metadata
print(f'{len(paths)} pinned schemas/conversions and {len(names)} conservative names verified')
