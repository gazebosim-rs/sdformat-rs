# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
import unittest
from auto_rotation_pose_reference import observation


class RotationProtocol(unittest.TestCase):
    def setUp(self):
        values = ' '.join(['1', '1'] + ['0'] * 14 + ['0', '0'])
        direct = ' '.join(['0'] * 14)
        matrix = ' '.join(['0'] * 9)
        self.text = (f'LOAD\nVALUES {values}\nROOT_STATUS 1 0\nCOLLISION_ERRORS\n'
                     f'COLLISION_VALUES {direct}\nCOLLISION_STATUS 1 0\n'
                     f'COLLISION_MOI {matrix}\nRESOLVE\nVALUES {values}\nROOT_STATUS 1 0\n')

    def test_complete_direct_matrix_and_root_snapshots_are_retained(self):
        value = observation(self.text)
        self.assertEqual(len(value['collision_moi']), 9)
        self.assertEqual(len(value['root_values']), 2)
        self.assertEqual(len(value['collision_values']), 14)

    def test_missing_duplicate_short_or_out_of_range_matrix_is_refused(self):
        for value in [self.text.replace('COLLISION_MOI ', 'MISSING '),
                      self.text + 'COLLISION_MOI ' + ' '.join(['0'] * 9) + '\n',
                      self.text.replace('COLLISION_MOI 0 ', 'COLLISION_MOI ', 1),
                      self.text.replace('COLLISION_MOI 0 ', 'COLLISION_MOI -1 ', 1),
                      self.text.replace('COLLISION_MOI 0 ', 'COLLISION_MOI 18446744073709551616 ', 1)]:
            with self.assertRaises(AssertionError):
                observation(value)
