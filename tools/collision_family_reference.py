#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay the retained Collision ownership and XML originals."""
import os
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parents[1]
upstream = Path(sys.argv[1]).resolve()
# This verifies the pinned source, original binary and its loaded libraries,
# then replays both native ToElement originals and the actual XML exchanges.
subprocess.run([sys.executable, str(root / 'tools/collision_xml_reference.py'), str(upstream)], cwd=root, check=True)
libdir = subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()
env = dict(os.environ, LD_LIBRARY_PATH=str(upstream / 'build/lib') + ':' + libdir)
native = subprocess.run([
    str(upstream / 'build/bin/UNIT_Collision_TEST'),
    '--gtest_filter=DOMCollision.MoveConstructor:DOMCollision.CopyConstructor:DOMCollision.MoveAssignment:DOMCollision.CopyAssignment:DOMCollision.CopyAssignmentAfterMove',
], env=env, capture_output=True, text=True, check=True, timeout=30)
assert '[  PASSED  ] 5 tests.' in native.stdout and not native.stderr
subprocess.run(['cargo', 'test', '--offline', '--locked', '--test', 'collision_ownership_originals'], cwd=root, check=True)
print('seven native originals and 13 Rust checks passed')
