// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <iomanip>
#include <iostream>
#include <iterator>
#include <string>
#include <sdf/Gui.hh>
#include <sdf/Root.hh>
#include <sdf/World.hh>
void string(const std::string &value) {
  std::cout << std::dec << value.size() << ':';
  for (unsigned char byte : value)
    std::cout << std::hex << std::setw(2) << std::setfill('0') << unsigned(byte);
  std::cout << std::dec;
}
void gui(const sdf::Gui *value) {
  std::cout << "GUI " << bool(value) << '\n';
  if (!value) return;
  std::cout << "FULLSCREEN " << value->Fullscreen()
            << "\nELEMENT " << bool(value->Element())
            << "\nPLUGINS " << value->PluginCount() << '\n';
  for (const auto &plugin : value->Plugins()) {
    std::cout << "PLUGIN "; string(plugin.Name()); std::cout << ' ';
    string(plugin.Filename()); std::cout << '\n';
  }
}
int main(int argc, char **) {
  if (argc > 1) {
    gui(sdf::World{}.Gui());
    const sdf::Gui constructed;
    gui(&constructed);
    return 0;
  }
  const std::string xml{std::istreambuf_iterator<char>(std::cin), {}};
  sdf::Root root;
  const auto errors = root.LoadSdfString(xml);
  std::cout << "ERRORS " << errors.size();
  for (const auto &error : errors) std::cout << ' ' << int(error.Code());
  std::cout << '\n';
  if (root.WorldCount()) gui(root.WorldByIndex(0)->Gui());
}
