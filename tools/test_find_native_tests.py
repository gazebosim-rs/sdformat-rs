# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0

import json
from pathlib import Path
import unittest

from find_native_tests import find_cases


class NativeSourceLookupTests(unittest.TestCase):
    def test_case_insensitive_airspeed_source_in_pinned_inventory(self):
        inventory = json.loads(Path('reference/inventory.json').read_text())
        matches = find_cases(inventory, 'AirSpeed_TEST.cc')
        self.assertEqual(
            [case for source, target, case in matches if target == 'UNIT_Airspeed_TEST'],
            [
                'DOMAirSpeed.Construction',
                'DOMAirSpeed.Load',
                'DOMAirSpeed.Set',
                'DOMAirSpeed.ToElement',
            ],
        )
        self.assertTrue(all(source == 'src/Airspeed_TEST.cc' for source, _, _ in matches))

    def test_absent_source_is_empty(self):
        self.assertEqual(find_cases({'targets': []}, 'missing_TEST.cc'), [])
