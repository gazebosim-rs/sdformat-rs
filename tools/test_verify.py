# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0

import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import verify


class CoverageGateTests(unittest.TestCase):
    def test_strict_integer_threshold(self):
        for covered in (0, 94, 95):
            with self.assertRaises(AssertionError):
                verify.check_counts({m: {'count': 100, 'covered': covered} for m in ('lines', 'regions')})
        verify.check_counts({m: {'count': 100, 'covered': 96} for m in ('lines', 'regions')})
        for metric in ('lines', 'regions'):
            counts = {m: {'count': 100, 'covered': 100} for m in ('lines', 'regions')}
            counts[metric] = {'count': 0, 'covered': 0}
            with self.assertRaises(AssertionError): verify.check_counts(counts)

    def test_failed_instrumented_tests_keep_output_and_failure(self):
        import subprocess
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            output = root / 'artifacts/core'
            failure = subprocess.CalledProcessError(101, ['cargo', 'test'], output='assertion failed\n', stderr='compiler diagnostic\n')
            with patch('verify.run', side_effect=failure) as command:
                with self.assertRaises(subprocess.CalledProcessError) as raised:
                    verify.coverage(root, output, {})
            self.assertIs(raised.exception, failure)
            self.assertEqual(raised.exception.returncode, 101)
            self.assertEqual((output / 'tests.log').read_text(), 'assertion failed\ncompiler diagnostic\n')
            command.assert_called_once()
            self.assertEqual(command.call_args.args[0], ['cargo', 'test', '--locked', '--all-targets', '--message-format=json'])
            self.assertTrue(command.call_args.kwargs['capture'])

    def test_missing_instrumentation_file_crate_profile_or_exclusion_fails(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / 'src').mkdir()
            (root / 'src/lib.rs').write_text('pub fn f() {}')
            manifest = {'profiles': ['core'], 'crates': {'sdformat': ['src']}, 'exclusions': []}
            report = {'type': 'llvm.coverage.json.export', 'data': [{'files': [{'filename': str(root / 'src/lib.rs'), 'summary': {m: {'count': 10, 'covered': 10} for m in ('lines', 'regions')}}]}]}
            verify.check_report(report, root, manifest)
            for key, value in [('profiles', []), ('crates', {}), ('exclusions', ['src/lib.rs'])]:
                broken = dict(manifest, **{key: value})
                with self.assertRaises(AssertionError): verify.check_report(report, root, broken)
            for change in ('instrumentation', 'file', 'duplicate', 'line', 'region', 'outside', 'include'):
                broken = copy.deepcopy(report)
                if change == 'instrumentation': broken['type'] = 'absent'
                elif change == 'file': broken['data'][0]['files'] = []
                elif change == 'duplicate': broken['data'][0]['files'] *= 2
                elif change == 'outside': broken['data'][0]['files'].append(dict(report['data'][0]['files'][0], filename=str(root / 'build.rs')))
                elif change == 'include': broken['data'][0]['files'].append({'filename': str(root / 'production.inc'), 'summary': {m: {'count': 100, 'covered': 0} for m in ('lines', 'regions')}})
                else: broken['data'][0]['files'][0]['summary'][change + 's']['covered'] = 9
                with self.assertRaises(AssertionError): verify.check_report(broken, root, manifest)

    def test_exported_tree_changed_lines_are_real_source(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / 'src').mkdir()
            file = root / 'src/lib.rs'
            file.write_text('one\ntwo\n')
            self.assertEqual({(file, 1), (file, 2)}, verify.changed_lines(root))


