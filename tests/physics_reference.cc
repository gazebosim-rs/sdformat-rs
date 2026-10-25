// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/World.hh>
#include <sdf/Physics.hh>
#include <sdf/Element.hh>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
static unsigned long long bits(double value) { unsigned long long out; std::memcpy(&out, &value, sizeof(out)); return out; }
static void text(const std::string &value) {
  if (value.empty()) {std::cout << '-'; return;}
  for (unsigned char c : value) {const char *hex="0123456789abcdef";std::cout<<hex[c>>4]<<hex[c&15];}
}
int main() {
  sdf::Root root;
  std::string xml((std::istreambuf_iterator<char>(std::cin)), {});
  auto errors=root.LoadSdfString(xml);
  std::cout<<errors.size();
  if(!errors.empty()) {for(const auto &error:errors) std::cout<<' '<<static_cast<int>(error.Code());std::cout<<'\n';return 0;}
  const auto *world=root.WorldByIndex(0);
  std::cout<<' '<<world->PhysicsCount();
  long long selected=-1;
  for(std::uint64_t i=0;i<world->PhysicsCount();++i) if(world->PhysicsByIndex(i)==world->PhysicsDefault())selected=i;
  std::cout<<' '<<selected;
  for(std::uint64_t i=0;i<world->PhysicsCount();++i) {
    const auto *p=world->PhysicsByIndex(i);
    std::cout<<' ';text(p->Name());std::cout<<' '<<p->IsDefault()<<' ';text(p->EngineType());
    std::cout<<' '<<bits(p->MaxStepSize())<<' '<<bits(p->RealTimeFactor())<<' '<<p->MaxContacts();
    auto e=p->Element();std::cout<<' '<<(e?1:0)<<' '<<(e?bits(e->Get<double>("real_time_update_rate")):0);
  }
  std::cout<<'\n';
}
