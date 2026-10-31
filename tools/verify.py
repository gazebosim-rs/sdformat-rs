#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0

"""Single local/CI gate; coverage uses the pinned toolchain's LLVM binaries."""
import argparse
from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def run(command, *, env=None, capture=False, cwd=ROOT):
    print('+', ' '.join(map(str, command)), flush=True)
    return subprocess.run(command, cwd=cwd, env=env, check=True, text=True, capture_output=capture)


def run_bytes(command, *, cwd=ROOT):
    print('+', ' '.join(map(str, command)), flush=True)
    return subprocess.run(command, cwd=cwd, check=True, capture_output=True).stdout


def read_source(path):
    return path.read_bytes().decode('utf-8')


def digest(root):
    files = sorted(p for pattern in ('src/**/*.rs', 'data/world_known_schema_names.txt', 'tests/**/*', 'tools/*.py') for p in root.glob(pattern) if p.is_file())
    files += [root / p for p in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'coverage.json', 'reference/traceability.json', 'reference/inventory.json', 'reference/results.json')]
    state = hashlib.sha256()
    for path in files:
        state.update(str(path.relative_to(root)).encode())
        state.update(path.read_bytes())
    return state.hexdigest()


def review_digest(root):
    protected = sorted(p for pattern in ('src/**/*.rs', 'data/world_known_schema_names.txt', 'LICENSE', 'NOTICE', 'README.md', 'tests/**/*', 'tools/*', '.githooks/*', '.github/workflows/*', '.pre-commit-config.yaml', 'coverage.json', 'AGENTS.md', 'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'reference/*.json') for p in root.glob(pattern) if p.is_file() and p.name != 'review.json')
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


def check_layout(root, metadata, manifest):
    """Fail closed before Cargo can execute an unaccounted build target."""
    assert manifest['profiles'] == ['core'], 'profile matrix changed without gate support'
    assert manifest['crates'] == {'sdformat': ['src']}, 'crate matrix changed without gate support'
    assert manifest['exclusions'] == [], 'coverage exclusions require review and implementation'
    assert manifest['runner'] == 'linux-x86_64', 'runner matrix changed without gate support'
    assert (platform.system(), platform.machine()) == ('Linux', 'x86_64'), 'required linux-x86_64 runner unavailable'
    owned = [p for p in metadata['packages'] if p['source'] is None]
    assert len(owned) == 1, 'additional local packages need production coverage support'
    package = owned[0]
    assert package['name'] == 'sdformat' and Path(package['manifest_path']).resolve() == root / 'Cargo.toml', 'unexpected production package'
    assert metadata['workspace_members'] == [package['id']], 'workspace matrix changed without gate support'
    assert not package['features'], 'feature profiles need explicit coverage support'
    assert package['license'] == 'Apache-2.0', 'owned package license must remain Apache-2.0'
    libraries = []
    for target in package['targets']:
        path = Path(target['src_path']).resolve()
        if target['kind'] == ['lib']:
            libraries.append(target)
            assert target['crate_types'] == ['lib'] and path == root / 'src/lib.rs', 'unaccounted library source or crate type'
        else:
            assert target['kind'] == ['test'] and path.is_relative_to(root / 'tests'), 'unaccounted Cargo target needs coverage support'
    assert len(libraries) == 1, 'exactly one production library is required'
    # Cargo metadata inventories entry points, but not module/include sources.
    # Keep owned Rust inside the two reviewed roots; output trees are not inputs.
    for directory, children, files in os.walk(root):
        if Path(directory) == root:
            children[:] = [c for c in children if c not in ('.git', 'target', 'artifacts')]
        for name in files:
            if name.endswith('.rs'):
                path = (Path(directory) / name).resolve()
                assert path.is_relative_to(root / 'src') or path.is_relative_to(root / 'tests'), f'unaccounted Rust source {path}'


def check_report(report, root, manifest):
    assert report.get('type') == 'llvm.coverage.json.export', 'missing instrumentation'
    files = {}
    for item in report['data']:
        for file in item['files']:
            path = Path(file['filename']).resolve()
            if path.is_relative_to(root):
                assert path.is_relative_to(root / 'src') or (path.is_relative_to(root / 'tests') and path.suffix == '.rs'), f'unaccounted instrumented source {path}'
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


def test_binaries(output, root, target):
    binaries = []
    syntax_binaries = []
    for line in output.split('\n'):
        if line.startswith('{'):
            item = json.loads(line)
            if item.get('reason') == 'compiler-artifact' and item.get('executable') and item['profile']['test']:
                binaries.append(item['executable'])
                if item['target']['name'] == 'coverage_syntax':
                    assert item['target']['kind'] == ['test'] and Path(item['target']['src_path']).resolve() == (root / 'tests/coverage_syntax.rs').resolve(), 'unexpected coverage syntax helper source'
                    syntax_binaries.append(item['executable'])
    assert len(syntax_binaries) == 1, 'missing or duplicate coverage syntax helper'
    syntax_binary = Path(syntax_binaries[0]).resolve()
    assert syntax_binary.is_file() and syntax_binary.is_relative_to(target.resolve()), 'invalid coverage syntax helper'
    assert binaries, 'no instrumented test binaries'
    return binaries, syntax_binary


def check_schema_dep_info(output, root, target):
    """Bind the production data asset to the actual compiled library."""
    artifacts = []
    for line in output.split('\n'):
        if not line.startswith('{'):
            continue
        item = json.loads(line)
        descriptor = item.get('target', {})
        if (item.get('reason') == 'compiler-artifact' and descriptor.get('name') == 'sdformat'
                and descriptor.get('kind') == ['lib'] and not item['profile']['test']):
            assert Path(descriptor['src_path']).resolve() == (root / 'src/lib.rs').resolve(), 'unexpected library source'
            artifacts += [Path(name) for name in item['filenames'] if name.endswith('.rlib')]
    assert len(artifacts) == 1, 'missing or duplicate production library'
    library = artifacts[0].resolve()
    assert library.is_file() and library.is_relative_to(target.resolve()), 'invalid production library'
    assert library.name.startswith('libsdformat-'), 'unexpected production library name'
    dep_info = library.with_name(library.name.removeprefix('lib').removesuffix('.rlib') + '.d')
    assert dep_info.is_file(), 'missing production dep-info'
    text = read_source(dep_info)
    assert re.search(r'(?<!\S)src/\.\./data/world_known_schema_names\.txt(?=\s|$)', text), 'production schema asset missing from dep-info'
    for source in ('lib', 'parse', 'types', 'camera'):
        assert re.search(r'(?<!\S)src/' + source + r'\.rs(?=\s|$)', text), 'constructor context missing from production dep-info'


def coverage(root, out, manifest, changed=None):
    out.mkdir(parents=True, exist_ok=True)
    raw = out / 'raw'
    if raw.exists(): shutil.rmtree(raw)
    raw.mkdir()
    env = dict(os.environ, CARGO_TARGET_DIR=str(out / 'target'), LLVM_PROFILE_FILE=str(raw / '%p-%m.profraw'))
    env['RUSTFLAGS'] = '-C instrument-coverage'
    env.pop('SDF_COVERAGE_SYNTAX_QUERY', None)
    try:
        result = run(['cargo', 'test', '--locked', '--all-targets', '--message-format=json'], env=env, capture=True, cwd=root)
    except subprocess.CalledProcessError as error:
        (out / 'tests.log').write_text((error.stdout or '') + (error.stderr or ''))
        raise
    (out / 'tests.log').write_text(result.stdout + result.stderr)
    binaries, syntax_binary = test_binaries(result.stdout, root, out / 'target')
    check_schema_dep_info(result.stdout, root, out / 'target')
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
    covered = lcov_records(lcov)
    full = check_changed_coverage(root, {'lines': all_source_lines(root)}, covered, strict=False)
    changed = changed if changed is not None else changed_manifest(root)
    def syntax_provider(path, source):
        query_env = dict(os.environ, SDF_COVERAGE_SYNTAX_QUERY='1', RUST_TEST_THREADS='2',
                         LLVM_PROFILE_FILE=str(out / 'syntax-query-%p-%m.profraw'))
        ancestor = read_source(root / 'src/lib.rs')
        module = path.stem if path.parent == (root / 'src').resolve() and path.name != 'lib.rs' else ''
        exports = {name: read_source(root / 'src' / (name + '.rs')) for name in ('types', 'camera')} if module == 'parse' else {}
        request = json.dumps({'source': source, 'ancestor': ancestor, 'module': module, 'exports': exports})
        result = subprocess.run([str(syntax_binary), '--exact', 'syntax_query', '--nocapture', '--color', 'never'],
                                input=request, capture_output=True, text=True, env=query_env, cwd=root, check=True, timeout=30)
        assert not result.stderr, 'coverage syntax helper stderr'
        return parse_syntax_output(result.stdout, source, read_source(root / 'tests/coverage_syntax.rs'), ancestor, module, exports)
    measured = check_changed_coverage(root, changed, covered, strict=changed['mode'] != 'full-source', syntax_provider=syntax_provider)
    evidence = {'source_digest': digest(root), 'toolchain': run(['rustc', '-vV'], capture=True, cwd=root).stdout, 'runner': os.uname().nodename, 'profile': 'core', 'coverage': counts, 'changed_lines': {'count': len(measured), 'covered': sum(measured), 'base': changed['base'], 'target': changed['target'], 'mode': changed['mode'], 'status': 'measured' if measured else ('no executable changes' if changed['lines'] else 'no added source lines')}, 'full_source_lines': {'count': len(full), 'covered': sum(full)}}
    (out / 'evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(json.dumps(evidence, indent=2))


def lcov_records(lcov):
    covered = {}
    path = None
    for line in lcov.split('\n'):
        if line.startswith('SF:'): path = Path(line[3:]).resolve()
        elif line.startswith('DA:') and path:
            number, count, *_ = line[3:].split(',')
            covered[(path, int(number))] = int(count) > 0
    return covered


def source_lines(text):
    if not text:
        return []
    lines = text.split('\n')
    return lines[:-1] if text.endswith('\n') else lines


def all_source_lines(root):
    return {(str(p.relative_to(root)), i) for p in (root / 'src').rglob('*.rs')
            for i, _ in enumerate(source_lines(read_source(p)), 1)}


def diff_lines(diff):
    path, result, in_hunk = None, set(), False
    for line in diff.split('\n'):
        if line.startswith('diff --git '):
            path, in_hunk = None, False
        elif not in_hunk and line.startswith('+++ b/'):
            path = line[6:]
        elif not in_hunk and line.startswith('+++ '):
            assert line == '+++ /dev/null', 'quoted source path cannot be measured'
            path = None
        elif line.startswith('@@') and path:
            in_hunk = True
            start, count = re.search(r'\+(\d+)(?:,(\d+))?', line).groups()
            result.update((path, i) for i in range(int(start), int(start) + int(count or 1)))
    return result


def changed_manifest(root, base=None, target=None):
    if not (root / '.git').exists():
        return {'base': None, 'target': None, 'mode': 'full-source', 'lines': all_source_lines(root)}
    head = head_revision(root)
    if base is None:
        base = os.environ.get('SDF_COVERAGE_BASE')
        if not base:
            parents = run(['git', 'rev-list', '--parents', '-n', '1', 'HEAD'], capture=True, cwd=root).stdout.split()
            base = parents[1] if len(parents) > 1 else None
    if not base or set(base) == {'0'}:
        return {'base': None, 'target': target or head, 'mode': 'full-source', 'lines': all_source_lines(root)}
    base = run(['git', 'rev-parse', '--verify', base + '^{commit}'], capture=True, cwd=root).stdout.strip()
    args = ['git', 'diff', '--no-ext-diff', '--unified=0', base]
    if target is not None:
        args.append(target)
    diff = run_bytes(args + ['--', 'src'], cwd=root).decode('utf-8')
    lines = diff_lines(diff)
    if target is None:
        raw = run_bytes(['git', 'ls-files', '--others', '--exclude-standard', '-z', '--', 'src'], cwd=root)
        for entry in raw.split(b'\0'):
            if not entry:
                continue
            name = entry.decode('utf-8')
            assert not any(ord(char) < 32 or ord(char) == 127 for char in name), 'unsupported source path'
            lines.update((name, i) for i, _ in enumerate(source_lines(read_source(root / name)), 1))
    return {'base': base, 'target': target or head, 'mode': 'parent-diff', 'lines': lines}


def parse_syntax_output(output, source, helper_source, ancestor, module, exports=None):
    prefix = '\nrunning 1 test\nSDF_COVERAGE_SYNTAX_V1='
    assert output.startswith(prefix), 'invalid coverage syntax helper output'
    def unique_object(pairs):
        result = {}
        for key, value in pairs:
            assert key not in result, 'duplicate coverage syntax key'
            result[key] = value
        return result
    value, end = json.JSONDecoder(object_pairs_hook=unique_object).raw_decode(output[len(prefix):])
    suffix = output[len(prefix) + end:]
    assert re.fullmatch(r'\ntest syntax_query \.\.\. ok\n\ntest result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; \d+ filtered out; finished in [0-9.]+s\n\n', suffix), 'invalid coverage syntax helper footer'
    assert isinstance(value, dict) and set(value) == {'source', 'ancestor', 'module', 'helper_source', 'fields', 'closings', 'opaque', 'declarations', 'items', 'else_delimiters', 'exports', 'aggregate_prefixes'}, 'invalid coverage syntax helper schema'
    assert value['source'] == source, 'stale coverage syntax source'
    assert value['helper_source'] == helper_source, 'stale coverage syntax helper'
    assert value['ancestor'] == ancestor and value['module'] == module, 'stale coverage syntax context'
    assert value['exports'] == (exports or {}), 'stale constructor export context'
    limit = len(source_lines(source))
    for key in ('fields', 'closings', 'opaque', 'declarations', 'items', 'else_delimiters', 'aggregate_prefixes'):
        lines = value[key]
        assert isinstance(lines, list) and all(type(line) is int and 1 <= line <= limit for line in lines), 'invalid coverage syntax line'
        assert lines == sorted(set(lines)), 'duplicate or unordered coverage syntax line'
        value[key] = set(lines)
    return value


def check_changed_coverage(root, changed, covered, strict=True, syntax_provider=None):
    instrumented = {path for path, _ in covered}
    for name, _ in changed['lines']:
        assert (root / name).resolve() in instrumented, 'changed source has no instrumentation'
    measured = [covered[key] for name, line in changed['lines']
                if (key := ((root / name).resolve(), line)) in covered]
    if measured:
        assert sum(measured) * 100 > len(measured) * 95, 'changed executable line coverage must exceed 95%'
    if strict:
        pending = {}
        for name, line in changed['lines']:
            path = (root / name).resolve()
            if (path, line) not in covered:
                pending.setdefault(path, set()).add(line)
        for path, numbers in pending.items():
            assert syntax_provider is not None, 'coverage syntax helper required'
            source = read_source(path)
            lines = source_lines(source)
            syntax = syntax_provider(path, source)
            for line in numbers:
                assert 1 <= line <= len(lines), 'changed line outside source'
                text = lines[line - 1].strip()
                assert line not in syntax['opaque'], 'changed source is opaque syntax'
                non_executable = (line in syntax['items'] or not text or text.startswith('//') or text in ('{', '}', '};')
                                  or (line in syntax['declarations'] and (re.fullmatch(r'impl\s+[A-Za-z_][A-Za-z0-9_]*\s*\{', text)
                                      or re.fullmatch(r'(?:pub(?:\([^)]*\))?\s+)?(?:use|mod)\s+[^;]+;', text)))
                                  or (line in syntax['fields'] and re.fullmatch(r'pub\s+[A-Za-z_][A-Za-z0-9_]*:\s*(?:Vec<[A-Za-z_][A-Za-z0-9_]*>|Option<String>|String),', text))
                                  or (line in syntax['closings'] and text in (');', '},', '})'))
                                  or (line in syntax['else_delimiters'] and text == '} else {')
                                  or line in syntax['aggregate_prefixes'])
                assert non_executable, 'changed source has no instrumented executable lines'
    return measured

def changed_lines(root):
    return {((root / path).resolve(), line) for path, line in changed_manifest(root)['lines']}

def verify(root, changed=None):
    input_digest = digest(root)
    check_review(root)
    manifest = json.loads((root / 'coverage.json').read_text())
    metadata = json.loads(run(['cargo', 'metadata', '--locked', '--format-version=1'], capture=True, cwd=root).stdout)
    check_layout(root, metadata, manifest)
    assert not any(p['name'].startswith(('bevy', 'gz-sim', 'gz-fuel', 'gz-transport')) for p in metadata['packages']), 'forbidden reverse dependency'
    check_ledger(root)
    run(['python3', '-m', 'unittest', 'discover', '-s', 'tools', '-p', 'test_*.py'], cwd=root)
    run(['cargo', 'fmt', '--all', '--check'], cwd=root)
    run(['cargo', 'clippy', '--locked', '--all-targets', '--', '-D', 'warnings'], cwd=root)
    run(['cargo', 'test', '--locked', '--doc'], cwd=root)
    coverage(root, root / 'artifacts' / 'core', manifest, changed)
    assert digest(root) == input_digest, 'verified source changed during checks'


def validate_snapshot(root, tree, candidate):
    algorithm = subprocess.check_output(['git', 'rev-parse', '--show-object-format'], cwd=root, text=True).strip()
    expected = set()
    for entry in subprocess.check_output(['git', 'ls-tree', '-rz', tree], cwd=root).split(b'\0'):
        if not entry:
            continue
        metadata, name = entry.split(b'\t', 1)
        mode, kind, oid = metadata.split()
        relative = os.fsdecode(name)
        expected.add(relative)
        path = candidate / relative
        if kind != b'blob':
            raise ValueError('Unsupported non-blob staged input')
        if mode == b'120000' and path.is_symlink():
            data = os.fsencode(os.readlink(path))
        elif mode in (b'100644', b'100755') and path.is_file() and not path.is_symlink():
            if bool(path.stat().st_mode & 0o100) != (mode == b'100755'):
                raise ValueError('Exported staged executable mode differs from Git tree')
            data = path.read_bytes()
        else:
            raise ValueError('Exported staged file type differs from Git tree')
        digest = hashlib.new(algorithm, f'blob {len(data)}\0'.encode() + data).hexdigest()
        if digest != oid.decode():
            raise ValueError(f'Exported staged blob differs from Git tree: {relative}')
    actual = {str(path.relative_to(candidate)) for path in candidate.rglob('*') if path.is_file() or path.is_symlink()}
    if actual != expected:
        raise ValueError('Exported staged paths differ from Git tree')


def export_index(root, destination, tree=None):
    tree = tree or run(['git', 'write-tree'], capture=True, cwd=root).stdout.strip()
    (root / 'target').mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='index-', dir=root / 'target') as directory:
        env = dict(os.environ, GIT_INDEX_FILE=str(Path(directory) / 'index'))
        run(['git', 'read-tree', tree], env=env, cwd=root)
        run(['git', 'checkout-index', '--all', f'--prefix={Path(destination).resolve()}/'], env=env, cwd=root)
    validate_snapshot(root, tree, Path(destination))
    return tree


def head_revision(root):
    result = subprocess.run(['git', 'rev-parse', '--verify', 'HEAD'], cwd=root, text=True, capture_output=True)
    return result.stdout.strip() if result.returncode == 0 else None


@contextmanager
def isolated_git_context(root):
    # Hooks export repository-local Git variables. Verification creates fixture
    # repositories, so retain that context only for the authoritative parent.
    keys = run(['git', 'rev-parse', '--local-env-vars'], capture=True, cwd=root).stdout.splitlines()
    previous = {key: os.environ.pop(key) for key in keys if key in os.environ}
    try:
        yield
    finally:
        for key in keys:
            os.environ.pop(key, None)
        os.environ.update(previous)


def staged():
    tree = run(['git', 'write-tree'], capture=True, cwd=ROOT).stdout.strip()
    head = head_revision(ROOT)
    changed = changed_manifest(ROOT, head, tree) if head else {'base': None, 'target': tree, 'mode': 'full-source', 'lines': None}
    with tempfile.TemporaryDirectory(prefix='staged-', dir=ROOT / 'target') as temp:
        export_index(ROOT, temp, tree)
        assert run(['git', 'write-tree'], capture=True, cwd=ROOT).stdout.strip() == tree, 'index changed during export'
        snapshot = Path(temp)
        if changed['lines'] is None:
            changed['lines'] = all_source_lines(snapshot)
        with isolated_git_context(ROOT):
            verify(snapshot, changed)
        assert run(['git', 'write-tree'], capture=True, cwd=ROOT).stdout.strip() == tree, 'index changed during verification'
        assert head_revision(ROOT) == head, 'HEAD changed during verification'
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