class LayoutGateTests(unittest.TestCase):
    def test_real_cargo_autodiscovery_and_features_cannot_escape_inventory(self):
        import subprocess
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / 'src').mkdir()
            (root / 'src/lib.rs').write_text('pub fn f() {}')
            cargo = '[package]\nname="sdformat"\nversion="0.1.0"\nedition="2024"\nlicense="Apache-2.0"\n'
            (root / 'Cargo.toml').write_text(cargo)
            manifest = {'profiles': ['core'], 'crates': {'sdformat': ['src']}, 'exclusions': [], 'runner': 'linux-x86_64'}
            def metadata():
                return json.loads(subprocess.run(['cargo', 'metadata', '--offline', '--no-deps', '--format-version=1'], cwd=root, check=True, text=True, capture_output=True).stdout)
            verify.check_layout(root, metadata(), manifest)
            # Cargo discovers these without explicit [[bin]]/[package].build.
            for relative in ('build.rs', 'src/main.rs', 'src/bin/tool.rs', 'examples/example.rs', 'benches/bench.rs'):
                path = root / relative
                path.parent.mkdir(exist_ok=True)
                path.write_text('compile_error!("must not compile before inventory rejection");')
                with self.subTest(target=relative), self.assertRaisesRegex(AssertionError, 'unaccounted Cargo target'):
                    verify.check_layout(root, metadata(), manifest)
                path.unlink()
            (root / 'Cargo.toml').write_text(cargo + '\n[features]\nextra=[]\n')
            with self.assertRaisesRegex(AssertionError, 'feature profiles'):
                verify.check_layout(root, metadata(), manifest)

    def test_packages_sources_runner_and_policy_changes_require_support(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / 'src').mkdir()
            (root / 'src/lib.rs').write_text('pub fn f() {}')
            manifest = {'profiles': ['core'], 'crates': {'sdformat': ['src']}, 'exclusions': [], 'runner': 'linux-x86_64'}
            package = {'id': 'owned', 'name': 'sdformat', 'manifest_path': str(root / 'Cargo.toml'), 'source': None, 'license': 'Apache-2.0', 'features': {}, 'targets': [{'kind': ['lib'], 'crate_types': ['lib'], 'src_path': str(root / 'src/lib.rs')}]}
            metadata = {'packages': [package], 'workspace_members': ['owned']}
            verify.check_layout(root, metadata, manifest)
            for key, value in [('profiles', []), ('crates', {}), ('exclusions', ['src/lib.rs']), ('runner', 'windows-x86_64')]:
                with self.subTest(policy=key), self.assertRaises(AssertionError):
                    verify.check_layout(root, metadata, dict(manifest, **{key: value}))
            with patch('verify.platform.system', return_value='Windows'), self.assertRaisesRegex(AssertionError, 'runner unavailable'):
                verify.check_layout(root, metadata, manifest)
            for key, value in [('workspace_members', ['owned', 'other']), ('packages', [package, dict(package, id='other')])]:
                with self.subTest(metadata=key), self.assertRaises(AssertionError):
                    verify.check_layout(root, dict(metadata, **{key: value}), manifest)
            for key, value in [('license', 'MIT'), ('targets', []), ('targets', [dict(package['targets'][0], src_path=str(root / 'other.rs'))])]:
                with self.subTest(package=key), self.assertRaises(AssertionError):
                    verify.check_layout(root, dict(metadata, packages=[dict(package, **{key: value})]), manifest)
            (root / 'forgotten.rs').write_text('pub fn omitted() {}')
            with self.assertRaisesRegex(AssertionError, 'unaccounted Rust source'):
                verify.check_layout(root, metadata, manifest)

