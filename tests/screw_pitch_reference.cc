// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
#include <sdf/Joint.hh>
#include <sdf/Model.hh>
#include <sdf/Root.hh>

void number(double value)
{
  uint64_t bits;
  std::memcpy(&bits, &value, sizeof(bits));
  std::cout << ' ' << bits;
}
int main()
{
  const std::string xml{std::istreambuf_iterator<char>(std::cin), {}};
  sdf::Root root;
  const auto errors = root.LoadSdfString(xml);
  std::cout << errors.size();
  if (errors.empty())
  {
    const auto *joint = root.Model()->JointByIndex(0);
    number(joint->ScrewThreadPitch());
    number(joint->ThreadPitch());
  }
  std::cout << '\n';
}
