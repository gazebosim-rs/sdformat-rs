// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <iomanip>
#include <iostream>
#include <iterator>
#include <string>
#include <sdf/Root.hh>
#include <sdf/World.hh>

void device(const sdf::World &world) {
  const auto value = world.AudioDevice();
  std::cout << "DEVICE " << value.size() << ':';
  for (unsigned char byte : value)
    std::cout << std::hex << std::setw(2) << std::setfill('0') << unsigned(byte);
  std::cout << '\n';
}
int main(int argc, char **) {
  if (argc > 1) { device(sdf::World{}); return 0; }
  const std::string xml{std::istreambuf_iterator<char>(std::cin), {}};
  sdf::Root root;
  const auto errors = root.LoadSdfString(xml);
  std::cout << "ERRORS " << errors.size() << '\n';
  if (root.WorldCount()) device(*root.WorldByIndex(0));
}
