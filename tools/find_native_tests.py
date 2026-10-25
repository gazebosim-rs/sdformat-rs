#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0

"""Find pinned native test cases by source name, ignoring filename case."""

import argparse
import json
from pathlib import Path


def find_cases(inventory, source_name):
    query = Path(source_name).name.casefold()
    matches = []
    for target in inventory['targets']:
        for case in target['cases']:
            for source in case['source']:
                if Path(source['path']).name.casefold() == query:
                    matches.append((source['path'], target['target'], case['id']))
    return sorted(set(matches))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source_name', help='native source filename, case insensitive')
    parser.add_argument('--inventory', type=Path, default=Path('reference/inventory.json'))
    args = parser.parse_args()
    matches = find_cases(json.loads(args.inventory.read_text()), args.source_name)
    for source, target, case in matches:
        print(f'{source}\t{target}\t{case}')
    if not matches:
        parser.exit(1, f'No pinned native cases for {args.source_name}\n')


if __name__ == '__main__':
    main()
