# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
import verify


class GitContextTests(unittest.TestCase):
    def test_failed_child_restores_parent_locals_and_preserves_nonlocal_settings(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(['git', 'init', '--quiet', str(root)], check=True)
            with patch.dict(os.environ, GIT_DIR=str(root / '.git'), GIT_WORK_TREE=str(root),
                            GIT_INDEX_FILE=str(root / 'alternate-index'), GIT_SSH_COMMAND='preserved'):
                before = dict(os.environ)
                with self.assertRaisesRegex(RuntimeError, 'child failure'):
                    with verify.isolated_git_context(root):
                        self.assertNotIn('GIT_DIR', os.environ)
                        self.assertNotIn('GIT_WORK_TREE', os.environ)
                        self.assertNotIn('GIT_INDEX_FILE', os.environ)
                        self.assertEqual(os.environ['GIT_SSH_COMMAND'], 'preserved')
                        os.environ['GIT_DIR'] = 'child-local-value'
                        raise RuntimeError('child failure')
                self.assertEqual(dict(os.environ), before)

    def test_real_worktree_commit_isolates_child_and_restores_parent_context(self):
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            repo = base / 'parent'
            repo.mkdir()
            def git(*args, cwd=repo, env=None):
                return subprocess.check_output(['git', *args], cwd=cwd, env=env, text=True).strip()
            git('init', '--quiet')
            (repo / 'src').mkdir()
            (repo / 'src/lib.rs').write_text('intended staged bytes\n')
            (repo / 'tools').mkdir()
            shutil.copyfile(verify.__file__, repo / 'tools/verify.py')
            foreign = base / 'foreign repository'
            proof = base / 'child-proof.json'
            driver = f'''import json,os,subprocess
from pathlib import Path
import verify
before=dict(os.environ)
def child(snapshot, changed):
    keys=subprocess.check_output(['git','rev-parse','--local-env-vars'],text=True).splitlines()
    assert not any(key in os.environ for key in keys)
    assert os.environ['GIT_SSH_COMMAND']=='nonlocal-setting-preserved'
    assert (snapshot/'src/lib.rs').read_text()=='intended staged bytes\\n'
    assert changed['target'] and changed['base']
    foreign=Path({str(foreign)!r})
    subprocess.run(['git','init','--quiet',str(foreign)],check=True)
    (foreign/'value').write_text('independent')
    subprocess.run(['git','add','.'],cwd=foreign,check=True)
    subprocess.run(['git','-c','user.name=Gate test','-c','user.email=gate@example.invalid','commit','--quiet','-m','foreign'],cwd=foreign,check=True)
    (snapshot/'artifacts/core').mkdir(parents=True)
    Path({str(proof)!r}).write_text(json.dumps({{'checked':True}}))
verify.verify=child
verify.main()
assert dict(os.environ)==before
'''
            (repo / 'tools/driver.py').write_text(driver)
            git('add', '.')
            git('-c', 'user.name=Gate test', '-c', 'user.email=gate@example.invalid', 'commit', '--quiet', '-m', 'fixture')
            original_head = git('rev-parse', 'HEAD')
            worktree = base / 'linked worktree'
            git('worktree', 'add', '--quiet', '--detach', str(worktree), 'HEAD')
            git_dir = Path(git('rev-parse', '--absolute-git-dir', cwd=worktree))
            alternate = base / 'intended index'
            shutil.copyfile(git_dir / 'index', alternate)
            (worktree / 'src/lib.rs').write_text('wrong normal index\n')
            git('add', 'src/lib.rs', cwd=worktree)
            normal_index = (git_dir / 'index').read_bytes()
            config = repo / '.git/config'
            original_config = config.read_bytes()
            hooks = base / 'hooks'
            hooks.mkdir()
            hook = hooks / 'pre-commit'
            hook.write_text('#!/bin/sh\nexec ' + shlex.quote(sys.executable) + ' tools/driver.py --staged\n')
            hook.chmod(0o755)
            env = dict(os.environ, GIT_DIR=str(git_dir), GIT_COMMON_DIR=str(repo / '.git'),
                       GIT_INDEX_FILE=str(alternate), GIT_SSH_COMMAND='nonlocal-setting-preserved')
            env.pop('GIT_WORK_TREE', None)
            result = subprocess.run(['git', '-c', 'core.hooksPath=' + str(hooks), '-c', 'user.name=Gate test',
                                     '-c', 'user.email=gate@example.invalid', 'commit', '--quiet', '--allow-empty', '-m', 'hook checked'],
                                    cwd=worktree, env=env, capture_output=True, text=True)
            self.assertEqual(config.read_bytes(), original_config, result.stderr)
            self.assertEqual((git_dir / 'index').read_bytes(), normal_index, result.stderr)
            self.assertEqual(git('rev-parse', 'HEAD'), original_head, result.stderr)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(json.loads(proof.read_text()), {'checked': True})
            self.assertTrue((foreign / '.git/HEAD').is_file())


if __name__ == '__main__':
    unittest.main()
