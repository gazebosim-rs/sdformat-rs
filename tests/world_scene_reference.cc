// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/World.hh>
#include <sdf/Scene.hh>
#include <sdf/Element.hh>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
void values(const sdf::Scene &scene) {
  std::cout << "VALUES";
  for (const auto &color : {scene.Ambient(), scene.Background()}) {
    for (float value : {color.R(), color.G(), color.B(), color.A()}) {
      std::uint32_t bits; std::memcpy(&bits, &value, sizeof(bits));std::cout << ' ' << bits;
    }
  }
  std::cout << ' ' << scene.Grid() << ' ' << scene.Shadows() << ' ' << scene.OriginVisual() << ' ' << bool(scene.Sky()) << ' ' << bool(scene.Element()) << '\n';
}
int main(int argc,char **argv) {
  if (argc == 2 && std::string(argv[1]) == "--constructor") { values(sdf::Scene{});return 0; }
  sdf::Root root;
  sdf::Errors errors;
  if (argc == 3 && std::string(argv[1]) == "--file") errors=root.Load(argv[2]);
  else { const std::string xml{std::istreambuf_iterator<char>(std::cin),{}};errors=root.LoadSdfString(xml); }
  std::cout << "ERRORS";for(const auto &e:errors)std::cout << ' ' << int(e.Code());std::cout << '\n';
  const auto *world=root.WorldByIndex(0);std::cout << "WORLD " << bool(world) << '\n';
  if (world) {std::cout << "SCENE " << bool(world->Scene()) << '\n';if(world->Scene())values(*world->Scene());}
}
