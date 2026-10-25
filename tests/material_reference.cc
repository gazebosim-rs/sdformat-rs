// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/Model.hh>
#include <sdf/Link.hh>
#include <sdf/Visual.hh>
#include <sdf/Material.hh>
#include <gz/math/Color.hh>
#include <iostream>
#include <iterator>
#include <cstring>
#include <cstdint>
void text(const std::string &s) {
  if(s.empty()){std::cout<<" -";return;}
  const char *digits="0123456789abcdef";std::cout<<' ';
  for(unsigned char c:s)std::cout<<digits[c>>4]<<digits[c&15];
}
void number(float value){uint32_t bits;std::memcpy(&bits,&value,sizeof(bits));std::cout<<' '<<bits;}
void number(double value){uint64_t bits;std::memcpy(&bits,&value,sizeof(bits));std::cout<<' '<<bits;}
void color(const gz::math::Color &c){number(c.R());number(c.G());number(c.B());number(c.A());}
int main(){
 std::string xml{std::istreambuf_iterator<char>(std::cin),{}};sdf::Root root;
 auto errors=root.LoadSdfString(xml);std::cout<<errors.size();
 if(!errors.empty()){for(const auto&e:errors)std::cout<<' '<<static_cast<int>(e.Code());std::cout<<'\n';return 0;}
 auto m=root.Model()->LinkByIndex(0)->VisualByIndex(0)->Material();std::cout<<' '<<(m!=nullptr);
 if(m){color(m->Ambient());color(m->Diffuse());color(m->Specular());color(m->Emissive());number(m->Shininess());number(m->RenderOrder());std::cout<<' '<<m->Lighting()<<' '<<m->DoubleSided()<<' '<<static_cast<int>(m->Shader());text(m->NormalMap());text(m->ScriptUri());text(m->ScriptName());}
 std::cout<<'\n';
}
