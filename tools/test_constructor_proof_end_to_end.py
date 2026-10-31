# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Actual intrinsic-prefix proof and real LLVM uncovered-code precedence."""
import copy
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
import verify

ROOT = Path(__file__).resolve().parents[1]

class ConstructorProofEndToEndTests(unittest.TestCase):
    def test_real_compiled_context_preserves_uncovered_da_and_region_failures(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); (root / 'src').mkdir()
            build = subprocess.run(['cargo', 'test', '--offline', '--locked', '--test', 'coverage_syntax', '--no-run', '--message-format=json'],
                    cwd=ROOT, env=dict(os.environ, CARGO_TARGET_DIR=str(root / 'cargo-target')), capture_output=True, text=True, check=True)
            binaries = [json.loads(line)['executable'] for line in build.stdout.split('\n') if line.startswith('{')
                    and json.loads(line).get('reason') == 'compiler-artifact' and json.loads(line)['target']['name'] == 'coverage_syntax'
                    and json.loads(line).get('executable')]
            self.assertEqual(len(binaries), 1)
            source = ('use crate::*;\npub fn run() -> Result<Scene, ()> {\n Ok(Scene {\n'
                      '  value: if std::hint::black_box(true) {\n   std::hint::black_box(1)\n  } else {\n'
                      '   std::hint::black_box(2)\n  },\n })\n}\n')
            ancestor = 'mod types; mod camera; mod parse; pub use types::*; pub use camera::*; fn main() { let _ = parse::run(); }\n'
            exports = {'types': 'pub struct Scene { pub value: u8 }\n', 'camera': 'pub struct Camera;\n'}
            for name, text in dict(exports, parse=source, main=ancestor).items():
                (root / 'src' / (name + '.rs')).write_text(text)
            query = subprocess.run([binaries[0], '--exact', 'syntax_query', '--nocapture', '--color', 'never'],
                    input=json.dumps(dict(source=source, ancestor=ancestor, module='parse', exports=exports)), capture_output=True,
                    text=True, check=True, cwd=ROOT, env=dict(os.environ, SDF_COVERAGE_SYNTAX_QUERY='1', RUST_TEST_THREADS='2'))
            self.assertEqual(query.stderr, '')
            evidence = verify.parse_syntax_output(query.stdout, source, (ROOT / 'tests/coverage_syntax.rs').read_text(), ancestor, 'parse', exports)
            self.assertEqual(evidence['aggregate_prefixes'], {3})
            changed_exports = dict(exports, types=exports['types'] + '// changed\n')
            with self.assertRaisesRegex(AssertionError, 'export context'):
                verify.parse_syntax_output(query.stdout, source, (ROOT / 'tests/coverage_syntax.rs').read_text(), ancestor, 'parse', changed_exports)
            binary = root / 'program'
            subprocess.run(['rustc', '--edition=2024', '-C', 'instrument-coverage', str(root / 'src/main.rs'), '-o', str(binary)], check=True, capture_output=True)
            profraw = root / 'data.profraw'
            subprocess.run([str(binary)], check=True, env=dict(os.environ, LLVM_PROFILE_FILE=str(profraw)))
            sysroot = Path(subprocess.check_output(['rustc', '--print', 'sysroot'], text=True).strip())
            host = next(line.split(': ', 1)[1] for line in subprocess.check_output(['rustc', '-vV'], text=True).split('\n') if line.startswith('host: '))
            llvm = sysroot / 'lib/rustlib' / host / 'bin'; merged = root / 'data.profdata'
            subprocess.run([str(llvm / 'llvm-profdata'), 'merge', '-sparse', str(profraw), '-o', str(merged)], check=True)
            args = [str(llvm / 'llvm-cov'), 'export', '-instr-profile=' + str(merged), str(binary)]
            report = json.loads(subprocess.check_output(args, text=True))
            lcov = subprocess.check_output(args + ['-format=lcov'], text=True)
            covered = verify.lcov_records(lcov); file = (root / 'src/parse.rs').resolve()
            self.assertFalse(covered[file, 7])
            self.assertEqual(verify.check_changed_coverage(root, {'lines': {('src/parse.rs', 3)}}, covered, syntax_provider=lambda *_: evidence), [])
            forged = copy.deepcopy(evidence); forged['aggregate_prefixes'].add(7)
            with self.assertRaisesRegex(AssertionError, 'changed executable'):
                verify.check_changed_coverage(root, {'lines': {('src/parse.rs', 7)}}, covered, syntax_provider=lambda *_: forged)
            regions = {'count': 0, 'covered': 0}
            for data in report['data']:
                for item in data['files']:
                    if Path(item['filename']).is_relative_to(root / 'src'):
                        for key in regions:
                            regions[key] += item['summary']['regions'][key]
            self.assertLessEqual(regions['covered'] * 100, regions['count'] * 95)
            with self.assertRaisesRegex(AssertionError, 'regions'):
                verify.check_counts({'lines': {'count': 1, 'covered': 1}, 'regions': regions})