class SnapshotAndReviewTests(unittest.TestCase):
    def test_production_schema_asset_participates_in_both_digests(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for name in ('data/world_known_schema_names.txt', 'Cargo.toml', 'Cargo.lock',
                         'rust-toolchain.toml', 'coverage.json', 'reference/traceability.json',
                         'reference/inventory.json', 'reference/results.json'):
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text('original')
            before = (verify.digest(root), verify.review_digest(root))
            import json
            receipt = {'decision': 'approved', 'reviewer': 'reviewer', 'author': 'author',
                       'protected_digest': before[1]}
            (root / 'reference/review.json').write_text(json.dumps(receipt))
            verify.check_review(root)
            (root / 'data/world_known_schema_names.txt').write_text('modified')
            after = (verify.digest(root), verify.review_digest(root))
            self.assertNotEqual(before[0], after[0])
            self.assertNotEqual(before[1], after[1])
            with self.assertRaisesRegex(AssertionError, 'independent review required'):
                verify.check_review(root)

    def test_schema_dep_info_requires_owned_library_and_exact_asset(self):
        import json
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); target = root / 'target'; deps = target / 'debug/deps'
            deps.mkdir(parents=True)
            (root / 'src').mkdir()
            library = deps / 'libsdformat-123.rlib'; library.write_bytes(b'library')
            dep = deps / 'sdformat-123.d'
            item = {'reason': 'compiler-artifact', 'target': {'name': 'sdformat', 'kind': ['lib'],
                    'src_path': str(root / 'src/lib.rs')}, 'profile': {'test': False},
                    'filenames': [str(library)]}
            output = json.dumps(item)
            dep.write_text('target: src/lib.rs src/parse.rs src/types.rs src/camera.rs src/../data/world_known_schema_names.txt\n')
            full = dep.read_text()
            for name in ('lib', 'parse', 'types', 'camera'):
                dep.write_text(full.replace('src/' + name + '.rs', 'src/' + name + '.rs.fake'))
                with self.assertRaisesRegex(AssertionError, 'constructor context'):
                    verify.check_schema_dep_info(output, root, target)
            dep.write_text(full)
            verify.check_schema_dep_info(output, root, target)
            for content in ('target: src/lib.rs\n', 'target: src/../data/world_known_schema_names.txt.fake\n'):
                dep.write_text(content)
                with self.assertRaisesRegex(AssertionError, 'schema asset'):
                    verify.check_schema_dep_info(output, root, target)
            dep.unlink()
            with self.assertRaisesRegex(AssertionError, 'dep-info'):
                verify.check_schema_dep_info(output, root, target)
            with self.assertRaisesRegex(AssertionError, 'missing or duplicate'):
                verify.check_schema_dep_info('', root, target)
            with self.assertRaisesRegex(AssertionError, 'missing or duplicate'):
                verify.check_schema_dep_info(output + '\n' + output, root, target)
            wrong = dict(item, target=dict(item['target'], src_path=str(root / 'other.rs')))
            with self.assertRaisesRegex(AssertionError, 'library source'):
                verify.check_schema_dep_info(json.dumps(wrong), root, target)
            foreign = root / 'libsdformat-foreign.rlib'; foreign.write_bytes(b'foreign')
            outside = dict(item, filenames=[str(foreign)])
            with self.assertRaisesRegex(AssertionError, 'production library'):
                verify.check_schema_dep_info(json.dumps(outside), root, target)

    def test_partial_staging_exports_only_index_and_detects_changes(self):
        import subprocess
        def git(root, *args):
            return subprocess.run(['git', *args], cwd=root, check=True, text=True, capture_output=True).stdout.strip()
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp)/'repo';root.mkdir()
            git(root,'init','-q')
            (root/'file').write_text('staged\n')
            (root/'.gitattributes').write_text('file export-ignore\n')
            git(root,'add','file','.gitattributes')
            (root/'file').write_text('unstaged\n')
            destination=Path(temp)/'export';destination.mkdir()
            tree=verify.export_index(root,destination)
            self.assertEqual((destination/'file').read_text(),'staged\n')
            self.assertEqual((root/'file').read_text(),'unstaged\n')
            git(root,'add','file')
            self.assertNotEqual(tree,git(root,'write-tree'))

    def test_assertion_removal_and_stale_review_fail(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);(root/'tests').mkdir();(root/'reference').mkdir()
            path=root/'tests/assertions.rs';path.write_text('assert!(important_contract);')
            receipt={'decision':'approved','reviewer':'different-agent','author':'implementation-agent','protected_digest':verify.review_digest(root)}
            (root/'reference/review.json').write_text(json.dumps(receipt))
            verify.check_review(root)
            path.write_text('// assertion removed')
            with self.assertRaises(AssertionError):verify.check_review(root)
            receipt['protected_digest']=verify.review_digest(root);receipt['reviewer']=receipt['author']
            (root/'reference/review.json').write_text(json.dumps(receipt))
            with self.assertRaises(AssertionError):verify.check_review(root)

    def test_verified_source_mutation_is_rejected(self):
        from types import SimpleNamespace
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / 'src').mkdir(); (root / 'reference').mkdir()
            source = root / 'src/lib.rs'; source.write_text('pub fn f() {}\n')
            for name in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'coverage.json',
                         'reference/traceability.json', 'reference/inventory.json', 'reference/results.json'):
                (root / name).write_text('{}')
            def mutate(*_): source.write_text('pub fn changed() {}\n')
            with patch('verify.check_review'), patch('verify.check_layout'), patch('verify.check_ledger'), \
                 patch('verify.run', return_value=SimpleNamespace(stdout='{"packages":[]}')), \
                 patch('verify.coverage', side_effect=mutate):
                with self.assertRaisesRegex(AssertionError, 'verified source changed'):
                    verify.verify(root)

    def test_failed_required_runner_is_failure(self):
        import subprocess
        with self.assertRaises(subprocess.CalledProcessError):
            verify.run(['false'])


