#!/usr/bin/env python3
"""Single local/CI gate; coverage uses the pinned toolchain's LLVM binaries."""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def run(command, *, env=None, capture=False, cwd=ROOT):
    print('+', ' '.join(map(str, command)), flush=True)
    return subprocess.run(command, cwd=cwd, env=env, check=True, text=True, capture_output=capture)


def digest(root):
    files = sorted(p for pattern in ('src/**/*.rs', 'tests/**/*', 'tools/*.py') for p in root.glob(pattern) if p.is_file())
    files += [root / p for p in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'coverage.json', 'reference/traceability.json', 'reference/inventory.json', 'reference/results.json')]
    state = hashlib.sha256()
    for path in files:
        state.update(str(path.relative_to(root)).encode())
        state.update(path.read_bytes())
    return state.hexdigest()


def review_digest(root):
    protected = sorted(p for pattern in ('tests/**/*', 'tools/*', '.githooks/*', '.github/workflows/*', '.pre-commit-config.yaml', 'coverage.json', 'AGENTS.md', 'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'reference/*.json') for p in root.glob(pattern) if p.is_file() and p.name != 'review.json')
    state = hashlib.sha256()
    for path in protected:
        state.update(str(path.relative_to(root)).encode())
        state.update(path.read_bytes())
    return state.hexdigest()


def check_review(root):
    receipt = json.loads((root / 'reference/review.json').read_text())
    assert receipt['decision'] == 'approved' and receipt['reviewer'] != receipt['author'], 'independent policy/assertion review required'
    assert receipt['protected_digest'] == review_digest(root), 'tests, assertion mappings, or policy changed: independent review required'


def check_counts(summary):
    for metric in ('lines', 'regions'):
        values = summary[metric]
        assert values['count'] > 0, f'{metric}: empty denominator'
        assert values['covered'] * 100 > values['count'] * 95, f'{metric}: {values["covered"]}/{values["count"]} must exceed 95%'


def check_report(report, root, manifest):
    assert report.get('type') == 'llvm.coverage.json.export', 'missing instrumentation'
    files = {}
    for item in report['data']:
        for file in item['files']:
            path = Path(file['filename']).resolve()
            if path.is_relative_to(root / 'src'):
                assert path not in files, f'duplicate coverage file {path}'
                files[path] = file
    expected = {p.resolve() for p in (root / 'src').rglob('*.rs')}
    assert expected and expected == set(files), f'missing or unexpected production files: {expected ^ set(files)}'
    assert manifest['profiles'] == ['core'], 'profile matrix changed without gate support'
    assert manifest['crates'] == {'sdformat': ['src']}, 'crate matrix changed without gate support'
    assert manifest['exclusions'] == [], 'coverage exclusions require review and implementation'
    total = {metric: {'count': 0, 'covered': 0} for metric in ('lines', 'regions')}
    for file in files.values():
        for metric in total:
            for key in total[metric]:
                total[metric][key] += file['summary'][metric][key]
    check_counts(total)  # One production crate: per-crate and repo are identical.
    return total


def check_ledger(root):
    inventory = json.loads((root / 'reference/inventory.json').read_text())
    trace = json.loads((root / 'reference/traceability.json').read_text())
    assert inventory['upstream_sha'] == trace['upstream_sha']
    ids = {(target['target'], case['id']) for target in inventory['targets'] for case in target['cases']}
    assert len(ids) == sum(len(target['cases']) for target in inventory['targets']), 'duplicate runtime test IDs'
    assert inventory['runtime_source_mapping_missing'] == 0, 'runtime cases missing source mappings'
    for target in inventory['targets']:
        for case in target['cases']:
            assert case['source'] and all(item['path'] in inventory['sha256'] for item in case['source']), 'unhashed runtime test source'
    seen = set()
    for mapping in trace['ports'] + trace.get('partial_ports', []):
        key = (mapping['target'], mapping['case'])
        assert key in ids and key not in seen, f'invalid or duplicate mapping {key}'
        seen.add(key)
        assert mapping['assertions'] and mapping['rust_test'] and mapping['review'], 'missing assertion/review record'
        if mapping['review'] == 'pending':
            assert mapping['status'] in ('ported_pending_review', 'partial_pending_review'), 'unreviewed ports cannot claim acceptance'
        else:
            assert mapping['status'] in ('ported_reviewed', 'partial_reviewed'), 'invalid reviewed port state'
        for name, sha in mapping['upstream_fixtures'].items():
            assert inventory['sha256'][name] == sha, f'upstream fixture hash mismatch {name}'
        for name, sha in mapping['fixtures'].items():
            assert hashlib.sha256((root / name).read_bytes()).hexdigest() == sha, f'fixture changed {name}'
    # Assertions/mappings/coverage policy changes are reviewed, not auto-approved.
    if (root / '.git').exists():
        changed = run(['git', 'diff', '--name-only', 'HEAD', '--', 'tests', 'reference', 'coverage.json', 'tools'], capture=True, cwd=root).stdout.splitlines()
        if changed:
            print('Protected changes:', ', '.join(changed))


def coverage(root, out, manifest):
    out.mkdir(parents=True, exist_ok=True)
    raw = out / 'raw'
    if raw.exists(): shutil.rmtree(raw)
    raw.mkdir()
    env = dict(os.environ, CARGO_TARGET_DIR=str(out / 'target'), LLVM_PROFILE_FILE=str(raw / '%p-%m.profraw'))
    env['RUSTFLAGS'] = '-C instrument-coverage'
    result = run(['cargo', 'test', '--locked', '--all-targets', '--message-format=json'], env=env, capture=True, cwd=root)
    (out / 'tests.log').write_text(result.stdout + result.stderr)
    binaries = []
    for line in result.stdout.splitlines():
        if line.startswith('{'):
            item = json.loads(line)
            if item.get('reason') == 'compiler-artifact' and item.get('executable') and item['profile']['test']:
                binaries.append(item['executable'])
    assert binaries, 'no instrumented test binaries'
    sysroot = Path(run(['rustc', '--print', 'sysroot'], capture=True, cwd=root).stdout.strip())
    host = next(line.split(': ', 1)[1] for line in run(['rustc', '-vV'], capture=True, cwd=root).stdout.splitlines() if line.startswith('host: '))
    llvm = sysroot / 'lib/rustlib' / host / 'bin'
    profiles = sorted(raw.glob('*.profraw'))
    assert profiles, 'no runtime profiles'
    merged = out / 'merged.profdata'
    run([str(llvm / 'llvm-profdata'), 'merge', '-sparse', *map(str, profiles), '-o', str(merged)], cwd=root)
    args = [str(llvm / 'llvm-cov'), 'export', f'-instr-profile={merged}']
    for binary in binaries: args += ['-object', binary]
    report = json.loads(run(args, capture=True, cwd=root).stdout)
    (out / 'coverage.json').write_text(json.dumps(report) + '\n')
    counts = check_report(report, root, manifest)
    lcov = run(args + ['-format=lcov'], capture=True, cwd=root).stdout
    (out / 'coverage.lcov').write_text(lcov)
    # LLVM's DA records are the authoritative executable-line denominator.
    covered = {}
    path = None
    for line in lcov.splitlines():
        if line.startswith('SF:'): path = Path(line[3:]).resolve()
        elif line.startswith('DA:') and path:
            number, count, *_ = line[3:].split(',')
            covered[(path, int(number))] = int(count) > 0
    changed = changed_lines(root)
    measured = [covered[key] for key in changed if key in covered]
    if measured:
        assert sum(measured) * 100 > len(measured) * 95, 'changed executable line coverage must exceed 95%'
    evidence = {'source_digest': digest(root), 'toolchain': run(['rustc', '-vV'], capture=True, cwd=root).stdout, 'runner': os.uname().nodename, 'profile': 'core', 'coverage': counts, 'changed_lines': {'count': len(measured), 'covered': sum(measured)}}
    (out / 'evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(json.dumps(evidence, indent=2))


def changed_lines(root):
    if not (root / '.git').exists():
        # In an exported staged tree every line must satisfy the gate. This is
        # stricter than borrowing history/evidence from a different working tree.
        return {(p.resolve(), i) for p in (root / 'src').rglob('*.rs') for i, _ in enumerate(p.read_text().splitlines(), 1)}
    diff = run(['git', 'diff', '--no-ext-diff', '--unified=0', 'HEAD', '--', 'src'], capture=True, cwd=root).stdout
    path, result = None, set()
    for line in diff.splitlines():
        if line.startswith('+++ b/'): path = (root / line[6:]).resolve()
        elif line.startswith('@@') and path:
            start, count = re.search(r'\+(\d+)(?:,(\d+))?', line).groups()
            result.update((path, i) for i in range(int(start), int(start) + int(count or 1)))
    # Untracked production files have no diff yet.
    for name in run(['git', 'ls-files', '--others', '--exclude-standard', 'src'], capture=True, cwd=root).stdout.splitlines():
        p = root / name
        result.update((p.resolve(), i) for i, _ in enumerate(p.read_text().splitlines(), 1))
    return result


def verify(root):
    check_review(root)
    manifest = json.loads((root / 'coverage.json').read_text())
    check_ledger(root)
    run(['python3', '-m', 'unittest', 'discover', '-s', 'tools', '-p', 'test_*.py'], cwd=root)
    run(['cargo', 'fmt', '--all', '--check'], cwd=root)
    run(['cargo', 'clippy', '--locked', '--all-targets', '--', '-D', 'warnings'], cwd=root)
    run(['cargo', 'test', '--locked', '--doc'], cwd=root)
    metadata = json.loads(run(['cargo', 'metadata', '--locked', '--format-version=1'], capture=True, cwd=root).stdout)
    assert not any(p['name'].startswith(('bevy', 'gz-sim', 'gz-fuel', 'gz-transport')) for p in metadata['packages']), 'forbidden reverse dependency'
    coverage(root, root / 'artifacts' / 'core', manifest)


def export_index(root, destination):
    tree = run(['git', 'write-tree'], capture=True, cwd=root).stdout.strip()
    data = subprocess.run(['git', 'archive', tree], cwd=root, capture_output=True, check=True).stdout
    with tarfile.open(fileobj=io.BytesIO(data)) as archive:
        archive.extractall(destination, filter='data')
    return tree


def staged():
    tree = run(['git', 'write-tree'], capture=True).stdout.strip()
    with tempfile.TemporaryDirectory(prefix='staged-', dir=ROOT / 'target') as temp:
        assert export_index(ROOT, temp) == tree, 'index changed during export'
        snapshot = Path(temp)
        verify(snapshot)
        assert run(['git', 'write-tree'], capture=True).stdout.strip() == tree, 'index changed during verification'
        destination = ROOT / 'artifacts' / 'staged' / tree
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copytree(snapshot / 'artifacts/core', destination, dirs_exist_ok=True, ignore=shutil.ignore_patterns('target', 'raw'))
        print('Verified staged tree', tree)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--staged', action='store_true')
    parser.add_argument('--review-digest', action='store_true')
    args = parser.parse_args()
    if args.review_digest:
        print(review_digest(ROOT))
    elif args.staged:
        (ROOT / 'target').mkdir(exist_ok=True)
        staged()
    else: verify(ROOT)


if __name__ == '__main__': main()
