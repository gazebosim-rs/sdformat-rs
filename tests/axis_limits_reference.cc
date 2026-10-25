// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/Model.hh>
#include <sdf/Joint.hh>
#include <sdf/JointAxis.hh>
#include <sdf/Element.hh>
#include <sdf/SDFImpl.hh>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
static unsigned long long bits(double value) {
  unsigned long long out;
  std::memcpy(&out, &value, sizeof(out));
  return out;
}
int main(int argc, char **argv) {
  if (argc == 2 && std::string(argv[1]) == "--joint-spec") {
    std::cout << sdf::SDF::EmbeddedSpec("joint.sdf", false);
    return 0;
  }
  if (argc != 1) return 2;
  sdf::Root root;
  std::string xml((std::istreambuf_iterator<char>(std::cin)), {});
  auto errors = root.LoadSdfString(xml);
  std::cout << errors.size();
  if (!errors.empty()) {
    for (const auto &error : errors) std::cout << ' ' << static_cast<int>(error.Code());
    std::cout << '\n';
    return 0;
  }
  const auto *joint = root.Model()->JointByIndex(0);
  for (unsigned i = 0; i < 2; ++i) {
    const auto *axis = joint->Axis(i);
    std::cout << ' ' << (axis != nullptr);
    if (!axis) continue;
    for (double value : {axis->Lower(), axis->Upper(), axis->Effort(), axis->MaxVelocity()})
      std::cout << ' ' << bits(value);
  }
  std::cout << '\n';
}
