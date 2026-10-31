# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Real LLVM controls for the URDF slice's absent-DA syntax forms."""
import copy
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
import verify

ROOT = Path(__file__).resolve().parents[1]


class UrdfSyntaxEndToEndTests(unittest.TestCase):
    def test_real_llvm_preserves_runtime_drop_and_missing_source_failures(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / 'src').mkdir()
            env = dict(os.environ, CARGO_BUILD_JOBS='2', CARGO_TARGET_DIR=str(root / 'target'))
            build = subprocess.run(['cargo', 'test', '--offline', '--locked', '--test', 'coverage_syntax', '--no-run', '--message-format=json'],
                                   cwd=ROOT, env=env, capture_output=True, text=True, check=True)
            artifacts = [json.loads(line) for line in build.stdout.splitlines() if line.startswith('{')]
            binaries = [item['executable'] for item in artifacts if item.get('reason') == 'compiler-artifact'
                        and item['target']['name'] == 'coverage_syntax' and item.get('executable')]
            self.assertEqual(len(binaries), 1)
            source = ('#[derive(Default)]\npub struct Value {\n pub value: u8,\n hidden: Option<Guard>,\n}\n'
                      'struct Guard;\nimpl Drop for Guard {\n fn drop(&mut self) {\n'
                      '  if std::hint::black_box(true) {\n   std::hint::black_box(1);\n  } else {\n'
                      '   std::hint::black_box(2);\n  }\n }\n}\n'
                      'fn wrap(v: u8) -> u8 { v }\npub fn run() {\n'
                      ' let mut base = Value::default();\n base.hidden = Some(Guard);\n let copy = Value {\n  value: 2,\n  ..base\n };\n'
                      ' let _ = format!(\n  "<value>{}</value>",\n  wrap(\n'
                      '   if std::hint::black_box(true) {\n    std::hint::black_box(copy.value)\n'
                      '   } else {\n    std::hint::black_box(9)\n   }\n  )\n );\n'
                      ' std::hint::black_box(copy.hidden);\n}\n')
            ancestor = 'mod body;\npub use body::Value;\nfn main() { body::run(); }\n'
            (root / 'src/body.rs').write_text(source)
            (root / 'src/main.rs').write_text(ancestor)

            def query(text, module):
                result = subprocess.run([binaries[0], '--exact', 'syntax_query', '--nocapture', '--color', 'never'],
                                        input=json.dumps(dict(source=text, ancestor=ancestor, module=module, exports={})),
                                        capture_output=True, text=True, check=True, cwd=ROOT,
                                        env=dict(os.environ, SDF_COVERAGE_SYNTAX_QUERY='1'))
                self.assertEqual(result.stderr, '')
                return verify.parse_syntax_output(result.stdout, text, (ROOT / 'tests/coverage_syntax.rs').read_text(), ancestor, module, {})

            proof = query(source, 'body')
            root_proof = query(ancestor, '')
            binary = root / 'program'
            subprocess.run(['rustc', '--edition=2024', '-C', 'instrument-coverage', str(root / 'src/main.rs'), '-o', str(binary)],
                           check=True, capture_output=True)
            raw = root / 'data.profraw'
            subprocess.run([str(binary)], check=True, env=dict(os.environ, LLVM_PROFILE_FILE=str(raw)))
            sysroot = Path(subprocess.check_output(['rustc', '--print', 'sysroot'], text=True).strip())
            host = next(line.split(': ', 1)[1] for line in subprocess.check_output(['rustc', '-vV'], text=True).splitlines() if line.startswith('host: '))
            llvm = sysroot / 'lib/rustlib' / host / 'bin'
            merged = root / 'data.profdata'
            subprocess.run([str(llvm / 'llvm-profdata'), 'merge', '-sparse', str(raw), '-o', str(merged)], check=True)
            args = [str(llvm / 'llvm-cov'), 'export', '-instr-profile=' + str(merged), str(binary)]
            report = json.loads(subprocess.check_output(args, text=True))
            covered = verify.lcov_records(subprocess.check_output(args + ['-format=lcov'], text=True))
            file = (root / 'src/body.rs').resolve()
            lines = source.splitlines()
            # Bind to actual template/rest/header syntax, not a fabricated DA map.
            syntax_lines = [lines.index(text) + 1 for text in ('pub struct Value {', ' hidden: Option<Guard>,',
                            'struct Guard;', '  ..base', '  "<value>{}</value>",', '  )', ' );')]
            # Unit structs are outside this bounded extension.
            syntax_lines.remove(lines.index('struct Guard;') + 1)
            self.assertNotIn((file, lines.index('  "<value>{}</value>",') + 1), covered)
            for line in syntax_lines:
                result = verify.check_changed_coverage(root, {'lines': {('src/body.rs', line)}}, covered, syntax_provider=lambda *_: proof)
                self.assertEqual(result, [True] if (file, line) in covered else [])
            for line in (1, 2):
                verify.check_changed_coverage(root, {'lines': {('src/main.rs', line)}}, covered, syntax_provider=lambda *_: root_proof)
            self.assertTrue(covered[file, lines.index('   std::hint::black_box(1);') + 1])
            for text in ('   std::hint::black_box(2);', '    std::hint::black_box(9)'):
                line = lines.index(text) + 1
                self.assertFalse(covered[file, line])
                forged = copy.deepcopy(proof)
                forged['items'].add(line)
                forged['opaque'].discard(line)
                with self.assertRaisesRegex(AssertionError, 'changed executable'):
                    verify.check_changed_coverage(root, {'lines': {('src/body.rs', line)}}, covered, syntax_provider=lambda *_: forged)
            absent = {key: value for key, value in covered.items() if key[0] != file}
            with self.assertRaisesRegex(AssertionError, 'no instrumentation'):
                verify.check_changed_coverage(root, {'lines': {('src/body.rs', syntax_lines[-1])}}, absent, syntax_provider=lambda *_: proof)
            missing = copy.deepcopy(report)
            for data in missing['data']:
                data['files'] = [item for item in data['files'] if Path(item['filename']).resolve() != file]
            with self.assertRaisesRegex(AssertionError, 'production files'):
                verify.check_report(missing, root, json.loads((ROOT / 'coverage.json').read_text()))
            regions = {'count': 0, 'covered': 0}
            for data in report['data']:
                for item in data['files']:
                    if Path(item['filename']).is_relative_to(root / 'src'):
                        for key in regions:
                            regions[key] += item['summary']['regions'][key]
            with self.assertRaisesRegex(AssertionError, 'regions'):
                verify.check_counts({'lines': {'count': 1, 'covered': 1}, 'regions': regions})

    def test_tuple_and_for_array_proof_preserves_actual_runtime_and_drop_records(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / 'src').mkdir()
            build = subprocess.run(['cargo', 'test', '--offline', '--locked', '--test', 'coverage_syntax', '--no-run', '--message-format=json'],
                                   cwd=ROOT, env=dict(os.environ, CARGO_BUILD_JOBS='2', CARGO_TARGET_DIR=str(root / 'target')),
                                   capture_output=True, text=True, check=True)
            artifacts = [json.loads(line) for line in build.stdout.splitlines() if line.startswith('{')]
            binaries = [item['executable'] for item in artifacts if item.get('reason') == 'compiler-artifact'
                        and item['target']['name'] == 'coverage_syntax' and item.get('executable')]
            self.assertEqual(len(binaries), 1)
            source = ('struct Guard;\nimpl Drop for Guard {\n fn drop(&mut self) {\n'
                      '  if std::hint::black_box(true) {\n   std::hint::black_box(3);\n'
                      '  } else {\n   std::hint::black_box(4);\n  }\n }\n}\n'
                      'pub fn run() {\n for (guard, active) in [\n (Guard, true),\n (\n'
                      ' Guard,\n if std::hint::black_box(true) {\n  std::hint::black_box(true)\n'
                      ' } else {\n  std::hint::black_box(false)\n },\n ),\n ] {\n'
                      '  std::hint::black_box(&guard);\n  if active {\n   std::hint::black_box(1);\n'
                      '  } else {\n   std::hint::black_box(2);\n  }\n }\n}\n')
            ancestor = 'mod body;\nfn main() { body::run(); }\n'
            (root / 'src/body.rs').write_text(source)
            (root / 'src/main.rs').write_text(ancestor)
            result = subprocess.run([binaries[0], '--exact', 'syntax_query', '--nocapture', '--color', 'never'],
                                    input=json.dumps(dict(source=source, ancestor=ancestor, module='body', exports={})),
                                    capture_output=True, text=True, check=True, cwd=ROOT,
                                    env=dict(os.environ, SDF_COVERAGE_SYNTAX_QUERY='1'))
            self.assertEqual(result.stderr, '')
            proof = verify.parse_syntax_output(result.stdout, source, (ROOT / 'tests/coverage_syntax.rs').read_text(), ancestor, 'body', {})
            binary = root / 'program'
            subprocess.run(['rustc', '--edition=2024', '-C', 'instrument-coverage', str(root / 'src/main.rs'), '-o', str(binary)],
                           check=True, capture_output=True)
            raw = root / 'data.profraw'
            subprocess.run([str(binary)], check=True, env=dict(os.environ, LLVM_PROFILE_FILE=str(raw)))
            sysroot = Path(subprocess.check_output(['rustc', '--print', 'sysroot'], text=True).strip())
            host = next(line.split(': ', 1)[1] for line in subprocess.check_output(['rustc', '-vV'], text=True).splitlines() if line.startswith('host: '))
            llvm = sysroot / 'lib/rustlib' / host / 'bin'
            merged = root / 'data.profdata'
            subprocess.run([str(llvm / 'llvm-profdata'), 'merge', '-sparse', str(raw), '-o', str(merged)], check=True)
            args = [str(llvm / 'llvm-cov'), 'export', '-instr-profile=' + str(merged), str(binary)]
            report = json.loads(subprocess.check_output(args, text=True))
            covered = verify.lcov_records(subprocess.check_output(args + ['-format=lcov'], text=True))
            file = (root / 'src/body.rs').resolve()
            lines = source.splitlines()
            for text, field in [(' (', 'items'), (' ),', 'closings'), (' ] {', 'items')]:
                line = lines.index(text) + 1
                self.assertIn(line, proof[field])
                self.assertNotIn((file, line), covered)
                self.assertEqual(verify.check_changed_coverage(root, {'lines': {('src/body.rs', line)}}, covered, syntax_provider=lambda *_: proof), [])
            self.assertTrue(covered[file, lines.index('   std::hint::black_box(3);') + 1])
            for text in ('   std::hint::black_box(4);', '  std::hint::black_box(false)', '   std::hint::black_box(2);'):
                line = lines.index(text) + 1
                self.assertFalse(covered[file, line])
                forged = copy.deepcopy(proof)
                forged['items'].add(line)
                forged['opaque'].discard(line)
                with self.assertRaisesRegex(AssertionError, 'changed executable'):
                    verify.check_changed_coverage(root, {'lines': {('src/body.rs', line)}}, covered, syntax_provider=lambda *_: forged)
            absent = {key: value for key, value in covered.items() if key[0] != file}
            with self.assertRaisesRegex(AssertionError, 'no instrumentation'):
                verify.check_changed_coverage(root, {'lines': {('src/body.rs', lines.index(' ] {') + 1)}}, absent, syntax_provider=lambda *_: proof)
            missing = copy.deepcopy(report)
            for data in missing['data']:
                data['files'] = [item for item in data['files'] if Path(item['filename']).resolve() != file]
            with self.assertRaisesRegex(AssertionError, 'production files'):
                verify.check_report(missing, root, json.loads((ROOT / 'coverage.json').read_text()))
            regions = {'count': 0, 'covered': 0}
            for data in report['data']:
                for item in data['files']:
                    if Path(item['filename']).is_relative_to(root / 'src'):
                        for key in regions:
                            regions[key] += item['summary']['regions'][key]
            with self.assertRaisesRegex(AssertionError, 'regions'):
                verify.check_counts({'lines': {'count': 1, 'covered': 1}, 'regions': regions})

    def test_local_result_tuple_and_array_keep_real_runtime_drop_and_missing_file_records(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / 'src').mkdir()
            build = subprocess.run(['cargo', 'test', '--offline', '--locked', '--test', 'coverage_syntax', '--no-run', '--message-format=json'],
                                   cwd=ROOT, env=dict(os.environ, CARGO_BUILD_JOBS='2', CARGO_TARGET_DIR=str(root / 'target')),
                                   capture_output=True, text=True, check=True)
            artifacts = [json.loads(line) for line in build.stdout.splitlines() if line.startswith('{')]
            helper = next(item['executable'] for item in artifacts if item.get('reason') == 'compiler-artifact'
                          and item['target']['name'] == 'coverage_syntax' and item.get('executable'))
            source = ('struct Articulation { parent: String, child: String, extension_link: String, limit: bool }\n'
                      'fn required(value: &str) -> Result<&str, ()> { Ok(value) }\n'
                      'fn value() -> Result<(Articulation, u8), ()> {\n let names = [\n'
                      ' required("first")?,\n required("second")?,\n ];\n let extensions = 3;\n Ok((\n Articulation {\n'
                      ' parent: names[0].into(),\n child: if std::hint::black_box(true) {\n String::new()\n'
                      ' } else {\n std::hint::black_box(String::from("uncovered"))\n },\n'
                      ' extension_link: names[1].into(),\n limit: false,\n },\n extensions,\n ))\n}\n'
                      'fn owned_array() {\n let values = [\n String::new(),\n String::new(),\n  ];\n std::hint::black_box(values);\n}\n'
                      'pub fn run() {\n owned_array();\n let guard = crate::Guard;\n let (value, _) = value().unwrap();\n'
                      ' std::hint::black_box((value.parent, value.child, value.extension_link, value.limit));\n'
                      ' std::hint::black_box(&guard);\n}\n')
            ancestor = ('mod urdf;\nstruct Guard;\nimpl Drop for Guard {\n fn drop(&mut self) {\n'
                        '  if std::hint::black_box(true) {\n   std::hint::black_box(1);\n'
                        '  } else {\n   std::hint::black_box(2);\n  }\n }\n}\n'
                        'fn main() { urdf::run(); }\n')
            (root / 'src/urdf.rs').write_text(source)
            (root / 'src/main.rs').write_text(ancestor)
            result = subprocess.run([helper, '--exact', 'syntax_query', '--nocapture', '--color', 'never'],
                                    input=json.dumps(dict(source=source, ancestor=ancestor, module='urdf', exports={})),
                                    capture_output=True, text=True, check=True, cwd=ROOT,
                                    env=dict(os.environ, SDF_COVERAGE_SYNTAX_QUERY='1'))
            self.assertEqual(result.stderr, '')
            proof = verify.parse_syntax_output(result.stdout, source, (ROOT / 'tests/coverage_syntax.rs').read_text(), ancestor, 'urdf', {})
            binary = root / 'program'
            subprocess.run(['rustc', '--edition=2024', '-C', 'instrument-coverage', str(root / 'src/main.rs'), '-o', str(binary)],
                           check=True, capture_output=True)
            raw = root / 'data.profraw'
            subprocess.run([str(binary)], check=True, env=dict(os.environ, LLVM_PROFILE_FILE=str(raw)))
            sysroot = Path(subprocess.check_output(['rustc', '--print', 'sysroot'], text=True).strip())
            host = next(line.split(': ', 1)[1] for line in subprocess.check_output(['rustc', '-vV'], text=True).splitlines() if line.startswith('host: '))
            llvm = sysroot / 'lib/rustlib' / host / 'bin'
            merged = root / 'data.profdata'
            subprocess.run([str(llvm / 'llvm-profdata'), 'merge', '-sparse', str(raw), '-o', str(merged)], check=True)
            args = [str(llvm / 'llvm-cov'), 'export', '-instr-profile=' + str(merged), str(binary)]
            report = json.loads(subprocess.check_output(args, text=True))
            covered = verify.lcov_records(subprocess.check_output(args + ['-format=lcov'], text=True))
            file = (root / 'src/urdf.rs').resolve()
            lines = source.splitlines()
            for text in (' ];', ' Ok((', ' Articulation {', ' },', ' ))'):
                line = len(lines) - lines[::-1].index(text)
                self.assertIn(line, proof['items'])
                self.assertNotIn((file, line), covered)
                self.assertEqual(verify.check_changed_coverage(root, {'lines': {('src/urdf.rs', line)}}, covered, syntax_provider=lambda *_: proof), [])
                self.assertFalse(any(region[0] <= line <= region[2]
                                     for data in report['data'] for function in data['functions']
                                     for region in function['regions']
                                     if Path(function['filenames'][region[5]]).resolve() == file))
            # Owned array elements can give this same punctuation a real DA.
            # Its authoritative measurement always precedes the syntax hint.
            owned_close = lines.index('  ];') + 1
            self.assertIn(owned_close, proof['items'])
            self.assertTrue(covered[file, owned_close])
            self.assertEqual(verify.check_changed_coverage(root, {'lines': {('src/urdf.rs', owned_close)}}, covered, syntax_provider=lambda *_: proof), [True])
            uncovered_owned = dict(covered)
            uncovered_owned[file, owned_close] = False
            with self.assertRaisesRegex(AssertionError, 'changed executable'):
                verify.check_changed_coverage(root, {'lines': {('src/urdf.rs', owned_close)}}, uncovered_owned, syntax_provider=lambda *_: proof)
            drop_file = (root / 'src/main.rs').resolve()
            self.assertTrue(covered[drop_file, ancestor.splitlines().index('   std::hint::black_box(1);') + 1])
            for runtime_file, text, runtime_lines in [(drop_file, '   std::hint::black_box(2);', ancestor.splitlines()),
                                                       (file, ' std::hint::black_box(String::from("uncovered"))', lines)]:
                line = runtime_lines.index(text) + 1
                self.assertFalse(covered[runtime_file, line])
                forged = copy.deepcopy(proof)
                forged['items'].add(line)
                forged['opaque'].discard(line)
                with self.assertRaisesRegex(AssertionError, 'changed executable'):
                    verify.check_changed_coverage(root, {'lines': {(str(runtime_file.relative_to(root)), line)}}, covered, syntax_provider=lambda *_: forged)
            absent = {key: value for key, value in covered.items() if key[0] != file}
            with self.assertRaisesRegex(AssertionError, 'no instrumentation'):
                verify.check_changed_coverage(root, {'lines': {('src/urdf.rs', lines.index(' ))') + 1)}}, absent, syntax_provider=lambda *_: proof)
            missing = copy.deepcopy(report)
            for data in missing['data']:
                data['files'] = [item for item in data['files'] if Path(item['filename']).resolve() != file]
            with self.assertRaisesRegex(AssertionError, 'production files'):
                verify.check_report(missing, root, json.loads((ROOT / 'coverage.json').read_text()))
            regions = {'count': 0, 'covered': 0}
            for data in report['data']:
                for item in data['files']:
                    if Path(item['filename']).is_relative_to(root / 'src'):
                        for key in regions:
                            regions[key] += item['summary']['regions'][key]
            with self.assertRaisesRegex(AssertionError, 'regions'):
                verify.check_counts({'lines': {'count': 1, 'covered': 1}, 'regions': regions})
