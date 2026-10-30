# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Compiler-backed declaration evidence and measured coverage precedence."""
import copy
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
import verify

ROOT = Path(__file__).resolve().parents[1]

class ItemProofEndToEndTests(unittest.TestCase):
    def test_real_helper_and_llvm_measurements_remain_authoritative(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); (root / 'src').mkdir()
            target = root / 'cargo-target'
            build = subprocess.run(['cargo', 'test', '--offline', '--locked', '--test', 'coverage_syntax', '--no-run', '--message-format=json'],
                                   cwd=ROOT, env=dict(os.environ, CARGO_TARGET_DIR=str(target)), capture_output=True, text=True, check=True)
            binaries = [json.loads(line)['executable'] for line in build.stdout.split('\n') if line.startswith('{')
                        and json.loads(line).get('reason') == 'compiler-artifact'
                        and json.loads(line)['target']['name'] == 'coverage_syntax'
                        and json.loads(line).get('executable')]
            self.assertEqual(len(binaries), 1)
            source = ('use std::{\n    sync::OnceLock,\n};\n'
                      'static GOOD: OnceLock<u8> = OnceLock::new();\n'
                      'const fn value() -> [u8; 1] { [0] }\n'
                      'static BAD: [u8; { #[unsafe(no_mangle)] pub extern "C" fn hidden() { std::hint::black_box(42); } 1 }] = value();\n'
                      'fn main() { std::hint::black_box(&GOOD); std::hint::black_box(BAD); }\n')
            source_path = root / 'src/parse.rs'; source_path.write_text(source)
            ancestor = 'mod parse;'; helper = (ROOT / 'tests/coverage_syntax.rs').read_text()
            query = subprocess.run([binaries[0], '--exact', 'syntax_query', '--nocapture', '--color', 'never'],
                    input=json.dumps({'source': source, 'ancestor': ancestor, 'module': 'parse'}), capture_output=True,
                    text=True, check=True, cwd=ROOT, env=dict(os.environ, SDF_COVERAGE_SYNTAX_QUERY='1', RUST_TEST_THREADS='2'))
            self.assertEqual(query.stderr, '')
            evidence = verify.parse_syntax_output(query.stdout, source, helper, ancestor, 'parse')
            self.assertTrue({1, 2, 3, 4}.issubset(evidence['items']))
            self.assertNotIn(6, evidence['items'])
            binary = root / 'program'
            subprocess.run(['rustc', '--edition=2024', '-C', 'instrument-coverage', str(source_path), '-o', str(binary)], check=True, capture_output=True)
            profraw = root / 'data.profraw'
            subprocess.run([str(binary)], check=True, env=dict(os.environ, LLVM_PROFILE_FILE=str(profraw)))
            sysroot = Path(subprocess.check_output(['rustc', '--print', 'sysroot'], text=True).strip())
            host = next(line.split(': ', 1)[1] for line in subprocess.check_output(['rustc', '-vV'], text=True).splitlines() if line.startswith('host: '))
            llvm = sysroot / 'lib/rustlib' / host / 'bin'; profdata = root / 'data.profdata'
            subprocess.run([str(llvm / 'llvm-profdata'), 'merge', '-sparse', str(profraw), '-o', str(profdata)], check=True)
            args = [str(llvm / 'llvm-cov'), 'export', str(binary), f'-instr-profile={profdata}']
            lcov = subprocess.check_output(args + ['-format=lcov'], text=True)
            covered = verify.lcov_records(lcov)
            self.assertIs(covered[(source_path.resolve(), 6)], False)
            changed = {'lines': {('src/parse.rs', 4), ('src/parse.rs', 6)}}
            forged = dict(evidence, items=evidence['items'] | {6})
            with self.assertRaisesRegex(AssertionError, 'changed executable'):
                verify.check_changed_coverage(root, changed, covered, syntax_provider=lambda *_: forged)
            report = json.loads(subprocess.check_output(args, text=True))
            file = next(f for item in report['data'] for f in item['files'] if Path(f['filename']).resolve() == source_path.resolve())
            self.assertLess(file['summary']['regions']['covered'] * 100, file['summary']['regions']['count'] * 95)
            isolated = copy.deepcopy(report)
            for item in isolated['data']:
                for entry in item['files']:
                    if Path(entry['filename']).resolve() == source_path.resolve():
                        entry['summary']['lines'] = {'count': 100, 'covered': 100}
            manifest = {'profiles': ['core'], 'crates': {'sdformat': ['src']}, 'exclusions': []}
            with self.assertRaisesRegex(AssertionError, 'regions:'):
                verify.check_report(isolated, root, manifest)

