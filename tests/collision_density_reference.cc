// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Collision.hh>
#include <sdf/Geometry.hh>
#include <sdf/Element.hh>
#include <sdf/SDFImpl.hh>
#include <sdf/parser.hh>
#include <iostream>
#include <iterator>
#include <cstdint>
#include <cstring>
#include <limits>
void number(double value) {uint64_t bits;std::memcpy(&bits,&value,8);std::cout<<' '<<bits;}
void errors(const char *tag,const sdf::Errors &errors){std::cout<<tag;for(auto &e:errors)std::cout<<' '<<int(e.Code());std::cout<<'\n';}
void emit(const sdf::Collision &value) {
 std::cout<<"VALUE";number(value.Density());std::cout<<'\n';sdf::Errors e;auto xml=value.ToElement(e);errors("EMIT",e);std::cout<<"PRESENT "<<xml->HasElement("density")<<'\n';if(xml->HasElement("density")){std::cout<<"STORED";number(xml->Get<double>("density"));std::cout<<'\n';}
 sdf::Collision loaded;e=loaded.Load(xml);errors("RELOAD",e);std::cout<<"RELOADED";number(loaded.Density());std::cout<<'\n';std::cout<<"XML_BEGIN\n"<<xml->ToString("")<<"XML_END\n";
}
int main(int argc,char **argv) {
 sdf::Collision value;value.SetName("c");
 if(argc>1 && std::string(argv[1])=="setters") {
  std::cout<<"DEFAULT";number(sdf::Collision::DensityDefault());std::cout<<'\n';emit(value);
  for(double d:{0.,-0.,-1.,1000.,1234.5,std::numeric_limits<double>::max(),std::numeric_limits<double>::min(),std::numeric_limits<double>::denorm_min(),std::numeric_limits<double>::quiet_NaN(),std::numeric_limits<double>::infinity()}) {std::cout<<"SET";number(d);std::cout<<'\n';value.SetDensity(d);emit(value);}return 0;
 }
 std::string input{std::istreambuf_iterator<char>(std::cin),{}};sdf::SDFPtr doc(new sdf::SDF);sdf::init(doc);sdf::Errors e;bool ok=sdf::readString("<sdf version='1.12'><model name='m'><link name='l'><collision name='c'>"+input+"</collision></link></model></sdf>",doc,e);std::cout<<"READ "<<ok<<'\n';errors("SCHEMA",e);if(!ok)return 0;auto xml=doc->Root()->GetElement("model")->GetElement("link")->GetElement("collision");e=value.Load(xml);errors("LOAD",e);emit(value);
}
