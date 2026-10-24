#!/usr/bin/env python3
"""Run acceptance corpus against the pinned native Python binding."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('upstream', type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    upstream = args.upstream.resolve()
    inventory = json.loads((root / 'reference/inventory.json').read_text())
    sha = subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert sha == inventory['upstream_sha'], 'reference revision mismatch'
    if '--native' not in sys.argv:
        env = dict(os.environ)
        env['PYTHONPATH'] = ':'.join([str(upstream / 'build/lib'), str(Path.home() / 'local/lib/python')])
        env['LD_LIBRARY_PATH'] = ':'.join([str(upstream / 'build/lib'), str(Path.home() / 'local/lib')])
        # The built extension matches the distribution Python, not a user venv.
        subprocess.run(['/usr/bin/python3', '-c', NATIVE, str(root)], check=True, env=env)


NATIVE = '''
import hashlib,json,sys
from pathlib import Path
import sdformat
root=Path(sys.argv[1])
path=root/'tests/differential.json'
results=[]
for case in json.loads(path.read_text()):
    try:
        sdformat.Root().load_sdf_string(case['xml'])
        accepted,errors=True,''
    except sdformat.SDFErrorsException as error:
        accepted,errors=False,str(error)
    assert accepted==case['accepted'],case['id']
    results.append(dict(id=case['id'],accepted=accepted,errors=errors))
output=dict(corpus_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),results=results)
(root/'artifacts').mkdir(exist_ok=True)
(root/'artifacts/reference-probe.json').write_text(json.dumps(output,indent=2)+'\\n')
print(json.dumps(output,indent=2))
'''

if __name__ == '__main__': main()
