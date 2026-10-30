# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import verify


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
            self.assertEqual(verify.check_changed_coverage(root, comments, covered), [])
            with self.assertRaisesRegex(AssertionError, 'no instrumentation'):
                verify.check_changed_coverage(root, comments, {})
            mixed = {'lines': {('src/lib.rs', 1), ('src/other.rs', 1)}}
            with self.assertRaisesRegex(AssertionError, 'no instrumentation'):
                verify.check_changed_coverage(root, mixed, covered)
            for changed in ({('src/lib.rs', 4)}, {('src/lib.rs', 1), ('src/lib.rs', 4)}):
                with self.assertRaisesRegex(AssertionError, 'no instrumented executable'):
                    verify.check_changed_coverage(root, {'lines': changed}, covered)

    def test_unsupported_quoted_path_fails_closed(self):
        with self.assertRaisesRegex(AssertionError, 'quoted source path'):
            verify.diff_lines('+++ "b/src/a\\tb.rs"\n@@ -1 +1 @@\n')

    def test_added_text_cannot_impersonate_a_diff_header(self):
        diff = 'diff --git a/src/a.rs b/src/a.rs\n+++ b/src/a.rs\n@@ -1 +1 @@\n+++ b/src/other.rs\n@@ -4 +4 @@\n'
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
