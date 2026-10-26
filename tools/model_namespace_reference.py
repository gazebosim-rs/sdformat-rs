#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Retain native namespace characterization; no parser parity or original credit."""
import argparse, datetime, hashlib, json, os, shlex, shutil, subprocess
from pathlib import Path

root = Path(__file__).resolve().parents[1]
base = root / 'tests'
parser = argparse.ArgumentParser()
parser.add_argument('upstream', type=Path)
parser.add_argument('--output-dir', type=Path, default=root/'artifacts/model-namespace-reference')
args = parser.parse_args()
upstream = args.upstream.resolve()
out = args.output_dir.resolve()
out.mkdir(parents=True, exist_ok=False)
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
revision = json.loads((root / 'reference/inventory.json').read_text())['upstream_sha']
assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=upstream, text=True).strip() == revision
files = ['src/Model.cc', 'src/parser.cc', 'src/Param.cc', 'src/Types.cc',
         'test/integration/includes.cc']
files += ['include/sdf/' + n for n in ['Model.hh', 'Root.hh', 'World.hh', 'Frame.hh', 'Element.hh']]
files += [f'sdf/1.{v}/model.sdf' for v in range(4, 13)]
inputs = {}
for rel in files:
    p = upstream / rel
    assert p.read_bytes() == subprocess.check_output(['git', 'show', revision + ':' + rel], cwd=upstream)
    inputs[str(p)] = sha(p)
compiler = Path(shutil.which('c++'))
for p in [base/'model_namespace_reference.cc', base/'model_namespace_xml.json', Path(__file__), compiler, upstream/'build/CMakeCache.txt', root/'reference/inventory.json']:
    inputs[str(p)] = sha(p)
cache = (upstream/'build/CMakeCache.txt').read_text()
assert Path(next(l.split('=', 1)[1] for l in cache.splitlines() if l.startswith('CMAKE_HOME_DIRECTORY:'))).resolve() == upstream
flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'sdformat'], text=True))
installed = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'sdformat'], text=True).strip())
for rel in ['Model.hh', 'Root.hh', 'World.hh', 'Frame.hh', 'Element.hh']:
    p = installed/'sdf'/rel
    assert sha(p) == inputs[str(upstream/'include/sdf'/rel)]
    inputs[str(p)] = sha(p)
env = os.environ.copy()
libdir = subprocess.check_output(['pkg-config', '--variable=libdir', 'sdformat'], text=True).strip()
assert libdir and Path(libdir).is_absolute()
env['LD_LIBRARY_PATH'] = os.pathsep.join([str(upstream/'build/lib'), libdir,
    *filter(None, env.get('LD_LIBRARY_PATH', '').split(os.pathsep))])
def capture(directory, command, timeout):
    directory.mkdir(exist_ok=False)
    def text(value):
        return value.decode(errors='replace') if isinstance(value, bytes) else value or ''
    try:
        result = subprocess.run(command, capture_output=True, text=True, timeout=timeout, env=env)
        state = dict(state='exited', exit=result.returncode)
        stdout, stderr = result.stdout, result.stderr
    except subprocess.TimeoutExpired as error:
        state = dict(state='timeout', timeout_seconds=timeout)
        stdout, stderr = text(error.stdout), text(error.stderr)
    except OSError as error:
        state = dict(state='launch_failed', error=str(error))
        stdout = stderr = ''
    state['command'] = command
    (directory/'stdout').write_text(stdout)
    (directory/'stderr').write_text(stderr)
    (directory/'exit.json').write_text(json.dumps(state, indent=2)+'\n')
    return state, stdout