if __name__ == '__main__':
    unittest.main()

class CustomMetadataEndToEndTests(unittest.TestCase):
    def test_real_module_field_and_else_proof_keeps_uncovered_branch_measured(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); (root / 'src').mkdir()
            target = root / 'cargo-target'
            build = subprocess.run(['cargo', 'test', '--offline', '--locked', '--test', 'coverage_syntax', '--no-run', '--message-format=json'],
                    cwd=ROOT, env=dict(os.environ, CARGO_TARGET_DIR=str(target)), capture_output=True, text=True, check=True)
            binaries = [json.loads(line)['executable'] for line in build.stdout.split('\n') if line.startswith('{')
                    and json.loads(line).get('reason') == 'compiler-artifact' and json.loads(line)['target']['name'] == 'coverage_syntax'
                    and json.loads(line).get('executable')]
            self.assertEqual(len(binaries), 1)
            source = ('#[derive(Debug, Default, Clone, PartialEq)]\n'
                    'pub struct Metadata {\n pub raw: String,\n pub subtype: Option<String>,\n}\n'
                    'pub fn run() {\n let number = if std::hint::black_box(true) {\n  std::hint::black_box(1)\n } else {\n  std::hint::black_box(2)\n };\n'
                    ' std::hint::black_box(number);\n std::hint::black_box(Metadata::default());\n}\n')
            ancestor = 'mod parse;\nfn main() { parse::run(); }\n'
            path = root / 'src/parse.rs'; path.write_text(source)
            (root / 'src/main.rs').write_text(ancestor)
            query = subprocess.run([binaries[0], '--exact', 'syntax_query', '--nocapture', '--color', 'never'],
                    input=json.dumps({'source': source, 'ancestor': ancestor, 'module': 'parse'}), capture_output=True, text=True, check=True,
                    env=dict(os.environ, SDF_COVERAGE_SYNTAX_QUERY='1', RUST_TEST_THREADS='2'), cwd=ROOT)
            self.assertEqual(query.stderr, '')
            evidence = verify.parse_syntax_output(query.stdout, source, (ROOT / 'tests/coverage_syntax.rs').read_text(), ancestor, 'parse')
            self.assertEqual(evidence['fields'], {3, 4})
            self.assertEqual(evidence['else_delimiters'], {9})
            binary = root / 'program'
            subprocess.run(['rustc', '--edition=2024', '-C', 'instrument-coverage', str(root / 'src/main.rs'), '-o', str(binary)], check=True, capture_output=True)
            profraw = root / 'data.profraw'
            subprocess.run([str(binary)], check=True, env=dict(os.environ, LLVM_PROFILE_FILE=str(profraw)))
            sysroot = Path(subprocess.check_output(['rustc', '--print', 'sysroot'], text=True).strip())
            host = next(line.split(': ', 1)[1] for line in subprocess.check_output(['rustc', '-vV'], text=True).splitlines() if line.startswith('host: '))
            llvm = sysroot / 'lib/rustlib' / host / 'bin'; profdata = root / 'data.profdata'
            subprocess.run([str(llvm / 'llvm-profdata'), 'merge', '-sparse', str(profraw), '-o', str(profdata)], check=True)
            lcov = subprocess.check_output([str(llvm / 'llvm-cov'), 'export', str(binary), f'-instr-profile={profdata}', '-format=lcov'], text=True)
            covered = verify.lcov_records(lcov)
            self.assertIs(covered[(path.resolve(), 10)], False)
            self.assertNotIn((path.resolve(), 9), covered)
            changed = {'lines': {('src/parse.rs', line) for line in (3, 4, 9)}}
            measured = verify.check_changed_coverage(root, changed, covered, syntax_provider=lambda *_: evidence)
            self.assertTrue(all(measured))
            for key in ('fields', 'else_delimiters', 'items'):
                forged = dict(evidence, **{key: evidence[key] | {10}})
                with self.assertRaisesRegex(AssertionError, 'changed executable'):
                    verify.check_changed_coverage(root, {'lines': changed['lines'] | {('src/parse.rs', 10)}}, covered, syntax_provider=lambda *_: forged)
