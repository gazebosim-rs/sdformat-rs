# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
import unittest

from auto_translation_pose_reference import observation


class TranslationProtocol(unittest.TestCase):
    def setUp(self):
        values = ' '.join(['1', '1'] + ['0'] * 14 + ['0', '0'])
        direct = ' '.join(['0'] * 14)
        self.text = (f'LOAD\nVALUES {values}\nROOT_STATUS 1 0\nCOLLISION_ERRORS\n'
                     f'COLLISION_VALUES {direct}\nCOLLISION_STATUS 1 0\n'
                     f'RESOLVE\nVALUES {values}\nROOT_STATUS 1 0\n')

    def test_full_and_error_vectors_are_retained(self):
        result = observation(self.text.replace('LOAD\n', 'LOAD 19\n'))
        self.assertEqual(result['load_errors'], [19])
        self.assertEqual(len(result['root_values']), 2)
        self.assertEqual(len(result['root_values'][0]), 18)
        self.assertEqual(result['root_status'], [[1, 0], [1, 0]])

    def test_missing_or_duplicate_snapshot_is_rejected(self):
        for value in [self.text.replace('RESOLVE\n', ''), self.text + self.text,
                      self.text.replace('ROOT_STATUS 1 0\n', '', 1)]:
            with self.assertRaises(AssertionError):
                observation(value)

    def test_unsigned_bounds_and_flags_are_rejected(self):
        for value in [self.text.replace('VALUES 1 1 ', 'VALUES 2 1 ', 1),
                      self.text.replace('COLLISION_VALUES 0 ', 'COLLISION_VALUES -1 ', 1),
                      self.text.replace('COLLISION_VALUES 0 ', 'COLLISION_VALUES 18446744073709551616 ', 1)]:
            with self.assertRaises(AssertionError):
                observation(value)

    def test_nonfinite_status_cannot_hide_float_bits(self):
        with self.assertRaises(AssertionError):
            observation(self.text.replace('COLLISION_VALUES 0 ', 'COLLISION_VALUES 9218868437227405312 ', 1))
        with self.assertRaises(AssertionError):
            observation(self.text.replace('ROOT_STATUS 1 0', 'ROOT_STATUS 0 0', 1))


if __name__ == '__main__':
    unittest.main()
