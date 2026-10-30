// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/World.hh>
#include <sdf/Model.hh>
#include <sdf/Link.hh>
#include <sdf/Sensor.hh>
#include <sdf/Element.hh>
#include <iostream>
#include <fstream>
#include <iterator>
int main(int argc, char **argv) {
  std::ifstream file(argv[1]);
  std::string xml{std::istreambuf_iterator<char>(file), {}};
  sdf::Root root;
  auto errors = root.LoadSdfString(xml);
  for (const auto &e : errors) std::cout << "ERROR " << int(e.Code()) << ' ' << e.Message() << '\n';
  auto world = root.WorldByIndex(0);
  if (!world) return 1;
  auto link = world->ModelByName("model_with_sensor")->LinkByName("link");
  for (uint64_t i=0; i<link->SensorCount(); ++i) {
    auto s=link->SensorByIndex(i);auto e=s->Element();
    std::cout << "SENSOR " << s->Name() << ' ' << s->TypeStr() << ' ' << s->UpdateRate() << ' ' << e->Get<bool>("always_on") << ' ' << e->Get<bool>("visualize") << ' ' << s->Topic() << '\n';
    auto a=e->GetAttribute("gz:type");std::cout << "CUSTOM_TYPE " << (a?a->GetAsString():"ABSENT") << '\n';
    for(auto child=e->GetFirstElement();child;child=child->GetNextElement()) std::cout << "CHILD " << child->GetName() << ' ' << (child->GetValue() ? child->GetValue()->GetAsString() : "NO_VALUE") << '\n';
  }
}