binary = out/'namespace-probe'
command = [str(compiler), '-std=c++17', '-O2', '-Wall', '-Wextra', '-Werror', str(base/'model_namespace_reference.cc'), *flags, '-o', str(binary)]
(out/'inputs.json').write_text(json.dumps(dict(sha256=inputs, command=command, compiler_version=subprocess.check_output([str(compiler), '--version'], text=True)), indent=2)+'\n')
state, _ = capture(out/'compile', command, 60)
assert state['state'] == 'exited' and state['exit'] == 0
binary_hash = sha(binary)
ldd = subprocess.check_output(['ldd', str(binary)], env=env, text=True)
(out/'ldd.txt').write_text(ldd)
assert 'not found' not in ldd
libraries = {p: sha(p) for p in ldd.split() if p.startswith('/') and Path(p).is_file()}
sdf_libs = [p for p in libraries if Path(p).name.startswith('libsdformat.so.')]
assert len(sdf_libs) == 1
assert libraries[sdf_libs[0]] == sha(upstream/'build/lib'/Path(sdf_libs[0]).name)
corpus = json.loads((base/'model_namespace_xml.json').read_text())
assert len(corpus) == 98 and len({r['id'] for r in corpus}) == 98
observations = []
for row in corpus:
    case = out/row['id']
    case.mkdir()
    xml = row['xml']
    fixtures = {}
    if 'resource' in row:
        resource = case/'resource.sdf'
        resource.write_text(row['resource'])
        fixtures[str(resource)] = sha(resource)
        xml = xml.replace('@RESOURCE@', str(resource))
    assert '@RESOURCE@' not in xml
    fixture = case/'input.sdf'
    fixture.write_text(xml)
    fixtures[str(fixture)] = sha(fixture)
    state, stdout = capture(case/'run', [str(binary), str(fixture)], 10)
    actual = None
    if state['state'] == 'exited' and state['exit'] == 0:
        try:
            actual = json.loads(stdout)
        except json.JSONDecodeError:
            state['decode_failed'] = True
    observations.append(dict(id=row['id'], state=state, fixtures_sha256=fixtures, observations=actual))
    (out/'observations-partial.json').write_text(json.dumps(observations, indent=2)+'\n')
for p,h in inputs.items():
    assert sha(p) == h
for p,h in libraries.items():
    assert sha(p) == h
assert sha(binary) == binary_hash
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=upstream,text=True).strip() == revision
for row in observations:
    for p,h in row['fixtures_sha256'].items():
        assert sha(p) == h
raw = {str(p): sha(p) for p in out.rglob('*') if p.is_file() and p.name in ['stdout', 'stderr', 'exit.json', 'ldd.txt', 'inputs.json']}
report = dict(raw_sha256=raw, state='native_characterization_pending_independent_review', utc=datetime.datetime.now(datetime.timezone.utc).isoformat(), source_revision=revision, inputs_sha256=inputs, libraries_sha256=libraries, binary_sha256=binary_hash, observations=observations, scope='98 XML namespace projections;errors and diagnostics are native observations,not passes. Existing native build qualified. No Rust parser parity,original retirement,serialization,graph mutation,performance or SDF02 claim.')
(out/'report.json').write_text(json.dumps(report, indent=2)+'\n')
assert all(r['state']['state']=='exited' and r['state']['exit']==0 and r['observations'] is not None for r in observations)
def projection(model):
    if model is None:
        return None
    decode = lambda value: None if value is None else bytes.fromhex(value).decode()
    return dict(name=decode(model['name_hex']), raw_namespace=decode(model['raw_hex']),
                namespace=decode(model['resolved_hex']), links=model['links'],
                frames=[decode(frame) for frame in model['frames_hex']],
                models=[projection(child) for child in model['models']])
for case,row in zip(corpus, observations):
    assert case['id'] == row['id']
    actual = row['observations']
    assert not actual['errors'] and not actual['worlds'], case['id']
    assert projection(actual['root_model']) == case['expected'], case['id']
    # Native version provenance is retained, without a matching Rust Model API.
    assert actual['root_model']['original_version_hex'] == case['native_original_versions']
print('Verified98 native namespace projections;no additional original credit.')
