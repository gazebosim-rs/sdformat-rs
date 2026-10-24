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
from gz.math import Pose3d
root=Path(sys.argv[1])
path=root/'tests/differential.json'
results=[]
for case in json.loads(path.read_text()):
    try:
        document=sdformat.Root()
        document.load_sdf_string(case['xml'])
        accepted,errors=True,''
    except sdformat.SDFErrorsException as error:
        accepted,errors=False,str(error)
    assert accepted==case['accepted'],case['id']
    observed=dict(id=case['id'],accepted=accepted,errors=errors)
    if accepted and 'model_pose' in case:
        model=document.model() if document.model() else document.world_by_index(0).model_by_index(0)
        pose=model.semantic_pose().resolve()
        assert pose==Pose3d(*case['model_pose']),case['id']
        observed['model_pose']=[pose.pos().x(),pose.pos().y(),pose.pos().z(),pose.rot().x(),pose.rot().y(),pose.rot().z(),pose.rot().w()]
    if accepted and 'link_pose' in case:
        pose=document.model().link_by_index(0).semantic_pose().resolve()
        assert pose==Pose3d(*case['link_pose']),case['id']
        observed['link_pose']=[pose.pos().x(),pose.pos().y(),pose.pos().z(),pose.rot().x(),pose.rot().y(),pose.rot().z(),pose.rot().w()]
    if accepted and 'link_mass' in case:
        mass=document.model().link_by_index(0).inertial().mass_matrix().mass()
        assert mass==case['link_mass'],case['id']
        observed['link_mass']=mass
    if accepted and 'model_name' in case:
        model=document.model()
        plugin=model.plugins()[0]
        assert model.name()==case['model_name'] and plugin.name()==case['plugin_name'] and plugin.filename()==case['plugin_filename'],case['id']
        observed['model_name']=model.name()
        observed['plugin_name']=plugin.name()
        observed['plugin_filename']=plugin.filename()
    results.append(observed)
output=dict(corpus_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),results=results)
(root/'artifacts').mkdir(exist_ok=True)
(root/'artifacts/reference-probe.json').write_text(json.dumps(output,indent=2)+'\\n')
print(json.dumps(output,indent=2))
'''

if __name__ == '__main__': main()