class FrozenExportTests(unittest.TestCase):
    @staticmethod
    def git(root, *args):
        import subprocess
        return subprocess.run(['git', *args], cwd=root, check=True, text=True, capture_output=True).stdout.strip()

    def test_checkout_transform_is_rejected_before_verification(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / 'repo'; root.mkdir()
            self.git(root, 'init', '-q')
            self.git(root, 'config', 'core.autocrlf', 'false')
            (root / 'file').write_bytes(b'staged\n')
            (root / '.gitattributes').write_text('file text eol=crlf\n')
            self.git(root, 'add', 'file', '.gitattributes')
            destination = Path(temp) / 'export'; destination.mkdir()
            with self.assertRaisesRegex(ValueError, 'blob differs'):
                verify.export_index(root, destination)

    def test_snapshot_identity_includes_paths_types_modes_and_symlink_targets(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / 'repo'; root.mkdir()
            self.git(root, 'init', '-q')
            (root / 'file').write_bytes(b'original')
            (root / 'executable').write_bytes(b'run')
            (root / 'link').symlink_to('file')
            self.git(root, 'add', 'file', 'executable', 'link')
            self.git(root, 'update-index', '--chmod=+x', 'executable')
            destination = Path(temp) / 'export'; destination.mkdir()
            tree = verify.export_index(root, destination)
            file = destination / 'file'
            file.write_bytes(b'substituted')
            with self.assertRaisesRegex(ValueError, 'blob differs'):
                verify.validate_snapshot(root, tree, destination)
            file.write_bytes(b'original'); file.chmod(0o755)
            with self.assertRaisesRegex(ValueError, 'executable mode differs'):
                verify.validate_snapshot(root, tree, destination)
            file.chmod(0o644); file.unlink(); file.symlink_to('executable')
            with self.assertRaisesRegex(ValueError, 'file type differs'):
                verify.validate_snapshot(root, tree, destination)
            file.unlink(); file.write_bytes(b'original')
            link = destination / 'link'; link.unlink(); link.symlink_to('executable')
            with self.assertRaisesRegex(ValueError, 'blob differs'):
                verify.validate_snapshot(root, tree, destination)
            link.unlink(); link.symlink_to('file')
            executable = destination / 'executable'; executable.chmod(0o645)
            with self.assertRaisesRegex(ValueError, 'executable mode differs'):
                verify.validate_snapshot(root, tree, destination)
            executable.chmod(0o755)
            (destination / 'extra').write_bytes(b'extra')
            with self.assertRaisesRegex(ValueError, 'paths differ'):
                verify.validate_snapshot(root, tree, destination)
            (destination / 'extra').unlink()
            self.git(root, 'update-index', '--add', '--cacheinfo', '160000,' + '1' * 40 + ',submodule')
            with self.assertRaisesRegex(ValueError, 'non-blob staged input'):
                verify.validate_snapshot(root, self.git(root, 'write-tree'), destination)

    def test_index_and_head_races_cannot_publish_verified_evidence(self):
        for race in ('during_export', 'index', 'HEAD'):
            with self.subTest(race=race), tempfile.TemporaryDirectory() as temp:
                root = Path(temp) / 'repo'; root.mkdir(); (root / 'target').mkdir()
                self.git(root, 'init', '-q')
                (root / 'file').write_bytes(b'staged')
                self.git(root, 'add', 'file')
                def mutate():
                    if race == 'HEAD':
                        self.git(root, '-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.test', 'commit', '-qm', 'concurrent commit')
                    else:
                        (root / 'file').write_bytes(b'changed')
                        self.git(root, 'add', 'file')
                def fake_verify(snapshot, changed):
                    self.assertEqual((snapshot / 'file').read_bytes(), b'staged')
                    proof = snapshot / 'artifacts/core'; proof.mkdir(parents=True)
                    (proof / 'evidence.json').write_text('{}')
                    mutate()
                original_export = verify.export_index
                def racing_export(*args):
                    result = original_export(*args)
                    if race == 'during_export': mutate()
                    return result
                with patch.object(verify, 'ROOT', root), patch.object(verify, 'verify', side_effect=fake_verify), patch.object(verify, 'export_index', side_effect=racing_export), self.assertRaisesRegex(AssertionError, 'changed during'):
                    verify.staged()
                self.assertFalse((root / 'artifacts/staged').exists())


if __name__ == '__main__': unittest.main()
