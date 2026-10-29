// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/World.hh>
#include <sdf/Element.hh>
#include <sdf/parser.hh>
#include <sdf/SDFImpl.hh>
#include <iostream>
#include <iterator>
#include <cstring>
#include <cstdint>
void vector(const gz::math::Vector3d &v) {for(double n:{v.X(),v.Y(),v.Z()}){uint64_t b;std::memcpy(&b,&n,8);std::cout<<' '<<b;}}
void world(const sdf::World &w){std::cout<<"VALUES";vector(w.WindLinearVelocity());vector(w.MagneticField());vector(w.Gravity());std::cout<<'\n';}
int main(int argc,char **argv) {
  if (argc == 2 && std::string(argv[1]) == "--profiles") {
    sdf::World constructed; std::cout << "constructor\n"; world(constructed);
    sdf::ElementPtr raw(new sdf::Element); raw->SetName("world");
    raw->AddAttribute("name", "string", "w", true);
    sdf::World loaded; auto errors=loaded.Load(raw);
    std::cout << "raw"; for(const auto &e:errors)std::cout << ' ' << int(e.Code());
    std::cout << '\n';world(loaded);return 0;
  }
  if(argc != 1)return 2;
  std::string input{std::istreambuf_iterator<char>(std::cin),{}};
  sdf::Root root;auto errors=root.LoadSdfString(input);
  std::cout << "ROOT";for(const auto &e:errors)std::cout << ' ' << int(e.Code());
  std::cout << '\n';if(root.WorldByIndex(0))world(*root.WorldByIndex(0));
}
