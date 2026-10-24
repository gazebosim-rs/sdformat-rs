#!/usr/bin/env python3
"""Freeze source hashes and enumerate the built reference, without changing it."""
import argparse
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
                    entry['cases'].append({'id': case, 'disabled': 'DISABLED_' in case, 'baseline': 'unrun', 'replacement': []})
        elif '-m' in command and 'pytest' in command:
            probe = capture(command[:3] + ['--collect-only', '-q', command[3]], env=env)
            entry['cases'] = [{'id': line, 'baseline': 'unrun', 'replacement': []} for line in probe.stdout.splitlines() if '::test_' in line]
        else:
            entry['enumeration'] = 'ctest target only; no supported runtime enumerator'
            entries.append(entry)
            continue
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
    static = sum(len(re.findall(r'\bTEST(?:_F|_P)?\s*\(', path.read_text(errors='replace'))) for path in source.rglob('*.cc') if 'build' not in path.parts)
    output = {'upstream_sha': revision, 'dirty': capture(['git', '-C', str(source), 'status', '--porcelain']).stdout, 'static_test_macros': static, 'targets': entries, 'sha256': files}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, indent=2) + '\n')
    print(json.dumps({'sha': revision, 'targets': len(entries), 'expanded_cases': sum(len(e['cases']) for e in entries), 'enumeration_failures': sum(e.get('enumeration_exit', 0) != 0 for e in entries), 'static_test_macros': static, 'files': len(files)}))


if __name__ == '__main__':
    main()
