#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0

"""Freeze source hashes and enumerate the built reference, without changing it."""
import argparse
import ast
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess


def capture(command, **kwargs):
    return subprocess.run(command, text=True, capture_output=True, **kwargs)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    parser.add_argument('--output', type=Path, default=Path('reference/inventory.json'))
    args = parser.parse_args()
    source = args.upstream.resolve()
    revision = capture(['git', '-C', str(source), 'rev-parse', 'HEAD']).stdout.strip()
    result = capture(['ctest', '--test-dir', str(source / 'build'), '--show-only=json-v1'])
    result.check_returncode()
    inventory = json.loads(result.stdout)
    env = dict(os.environ)
    env['LD_LIBRARY_PATH'] = ':'.join([str(source / 'build/lib'), str(Path.home() / 'local/lib'), env.get('LD_LIBRARY_PATH', '')])
    env['PYTHONPATH'] = ':'.join([str(source / 'build/lib'), str(Path.home() / 'local/lib/python'), env.get('PYTHONPATH', '')])
    env['PYTHONDONTWRITEBYTECODE'] = '1'
    literal_sources = {}
    vendor_mentions = 0
    pattern = r'\b(TEST(?:_F|_P)?)\s*\(\s*(\w+)\s*,\s*(\w+)\s*\)'
    for path in sorted(source.rglob('*.cc')):
        if 'build' in path.parts:
            continue
        text = path.read_text(errors='replace')
        if 'gtest_vendor' in path.parts:
            vendor_mentions += len(re.findall(r'\bTEST(?:_F|_P)?\s*\(', text))
            continue
        text = re.sub(r'GZ_UTILS_TEST_DISABLED_ON_WIN32\((\w+)\)', r'\1', text)
        for match in re.finditer(pattern, text):
            symbol = match[2] + '.' + match[3]
            literal_sources.setdefault(symbol, []).append({'path': str(path.relative_to(source)), 'line': text.count('\n', 0, match.start()) + 1, 'symbol': symbol})
    previous = {}
    if args.output.exists():
        for target in json.loads(args.output.read_text())['targets']:
            previous.update({(target['target'], case['id']): case for case in target['cases']})
    entries = []
    for target in inventory['tests']:
        command = target.get('command', [])
        entry = {'target': target['name'], 'command': command, 'properties': target['properties'], 'cases': [], 'migration': 'pending'}
        if command and '/bin/' in command[0] and Path(command[0]).name.startswith(('UNIT_', 'INTEGRATION_', 'PERFORMANCE_')):
            probe = capture([command[0], '--gtest_list_tests'], env=env)
            suite = ''
            for line in probe.stdout.splitlines():
                clean = line.split('#')[0].rstrip()
                if clean and not clean.startswith(' ') and clean.endswith('.'):
                    suite = clean
                elif suite and clean.startswith('  '):
                    case = suite + clean.strip()
                    sources = literal_sources.get(case, [])
                    expected_stem = Path(command[0]).name.split('_', 1)[1]
                    specific = [item for item in sources if Path(item['path']).stem == expected_stem]
                    sources = specific or sources
                    entry['cases'].append({'id': case, 'disabled': 'DISABLED_' in case, 'source': sources, 'baseline': 'unrun', 'replacement': [], 'assertion_contract': 'pending', 'fixture_contract': 'pending manual extraction'})
        elif '-m' in command and 'pytest' in command:
            probe = capture(command[:3] + ['--collect-only', '-q', '-p', 'no:cacheprovider', command[3]], env=env)
            py_path = Path(command[3])
            syntax = ast.parse(py_path.read_text())
            methods = {cls.name + '.' + method.name: method.lineno for cls in syntax.body if isinstance(cls, ast.ClassDef) for method in cls.body if isinstance(method, ast.FunctionDef)}
            entry['cases'] = []
            for line in probe.stdout.splitlines():
                if '::test_' in line:
                    symbol = '.'.join(line.split('::')[1:])
                    entry['cases'].append({'id': line, 'source': [{'path': str(py_path.relative_to(source)), 'line': methods[symbol], 'symbol': symbol}], 'baseline': 'unrun', 'replacement': [], 'assertion_contract': 'pending', 'fixture_contract': 'pending manual extraction'})
        else:
            entry['enumeration'] = 'ctest target only; no supported runtime enumerator'
            entries.append(entry)
            continue
        for case in entry['cases']:
            old = previous.get((entry['target'], case['id']), {})
            for key in ('baseline', 'replacement'):
                if key in old: case[key] = old[key]
        entry['enumeration_exit'] = probe.returncode
        entry['enumeration_stderr'] = probe.stderr
        if not entry['cases']:
            entry['enumeration_stdout'] = probe.stdout
        entries.append(entry)
    files = {}
    for root in ('src', 'include', 'test', 'python', 'sdf'):
        for path in sorted((source / root).rglob('*')):
            if path.is_file() and path.suffix in ('.cc', '.hh', '.sdf', '.config', '.py', '.convert'):
                files[str(path.relative_to(source))] = hashlib.sha256(path.read_bytes()).hexdigest()
    static = sum(len(items) for items in literal_sources.values())
    output = {'upstream_sha': revision, 'dirty': capture(['git', '-C', str(source), 'status', '--porcelain']).stdout, 'static_test_macros': static, 'excluded_third_party_macro_mentions': vendor_mentions, 'runtime_source_mapping_missing': sum(not case['source'] for target in entries for case in target['cases']), 'targets': entries, 'sha256': files}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, indent=2) + '\n')
    print(json.dumps({'sha': revision, 'targets': len(entries), 'expanded_cases': sum(len(e['cases']) for e in entries), 'enumeration_failures': sum(e.get('enumeration_exit', 0) != 0 for e in entries), 'static_test_macros': static, 'files': len(files)}))


if __name__ == '__main__':
    main()
