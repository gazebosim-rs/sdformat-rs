# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0

import copy
import json
from pathlib import Path
import tempfile
import unittest
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
            for change in ('instrumentation', 'file', 'duplicate', 'line', 'region'):
                broken = copy.deepcopy(report)
                if change == 'instrumentation': broken['type'] = 'absent'
                elif change == 'file': broken['data'][0]['files'] = []
                elif change == 'duplicate': broken['data'][0]['files'] *= 2
                else: broken['data'][0]['files'][0]['summary'][change + 's']['covered'] = 9
                with self.assertRaises(AssertionError): verify.check_report(broken, root, manifest)

    def test_exported_tree_changed_lines_are_real_source(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / 'src').mkdir()
            file = root / 'src/lib.rs'
            file.write_text('one\ntwo\n')
            self.assertEqual({(file, 1), (file, 2)}, verify.changed_lines(root))


if __name__ == '__main__': unittest.main()

class SnapshotAndReviewTests(unittest.TestCase):
    def test_partial_staging_exports_only_index_and_detects_changes(self):
        import subprocess
        def git(root, *args):
            return subprocess.run(['git', *args], cwd=root, check=True, text=True, capture_output=True).stdout.strip()
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp)/'repo';root.mkdir()
            git(root,'init','-q')
            (root/'file').write_text('staged\n')
            git(root,'add','file')
            (root/'file').write_text('unstaged\n')
            destination=Path(temp)/'export';destination.mkdir()
            tree=verify.export_index(root,destination)
            self.assertEqual((destination/'file').read_text(),'staged\n')
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

    def test_failed_required_runner_is_failure(self):
        import subprocess
        with self.assertRaises(subprocess.CalledProcessError):
            verify.run(['false'])
