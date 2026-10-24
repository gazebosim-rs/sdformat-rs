#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0

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
    parser.add_argument('--corpus', type=Path, default=Path('tests/differential.json'))
    parser.add_argument('--output', type=Path, default=Path('artifacts/reference-probe.json'))
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
        subprocess.run(['/usr/bin/python3', '-c', NATIVE, str(root), str(args.corpus), str(args.output)], check=True, env=env)


NATIVE = '''
import hashlib,json,sys,struct
from pathlib import Path
import sdformat
from gz.math import Pose3d
root=Path(sys.argv[1])
path=root/sys.argv[2]
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
    if accepted and 'declaration_names' in case:
        owner=document.world_by_index(0) if case['names_scope']=='world' else document.model()
        names={kind:[getattr(owner,kind+'_by_index')(i).name() for i in range(getattr(owner,kind+'_count')())] for kind in case['declaration_names']}
        assert names==case['declaration_names'],case['id']
        observed['declaration_names']=names
        if 'joint_bodies' in case:
            joint=owner.joint_by_index(0)
            bodies=[joint.resolve_parent_link(),joint.resolve_child_link()]
            assert bodies==case['joint_bodies'],case['id']
            observed['joint_bodies']=bodies
    if accepted and 'surface_coefficient_bits' in case:
        value=document.model().link_by_index(0).collision_by_index(0).surface().friction().torsional().coefficient()
        bits=struct.unpack('<Q',struct.pack('<d',value))[0]
        assert bits==case['surface_coefficient_bits'],case['id']
        observed['surface_coefficient_bits']=bits
    if accepted and 'surface' in case:
        surface=document.model().link_by_index(0).collision_by_index(0).surface()
        contact=surface.contact()
        friction=surface.friction()
        ode=friction.ode()
        bullet=friction.bullet_friction()
        torsional=friction.torsional()
        vector=lambda v:[v.x(),v.y(),v.z()]
        measured=dict(contact=dict(collide_bitmask=contact.collide_bitmask(),category_bitmask=contact.category_bitmask()),
            ode=dict(mu=ode.mu(),mu2=ode.mu2(),slip1=ode.slip1(),slip2=ode.slip2(),fdir1=vector(ode.fdir1())),
            bullet=None if bullet is None else dict(friction=bullet.friction(),friction2=bullet.friction2(),fdir1=vector(bullet.fdir1()),rolling_friction=bullet.rolling_friction()),
            torsional=None if torsional is None else dict(coefficient=torsional.coefficient(),use_patch_radius=torsional.use_patch_radius(),patch_radius=torsional.patch_radius(),surface_radius=torsional.surface_radius(),ode_slip=torsional.ode_slip()))
        assert measured==case['surface'],case['id']
        observed['surface']=measured
    if accepted and 'geometry' in case:
        expected=case['geometry']
        kind=expected['kind']
        geometry=document.model().link_by_index(0).collision_by_index(0).geometry()
        shape=getattr(geometry,kind+'_shape')()
        assert shape is not None,case['id']
        if kind in ('capsule','cone'):
            measured=dict(kind=kind,radius=shape.radius(),length=shape.length())
        elif kind=='ellipsoid':
            radii=shape.radii()
            measured=dict(kind=kind,radii=[radii.x(),radii.y(),radii.z()])
        else:
            scale=shape.scale()
            convex=shape.convex_decomposition()
            measured=dict(kind=kind,uri=shape.uri(),scale=[scale.x(),scale.y(),scale.z()],submesh=shape.submesh(),center_submesh=shape.center_submesh(),optimization=shape.optimization_str(),convex_decomposition=None if convex is None else [convex.max_convex_hulls(),convex.voxel_resolution()])
        assert measured==expected,case['id']
        observed['geometry']=measured
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
    if accepted and 'deferred_pose_error' in case:
        owner=case['deferred_pose_error']
        link=document.model().link_by_index(0)
        object=getattr(link,owner+'_by_index')(0)
        try:
            object.semantic_pose().resolve()
            raise AssertionError(case['id']+' expected deferred failure')
        except sdformat.SDFErrorsException as error:
            observed['deferred_pose_error']=owner
            observed['resolve_errors']=str(error)
    if accepted and 'visual_geometry_missing' in case:
        missing=document.model().link_by_index(0).visual_by_index(0).geometry().type()==sdformat.GeometryType.EMPTY
        assert missing==case['visual_geometry_missing'],case['id']
        observed['visual_geometry_missing']=missing
    if accepted and 'axis2_missing' in case:
        missing=document.model().joint_by_index(0).axis(1) is None
        assert missing==case['axis2_missing'],case['id']
        observed['axis2_missing']=missing
    if accepted and 'link_relative_to' in case:
        value=document.model().link_by_index(0).pose_relative_to()
        assert value==case['link_relative_to'],case['id']
        observed['link_relative_to']=value
    if accepted and 'joint_axes' in case:
        joint=document.model().joint_by_index(0)
        axes=[joint.axis(index) for index in range(2) if joint.axis(index) is not None]
        assert len(axes)==len(case['joint_axes']),case['id']
        measured=[]
        for axis,expected in zip(axes,case['joint_axes']):
            assert axis.xyz_expressed_in()==expected['expressed_in'],case['id']
            value=axis.resolve_xyz('__model__')
            values=[value.x(),value.y(),value.z()]
            assert all(abs(a-b)<1e-12 for a,b in zip(values,expected['resolved'])),case['id']
            measured.append(dict(expressed_in=axis.xyz_expressed_in(),resolved=values))
        observed['joint_axes']=measured
    if accepted and 'axis_missing' in case:
        missing=document.model().joint_by_index(0).axis(0) is None
        assert missing==case['axis_missing'],case['id']
        observed['axis_missing']=missing
    results.append(observed)
output=dict(corpus_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),results=results)
(root/'artifacts').mkdir(exist_ok=True)
(root/sys.argv[3]).write_text(json.dumps(output,indent=2)+'\\n')
print(json.dumps(output,indent=2))
'''

if __name__ == '__main__': main()
