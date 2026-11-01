#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Recheck NEW sphere-case bounds using exact inputs and independent mpmath.

This optional reference tool requires mpmath 1.2.1, not a production dependency.
It never supplies a mass formula to the loader or changes retained tolerances.
"""
import json
import math
from pathlib import Path
import struct

import mpmath as mp

ROOT = Path(__file__).resolve().parents[1]


def bits(value):
    return struct.unpack('<Q', struct.pack('<d', value))[0]


def value(encoded):
    return struct.unpack('<d', struct.pack('<Q', encoded))[0]


def exact(number):
    numerator, denominator = number.as_integer_ratio()
    return mp.mpf(numerator) / denominator


def mathematical(density, radius, digits):
    with mp.workdps(digits):
        mass = 4 * mp.pi * exact(density) * exact(radius) ** 3 / 3
        inertia = 2 * mass * exact(radius) ** 2 / 5
        return [mp.nstr(x, digits - 10) for x in (mass, inertia)]


def gamma(count):
    unit = mp.mpf(2) ** -53
    return count * unit / (1 - count * unit)


def interval(number, count):
    # Round outward once more so binary endpoints enclose the real interval.
    return [bits(math.nextafter(float(number * (1 - gamma(count))), -math.inf)),
            bits(math.nextafter(float(number * (1 + gamma(count))), math.inf))]


def main():
    assert mp.__version__ == '1.2.1', 'requalify the reference implementation version'
    rows = json.loads((ROOT / 'tests/auto_sphere_inertial.json').read_text())['cases']
    checked = 0
    for row in rows:
        if 'oracle_intervals' not in row:
            continue
        density, radius = map(value, row['oracle_input_bits'])
        refs200 = mathematical(density, radius, 200)
        refs400 = mathematical(density, radius, 400)
        with mp.workdps(400):
            assert [bits(float(mp.mpf(x))) for x in refs200] == [bits(float(mp.mpf(x))) for x in refs400]
            refs = list(map(mp.mpf, refs200))
            # Classification follows inspected operations, not a geometry cutoff.
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
            normal = lambda x: math.isfinite(x) and x >= float.fromhex('0x1p-1022')
            assert all(normal(x) for x in (a, b, c, d, volume, mass, square, twice, unit, inertia))
            native_cube = math.pow(radius, 3)
            native_volume = ((4. / 3.) * math.pi) * native_cube
            assert all(normal(x) for x in (native_cube, native_volume, native_volume * density, (.4 * (native_volume * density)) * square))
            assert abs(exact((4. / 3.) * math.pi) - 4 * mp.pi / 3) / (4 * mp.pi / 3) <= mp.mpf(2) ** -53
            assert abs(exact(.4) - mp.mpf(2) / 5) / (mp.mpf(2) / 5) <= mp.mpf(2) ** -53
            assert abs(exact(native_cube) - exact(radius) ** 3) / exact(radius) ** 3 <= 2 * mp.mpf(2) ** -53
            assert row['bound_factors']['native'] == [5, 9]
            # A subnormal stored reciprocal costs at most four units, followed
            # by one normal extraction division, rather than two normal units.
            assert row['bound_factors']['parry'] == [8 if normal(1. / mass) else 11, 11 if normal(1. / inertia) else 14]
            factors = row['bound_factors']
            assert factors['differential'] == [a + b for a, b in zip(factors['native'], factors['parry'])]
            for kind in ('native', 'parry'):
                assert row['oracle_intervals'][kind] == [interval(x, n) for x, n in zip(refs, factors[kind])], row['name']
            assert row['differential_absolute_bounds'] == [bits(math.nextafter(float(x * gamma(n)), math.inf)) for x, n in zip(refs, factors['differential'])]
            native = list(map(int, row['stdout'].splitlines()[1].split()[1:]))
            for i, reference in enumerate(refs):
                assert abs(exact(value(native[2 + i])) - reference) / reference <= gamma(factors['native'][i]), row['name']
        checked += 1
    print(f'{checked} exact-input, 200/400-digit sphere reference bounds passed')


if __name__ == '__main__':
    main()
