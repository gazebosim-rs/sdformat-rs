// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/Model.hh>
#include <sdf/Joint.hh>
#include <sdf/JointAxis.hh>
#include <sdf/Element.hh>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
static unsigned long long bits(double value) {
  unsigned long long out;
  std::memcpy(&out, &value, sizeof(out));
  return out;
}
int main() {
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
    std::cout << ' ' << axis->Element()->HasElement("dynamics");
    for (double value : {axis->Damping(), axis->Friction(), axis->SpringReference(),
                        axis->SpringStiffness(), axis->Stiffness(), axis->Dissipation()})
      std::cout << ' ' << bits(value);
  }
  std::cout << '\n';
}
