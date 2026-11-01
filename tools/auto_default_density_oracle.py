#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Reuse the qualified sphere roundoff oracle for both default-density inputs."""
import json
import math
from pathlib import Path

import sphere_inertia_oracle as oracle

ROOT = Path(__file__).resolve().parents[1]


def check(record, rows, radius, count):
    density, actual_radius = map(oracle.value, record['input_bits'])
    assert (density, actual_radius) == (1000., radius)
    first = oracle.mathematical(density, radius, 200)
    second = oracle.mathematical(density, radius, 400)
    with oracle.mp.workdps(400):
        assert [oracle.bits(float(oracle.mp.mpf(x))) for x in first] == [oracle.bits(float(oracle.mp.mpf(x))) for x in second]
        refs = list(map(oracle.mp.mpf, second))
        assert record['bound_factors'] == {'native': [5, 9], 'parry': [8, 11], 'differential': [13, 20]}
        # Qualify the existing normal-intermediate/normal-reciprocal contract.
        a = math.pi * radius
        b = a * radius
        c = b * radius
        d = c * 4.
        volume = d / 3.
        mass = volume * density
        square = radius * radius
        twice = square * 2.
        unit = twice / 5.
        inertia = unit * mass
        native_cube = math.pow(radius, 3)
        native_volume = ((4. / 3.) * math.pi) * native_cube
        assert all(math.isfinite(x) and x >= float.fromhex('0x1p-1022') for x in (
            a, b, c, d, volume, mass, square, twice, unit, inertia,
            native_cube, native_volume, native_volume * density,
            (.4 * (native_volume * density)) * square, 1. / mass, 1. / inertia))
        assert abs(oracle.exact((4. / 3.) * math.pi) - 4 * oracle.mp.pi / 3) / (4 * oracle.mp.pi / 3) <= oracle.mp.mpf(2) ** -53
        assert abs(oracle.exact(.4) - oracle.mp.mpf(2) / 5) / (oracle.mp.mpf(2) / 5) <= oracle.mp.mpf(2) ** -53
        assert abs(oracle.exact(native_cube) - oracle.exact(radius) ** 3) / oracle.exact(radius) ** 3 <= 2 * oracle.mp.mpf(2) ** -53
        for kind in ['native', 'parry']:
            assert record['oracle_intervals'][kind] == [oracle.interval(x, n) for x, n in zip(refs, record['bound_factors'][kind])]
        assert record['differential_absolute_bounds'] == [oracle.bits(math.nextafter(float(x * oracle.gamma(n)), math.inf)) for x, n in zip(refs, record['bound_factors']['differential'])]
        checked = 0
        for row in rows:
            values = list(map(int, row['stdout'].splitlines()[1].split()[1:]))
            assert values[:2] == [1, 1] and values[16:] == [0, 0]
            for i, reference in enumerate(refs):
                assert abs(oracle.exact(oracle.value(values[2 + i])) - reference) / reference <= oracle.gamma(record['bound_factors']['native'][i])
            checked += 1
        assert checked == count


def main():
    assert oracle.mp.__version__ == '1.2.1'
    record = json.loads((ROOT / 'tests/auto_default_density_oracle.json').read_text())
    rows = json.loads((ROOT / 'tests/auto_default_density.json').read_text())['cases']
    check(record, [row for row in rows if row['shape'] == 'sphere' and 'rust_error' not in row], 1., 38)
    rows = json.loads((ROOT / 'tests/auto_sphere_inertial.json').read_text())['cases']
    check(record['dependent_sphere'], [row for row in rows if row['name'] in ['no_density', 'empty_density']], 0.1, 2)
    print('Two fixed-input 200/400-digit oracles and40 native sphere projections passed')


if __name__ == '__main__':
    main()
