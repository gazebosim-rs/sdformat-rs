// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/Model.hh>
#include <sdf/Link.hh>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
int main()
{
  const std::string xml((std::istreambuf_iterator<char>(std::cin)), {});
  sdf::Root root;
  if (!root.LoadSdfString(xml).empty())
  {
    std::cout << "error\n";
    return 0;
  }
  const auto &fluid = root.Model()->LinkByIndex(0)->Inertial().FluidAddedMass();
  if (!fluid)
  {
    std::cout << "none\n";
    return 0;
  }
  std::cout << "matrix\n";
  for (unsigned row = 0; row < 6; ++row)
    for (unsigned column = 0; column < 6; ++column)
    {
      const double value = (*fluid)(row, column);
      std::uint64_t bits;
      std::memcpy(&bits, &value, sizeof(bits));
      std::cout << bits << '\n';
    }
}
