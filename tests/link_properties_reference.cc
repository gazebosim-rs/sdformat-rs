// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/Model.hh>
#include <sdf/Link.hh>
#include <sdf/Joint.hh>
#include <limits>
#include <sdf/Element.hh>
#include <iostream>
#include <iterator>
#include <string>
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
  const auto *model = root.Model();
  std::cout << ' ' << model->LinkCount();
  for (std::uint64_t i = 0; i < model->LinkCount(); ++i) {
    const auto *link = model->LinkByIndex(i);
    std::cout << ' ' << link->EnableWind() << ' ' << link->EnableGravity()
              << ' ' << link->Kinematic() << ' '
              << link->Element()->Get<bool>("self_collide", false).first;
  }
  if (model->JointCount() > 0) {
    const auto *joint = model->JointByIndex(0);
    std::cout << " A";
    for (unsigned index : {0u, 1u, 2u, std::numeric_limits<unsigned>::max()}) {
      const auto *axis = joint->Axis(index);
      std::cout << ' ' << (axis != nullptr) << ' ' << (axis == joint->Axis(1));
    }
  }
  std::cout << '\n';
}
