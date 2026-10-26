# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0

"""The static-model preimage guard survives reviewed later test edits."""

import json
from pathlib import Path
import shutil
import tempfile
import unittest

from static_model_reference import check_disposition


ROOT = Path(__file__).resolve().parents[1]


class StaticModelDispositionTest(unittest.TestCase):
    def test_reviewed_successor_states_and_unreviewed_mutation(self):
        disposition = json.loads((ROOT / 'reference/static_model_disposition.json').read_text())
        with tempfile.TemporaryDirectory() as directory:
            snapshot = Path(directory)
            for name in set(disposition['files']) | set(disposition['unchanged_input_sha256']):
                (snapshot / name).parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(ROOT / name, snapshot / name)
            check_disposition(snapshot, disposition)

            path = snapshot / 'tests/semantics.rs'
            edits = disposition['files']['tests/semantics.rs']['successor_edits']
            for edit in reversed(edits):
                current = path.read_text()
                assert current.count(edit['after']) == 1
                path.write_text(current.replace(edit['after'], edit['before'], 1))
                check_disposition(snapshot, disposition)

            path.write_text(path.read_text().replace('m::__model__', '::', 1))
            with self.assertRaises(AssertionError):
                check_disposition(snapshot, disposition)


if __name__ == '__main__':
    unittest.main()
