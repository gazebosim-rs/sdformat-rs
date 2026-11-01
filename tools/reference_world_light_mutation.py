#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Qualify owned World Light mutation against the pinned native implementation."""
import argparse
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import xml.etree.ElementTree as ET

from verify import isolated_git_context
from world_scene_reference import PIN, digest, linkage

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--capture', action='store_true')
    args = parser.parse_args()
    upstream = args.upstream.resolve()
    with isolated_git_context(ROOT):
        assert subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip() == PIN
        subprocess.run(['git', '-C', str(upstream), 'diff', '--quiet', PIN, '--'], check=True)
    env = dict(os.environ)
    for key in ['LD_PRELOAD', 'LD_AUDIT', 'LD_LIBRARY_PATH', 'SDF_PATH']:
        env.pop(key, None)
    def pkg(name):
        return Path(subprocess.check_output(['pkg-config', '--variable=' + name, 'sdformat'], env=env, text=True).strip()).resolve()
    libdir, includedir = pkg('libdir'), pkg('includedir')
    env['LD_LIBRARY_PATH'] = str(libdir) + ':/opt/ros/jazzy/lib/x86_64-linux-gnu'
    library = (libdir / 'libsdformat.so').resolve()
    assert digest(library) == digest(upstream / 'build/lib' / library.name)
    headers = [includedir / 'sdf' / (name + '.hh') for name in ['World', 'Light']]
    for header in headers:
        assert digest(header) == digest(upstream / 'include/sdf' / header.name)
    sources = ['src/World.cc', 'src/Light.cc', 'src/World_TEST.cc', 'include/sdf/World.hh', 'include/sdf/Light.hh']
    helper = ROOT / 'tests/world_light_mutation_reference.cc'
    compiler = Path(shutil.which('c++')).resolve()
    original = upstream / 'build/bin/UNIT_World_TEST'
    protected = [Path(__file__), helper, compiler, library, original, *headers, *[upstream / p for p in sources]]
    before = {str(p): digest(p) for p in protected}
    output = ROOT / 'artifacts/world-light-mutation-native'
    output.mkdir(parents=True, exist_ok=True)
    assert shutil.disk_usage(output).free > 1280 * 1024 ** 2
    binary = output / 'reference'
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], env=env, text=True))
    command = [str(compiler), '-std=c++17', '-Wall', '-Wextra', '-Werror', str(helper), *flags,
               '-Wl,--disable-new-dtags', '-Wl,-rpath,' + str(libdir), '-o', str(binary)]
    subprocess.run(command, env=env, check=True)
    libraries = linkage(binary, env)
    assert libraries == json.loads((ROOT / 'reference/document-lights.json').read_text())['libraries']
    for executable in [binary, original]:
        for line in subprocess.check_output(['ldd', str(executable)], env=env, text=True).splitlines():
            words = line.split()
            if '=>' in words and len(words) > 2 and words[2].startswith('/'):
                path = Path(words[2]).resolve()
                before[str(path)] = digest(path)
    run = subprocess.run([str(binary)], env=env, capture_output=True, timeout=10)
    (output / 'helper.stdout').write_bytes(run.stdout)
    (output / 'helper.stderr').write_bytes(run.stderr)
    assert run.returncode == 0 and run.stderr == b''
    report = output / 'original.xml'
    original_run = subprocess.run([str(original), '--gtest_filter=DOMWorld.AddLight', '--gtest_output=xml:' + str(report)], env=env, capture_output=True, timeout=30)
    (output / 'original.stdout').write_bytes(original_run.stdout)
    (output / 'original.stderr').write_bytes(original_run.stderr)
    assert original_run.returncode == 0
    xml = ET.parse(report).getroot()
    assert xml.attrib['tests'] == '1' and xml.attrib['failures'] == '0'
    assert xml.attrib.get('disabled', '0') == xml.attrib.get('errors', '0') == '0'
    assert not list(xml.iter('skipped'))
    assert {r.attrib['classname'] + '.' + r.attrib['name'] for r in xml.iter('testcase')} == {'DOMWorld.AddLight'}
    provenance = {'upstream_sha': PIN, 'native_sources': {p: digest(upstream / p) for p in sources},
                  'installed_headers': {p.name: digest(p) for p in headers}, 'compiler_sha256': digest(compiler),
                  'compiler_version': subprocess.check_output([str(compiler), '--version'], env=env, text=True),
                  'helper_sha256': digest(helper), 'script_sha256': digest(Path(__file__)),
                  'libraries': libraries, 'original_libraries': linkage(original, env),
                  'original_binary_sha256': digest(original), 'helper_stdout': run.stdout.decode()}
    assert before == {name: digest(Path(name)) for name in before}
    reference = ROOT / 'reference/world-light-mutation.json'
    if args.capture:
        reference.write_text(json.dumps(provenance, indent=2) + '\n')
    else:
        assert provenance == json.loads(reference.read_text())
    (output / 'result.json').write_text(json.dumps({'command': command, 'helper_exit': run.returncode,
        'unchanged_original_tests': 1, 'provenance': provenance}, indent=2) + '\n')
    print('Native World Light collection controls and unchanged DOMWorld.AddLight passed')


if __name__ == '__main__':
    main()
