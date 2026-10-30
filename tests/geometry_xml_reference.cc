// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Geometry.hh>
#include <sdf/Box.hh>
#include <sdf/Sphere.hh>
#include <sdf/Cylinder.hh>
#include <sdf/Capsule.hh>
#include <sdf/Cone.hh>
#include <sdf/Ellipsoid.hh>
#include <sdf/Plane.hh>
#include <sdf/Element.hh>
#include <sdf/SDFImpl.hh>
#include <sdf/parser.hh>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
void number(double n){uint64_t b;std::memcpy(&b,&n,8);std::cout<<' '<<b;}
void values(const sdf::Geometry &g) {
 unsigned mask=bool(g.BoxShape())|(bool(g.CapsuleShape())<<1)|(bool(g.ConeShape())<<2)|(bool(g.CylinderShape())<<3)|(bool(g.EllipsoidShape())<<4)|(bool(g.SphereShape())<<5)|(bool(g.PlaneShape())<<6)|(bool(g.MeshShape())<<7)|(bool(g.HeightmapShape())<<8)|(!g.PolylineShape().empty()<<9);
 std::cout<<"VALUES "<<int(g.Type())<<' '<<mask;
 if(auto v=g.BoxShape())for(double n:{v->Size().X(),v->Size().Y(),v->Size().Z()})number(n);
 if(auto v=g.SphereShape())number(v->Radius());
 if(auto v=g.CapsuleShape()){number(v->Radius());number(v->Length());}
 if(auto v=g.ConeShape()){number(v->Radius());number(v->Length());}
 if(auto v=g.CylinderShape()){number(v->Radius());number(v->Length());}
 if(auto v=g.EllipsoidShape())for(double n:{v->Radii().X(),v->Radii().Y(),v->Radii().Z()})number(n);
 if(auto v=g.PlaneShape())for(double n:{v->Normal().X(),v->Normal().Y(),v->Normal().Z(),v->Size().X(),v->Size().Y()})number(n);
 std::cout<<'\n';
}
int main(int argc,char**argv){
 if(argc==2 && std::string(argv[1])=="--plane-profiles") {
  for(const auto &normal: {gz::math::Vector3d::Zero, gz::math::Vector3d(1e-7,0,0), gz::math::Vector3d(1e-6,0,0), gz::math::Vector3d(1e-5,0,0), gz::math::Vector3d(1,2,3), gz::math::Vector3d(1e300,1e300,0)}) {
   sdf::Plane plane;plane.SetNormal(normal);sdf::Geometry geometry;geometry.SetType(sdf::GeometryType::PLANE);geometry.SetPlaneShape(plane);values(geometry);
   sdf::Errors errors;auto element=geometry.ToElement(errors);if(!element||!errors.empty())return 5;
   sdf::Geometry loaded;errors=loaded.Load(element);if(!errors.empty())return 6;values(loaded);
  }
  return 0;
 }
 if(argc==2){std::string mode=argv[1];sdf::Geometry g;
  if(mode=="box"){g.SetType(sdf::GeometryType::BOX);g.SetBoxShape(sdf::Box());}
  else if(mode=="sphere"){g.SetType(sdf::GeometryType::SPHERE);g.SetSphereShape(sdf::Sphere());}
  else if(mode=="cylinder"){g.SetType(sdf::GeometryType::CYLINDER);g.SetCylinderShape(sdf::Cylinder());}
  else if(mode=="capsule"){g.SetType(sdf::GeometryType::CAPSULE);g.SetCapsuleShape(sdf::Capsule());}
  else if(mode=="cone"){g.SetType(sdf::GeometryType::CONE);g.SetConeShape(sdf::Cone());}
  else if(mode=="ellipsoid"){g.SetType(sdf::GeometryType::ELLIPSOID);g.SetEllipsoidShape(sdf::Ellipsoid());}
  else if(mode=="plane"){g.SetType(sdf::GeometryType::PLANE);g.SetPlaneShape(sdf::Plane());}
  else return 2;
  sdf::Errors e;auto element=g.ToElement(e);if(!element||!e.empty())return 3;std::cout<<element->ToString("");return 0;
 }
 std::string input{std::istreambuf_iterator<char>(std::cin),{}};sdf::SDFPtr d(new sdf::SDF);sdf::init(d);sdf::Errors e;
 bool ok=sdf::readString("<sdf version='1.12'><model name='m'><link name='l'><collision name='c'>"+input+"</collision></link></model></sdf>",d,e);
 std::cout<<"READ "<<ok;for(auto &v:e)std::cout<<' '<<int(v.Code());std::cout<<'\n';if(!ok||!e.empty())return 0;
 auto c=d->Root()->GetElement("model")->GetElement("link")->GetElement("collision");if(!c->HasElement("geometry"))return 4;
 sdf::Geometry g;e=g.Load(c->GetElement("geometry"));std::cout<<"LOAD";for(auto &v:e)std::cout<<' '<<int(v.Code());std::cout<<'\n';values(g);
}
