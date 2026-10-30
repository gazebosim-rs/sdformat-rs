# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import verify


def syntax(*, fields=(), closings=(), opaque=(), declarations=()):
    return lambda _path, _source: dict(fields=set(fields), closings=set(closings), opaque=set(opaque), declarations=set(declarations))


class ChangedCoverageTests(unittest.TestCase):
    def test_changed_subset_fails_even_when_aggregate_passes(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            path = (root / 'src/lib.rs').resolve()
            covered = {(path, i): i != 1 for i in range(1, 101)}
            verify.check_counts({m: {'count': 100, 'covered': 99} for m in ('lines', 'regions')})
            with self.assertRaisesRegex(AssertionError, 'changed executable'):
                verify.check_changed_coverage(root, {'lines': {('src/lib.rs', 1)}}, covered)
            changed = {'lines': {('src/lib.rs', i) for i in range(1, 21)}}
            with self.assertRaisesRegex(AssertionError, 'changed executable'):
                verify.check_changed_coverage(root, changed, covered)
            covered[(path, 1)] = True
            self.assertEqual(len(verify.check_changed_coverage(root, changed, covered)), 20)
            with self.assertRaises((AssertionError, FileNotFoundError)):
                verify.check_changed_coverage(root, {'lines': {('src/lib.rs', 101)}}, covered)

    def test_only_source_comments_or_blanks_are_explicitly_non_executable(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / 'src').mkdir()
            file = root / 'src/lib.rs'
            file.write_text('pub fn value() -> i32 { 1 }\n// explanation\n\npub fn other() -> i32 { 2 }\n')
            covered = {(file.resolve(), 1): True}
            comments = {'lines': {('src/lib.rs', 2), ('src/lib.rs', 3)}}
            self.assertEqual(verify.check_changed_coverage(root, comments, covered, syntax_provider=syntax()), [])
            with self.assertRaisesRegex(AssertionError, 'no instrumentation'):
                verify.check_changed_coverage(root, comments, {})
            mixed = {'lines': {('src/lib.rs', 1), ('src/other.rs', 1)}}
            with self.assertRaisesRegex(AssertionError, 'no instrumentation'):
                verify.check_changed_coverage(root, mixed, covered)
            for changed in ({('src/lib.rs', 4)}, {('src/lib.rs', 1), ('src/lib.rs', 4)}):
                with self.assertRaisesRegex(AssertionError, 'no instrumented executable'):
                    verify.check_changed_coverage(root, {'lines': changed}, covered, syntax_provider=syntax())

    def test_plain_impl_header_is_syntax_but_inline_code_is_not(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / 'src').mkdir()
            file = root / 'src/lib.rs'
            file.write_text('fn f() {}\nimpl Collision {\n}\nimpl Collision { fn f() {} }\nimpl Collision { fn f() {\nimpl crate::Collision {\n')
            covered = {(file.resolve(), 1): True}
            self.assertEqual(verify.check_changed_coverage(root, {'lines': {('src/lib.rs', 2)}}, covered, syntax_provider=syntax(declarations=[2])), [])
            for line in (4, 5, 6):
                with self.assertRaisesRegex(AssertionError, 'no instrumented executable'):
                    verify.check_changed_coverage(root, {'lines': {('src/lib.rs', 1), ('src/lib.rs', line)}}, covered, syntax_provider=syntax(declarations=[2]))

    def test_syntax_helper_output_and_coverage_priority_fail_closed(self):
        import json
        source = "pub struct World {\n    pub extensions: Vec<Thing>,\n}\n"
        value = {'source': source, 'helper_source': 'reviewed helper', 'ancestor': 'mod types;', 'module': 'types', 'fields': [2], 'closings': [], 'opaque': [], 'declarations': []}
        prefix = "\nrunning 1 test\nSDF_COVERAGE_SYNTAX_V1="
        suffix = "\ntest syntax_query ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.01s\n\n"
        output = lambda item: prefix + json.dumps(item) + suffix
        self.assertEqual(verify.parse_syntax_output(output(value), source, 'reviewed helper', 'mod types;', 'types')['fields'], {2})
        for broken in [dict(value, source=source + ' '), dict(value, helper_source='stale helper'), dict(value, ancestor='stale context'), dict(value, module='wrong'), dict(value, fields=[2, 2]), dict(value, fields=[4]), dict(value, fields=[True]), dict(value, extra=[]), dict(value, opaque='bad')]:
            with self.subTest(broken=broken), self.assertRaises(AssertionError):
                verify.parse_syntax_output(output(broken), source, 'reviewed helper', 'mod types;', 'types')
        for broken in [output(value).replace('SYNTAX_V1', 'SYNTAX_V0'), output(value) + 'unexpected', prefix + '{bad' + suffix, output(value).replace('{\"source\":', '{\"source\":\"duplicate\",\"source\":', 1)]:
            with self.subTest(broken=broken), self.assertRaises((AssertionError, ValueError)):
                verify.parse_syntax_output(broken, source, 'reviewed helper', 'mod types;', 'types')
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); (root / 'src').mkdir(); file = root / 'src/lib.rs'; file.write_text(source)
            changed = {'lines': {('src/lib.rs', 2)}}
            with self.assertRaisesRegex(AssertionError, 'syntax helper required'):
                verify.check_changed_coverage(root, changed, {(file.resolve(), 1): True})
            self.assertEqual(verify.check_changed_coverage(root, changed, {(file.resolve(), 1): True}, syntax_provider=syntax(fields=[2])), [])
            with self.assertRaisesRegex(AssertionError, 'opaque syntax'):
                verify.check_changed_coverage(root, changed, {(file.resolve(), 1): True}, syntax_provider=syntax(fields=[2], opaque=[2]))
            # A mapped uncovered line must fail before any syntax helper is called.
            with self.assertRaisesRegex(AssertionError, 'changed executable'):
                verify.check_changed_coverage(root, changed, {(file.resolve(), 2): False}, syntax_provider=lambda *_: self.fail('must not query syntax'))

    def test_mixed_call_closing_never_hides_uncovered_mapped_code(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); (root / 'src').mkdir()
            file = root / 'src/lib.rs'
            file.write_text('fn f() {\n    work(\n        1\n    );\n}\n')
            changed = {'lines': {('src/lib.rs', 2), ('src/lib.rs', 4)}}
            with self.assertRaisesRegex(AssertionError, 'changed executable'):
                verify.check_changed_coverage(root, changed, {(file.resolve(), 2): False},
                                              syntax_provider=lambda *_: self.fail('mapped code must fail first'))
            self.assertEqual(verify.check_changed_coverage(root, changed, {(file.resolve(), 2): True},
                                                          syntax_provider=syntax(closings=[4])), [True])

    def test_source_bytes_and_git_diff_preserve_carriage_returns(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            file = root / 'source.rs'
            file.write_bytes(b'a\rb\r\nc\n')
            self.assertEqual(verify.read_source(file), 'a\rb\r\nc\n')
            self.assertEqual(verify.source_lines(verify.read_source(file)), ['a\rb\r', 'c'])
            raw = verify.run_bytes(['python3', '-c', 'import sys;sys.stdout.buffer.write(b"a\\rb\\r\\nc\\n")'], cwd=root)
            self.assertEqual(raw, b'a\rb\r\nc\n')

    def test_real_git_diff_keeps_lone_cr_and_crlf_source_lines(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            def git(*args):
                return subprocess.check_output(['git', *args], cwd=root, text=True).strip()
            git('init', '-q'); (root / 'src').mkdir()
            file = root / 'src/lib.rs'
            file.write_bytes(b'const S: &str = r#"a\rb"#;\r\n// comment\r\npub fn old() {}\r\n')
            git('add', '.')
            git('-c', 'user.name=T', '-c', 'user.email=t@t', 'commit', '-qm', 'initial')
            parent = git('rev-parse', 'HEAD')
            file.write_bytes(b'const S: &str = r#"a\rb"#;\r\n// comment\r\npub fn changed() {}\r\n')
            with patch.dict(os.environ, {}, clear=True):
                changed = verify.changed_manifest(root, base=parent)
            self.assertEqual(changed['lines'], {('src/lib.rs', 3)})
            self.assertEqual(verify.source_lines(verify.read_source(file))[2], 'pub fn changed() {}\r')
            with self.assertRaisesRegex(AssertionError, 'no instrumented executable'):
                verify.check_changed_coverage(root, changed, {(file.resolve(), 1): True}, syntax_provider=syntax())

    def test_source_lines_follow_git_lf_not_unicode_separators(self):
        source = 'const S: &str = "a\u2028b";\npub struct World {\n    pub extensions: Vec<Thing>,\n}\n'
        self.assertEqual(len(verify.source_lines(source)), 4)
        self.assertEqual(verify.source_lines(source)[2], '    pub extensions: Vec<Thing>,')
        self.assertEqual(verify.source_lines(''), [])
        self.assertEqual(verify.source_lines('a'), ['a'])
        self.assertEqual(verify.source_lines('a\n'), ['a'])

    def test_unicode_separator_cannot_disguise_unmapped_changed_function(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); (root / 'src').mkdir()
            file = root / 'src/lib.rs'
            file.write_text('const S: &str = "a\u2028b";\n// comment\npub fn hidden() {}\n')
            changed = {'lines': {('src/lib.rs', 3)}}
            with self.assertRaisesRegex(AssertionError, 'no instrumented executable'):
                verify.check_changed_coverage(root, changed, {(file.resolve(), 1): True}, syntax_provider=syntax())

    def test_missing_duplicate_wrong_or_outside_helper_is_rejected(self):
        import json
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); (root / 'tests').mkdir(); (root / 'target').mkdir()
            binary = root / 'target/helper'; binary.write_text('binary')
            item = {'reason': 'compiler-artifact', 'executable': str(binary), 'profile': {'test': True},
                    'target': {'name': 'coverage_syntax', 'kind': ['test'], 'src_path': str(root / 'tests/coverage_syntax.rs')}}
            line = json.dumps(item)
            self.assertEqual(verify.test_binaries(line, root, root / 'target')[1], binary)
            unicode_binary = root / 'target/helper\u2028name'; unicode_binary.write_text('binary')
            unicode_item = dict(item, executable=str(unicode_binary))
            self.assertEqual(verify.test_binaries(json.dumps(unicode_item, ensure_ascii=False), root, root / 'target')[1], unicode_binary)
            for output in ('', line + '\n' + line, json.dumps(dict(item, executable=str(root / 'missing'))),
                           json.dumps(dict(item, target=dict(item['target'], src_path=str(root / 'tests/wrong.rs')))),
                           json.dumps(dict(item, target=dict(item['target'], kind=['bin'])))):
                with self.subTest(output=output), self.assertRaises(AssertionError):
                    verify.test_binaries(output, root, root / 'target')
            outside = root / 'outside'; outside.write_text('binary')
            with self.assertRaises(AssertionError):
                verify.test_binaries(json.dumps(dict(item, executable=str(outside))), root, root / 'target')

    def test_unsupported_quoted_path_fails_closed(self):
        with self.assertRaisesRegex(AssertionError, 'quoted source path'):
            verify.diff_lines('+++ "b/src/a\\tb.rs"\n@@ -1 +1 @@\n')

    def test_added_text_cannot_impersonate_a_diff_header(self):
        diff = 'diff --git a/src/a.rs b/src/a.rs\n+++ b/src/a.rs\n@@ -1 +1 @@\n+++ b/src/other.rs\n@@ -4 +4 @@\n'
        self.assertEqual(verify.diff_lines(diff), {('src/a.rs', 1), ('src/a.rs', 4)})

    def test_lcov_paths_do_not_split_at_unicode_separator(self):
        path = Path('/tmp/source\u2028name.rs').resolve()
        values = verify.lcov_records(f'SF:{path}\nDA:3,1\nend_of_record\n')
        self.assertEqual(values, {(path, 3): True})

    def test_unicode_separator_inside_patch_cannot_reset_diff_state(self):
        diff = ('diff --git a/src/a.rs b/src/a.rs\n+++ b/src/a.rs\n@@ -1 +1 @@\n'
                '+inside\u2028diff --git a/src/evil.rs b/src/evil.rs\n'
                '+++ b/src/evil.rs\n@@ -4 +4 @@\n')
        self.assertEqual(verify.diff_lines(diff), {('src/a.rs', 1), ('src/a.rs', 4)})

    def test_clean_checkout_uses_parent_and_frozen_index_is_distinct(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            def git(*args):
                return subprocess.check_output(['git', *args], cwd=root, text=True).strip()
            git('init', '-q')
            (root / 'src').mkdir()
            file = root / 'src/lib.rs'
            file.write_text('one\n')
            git('add', '.')
            git('-c', 'user.name=T', '-c', 'user.email=t@t', 'commit', '-qm', 'initial')
            parent = git('rev-parse', 'HEAD')
            with patch.dict(os.environ, {}, clear=True):
                first = verify.changed_manifest(root)
            self.assertEqual(first['mode'], 'full-source')
            self.assertEqual(first['lines'], {('src/lib.rs', 1)})
            file.write_text('one\ntwo\n')
            git('add', '.')
            tree = git('write-tree')
            file.write_text('one\ntwo\nunstaged\n')
            staged = verify.changed_manifest(root, parent, tree)
            self.assertEqual(staged['base'], parent)
            self.assertEqual(staged['target'], tree)
            self.assertEqual(staged['lines'], {('src/lib.rs', 2)})
            with self.assertRaisesRegex(AssertionError, 'changed executable'):
                verify.check_changed_coverage(root, staged, {(file.resolve(), 2): False})
            git('-c', 'user.name=T', '-c', 'user.email=t@t', 'commit', '-qm', 'second')
            file.write_text('one\ntwo\n')
            with patch.dict(os.environ, {'SDF_COVERAGE_BASE': parent}, clear=True):
                hosted = verify.changed_manifest(root)
            self.assertEqual(hosted['lines'], {('src/lib.rs', 2)})
            self.assertEqual(hosted['base'], parent)
            with patch.dict(os.environ, {'SDF_COVERAGE_BASE': 'not-a-commit'}, clear=True):
                with self.assertRaises(subprocess.CalledProcessError):
                    verify.changed_manifest(root)
