// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/World.hh>
#include <sdf/Atmosphere.hh>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
static std::uint64_t bits(double value)
{
  std::uint64_t output;
  std::memcpy(&output, &value, sizeof(output));
  return output;
}
int main()
{
  std::string xml((std::istreambuf_iterator<char>(std::cin)), {});
  sdf::Root root;
  const auto errors = root.LoadSdfString(xml);
  std::cout << errors.size();
  if (!errors.empty())
  {
    for (const auto &error : errors)
      std::cout << ' ' << static_cast<int>(error.Code());
    std::cout << '\n';
    return 0;
  }
  const auto *world = root.WorldByIndex(0);
  const auto *atmosphere = world ? world->Atmosphere() : nullptr;
  std::cout << ' ' << (atmosphere ? 1 : 0);
  if (atmosphere)
  {
    std::cout << ' '
      << (atmosphere->Type() == sdf::AtmosphereType::ADIABATIC ? "adiabatic" : "unknown")
      << ' ' << bits(atmosphere->Temperature().Kelvin())
      << ' ' << bits(atmosphere->Pressure())
      << ' ' << bits(atmosphere->TemperatureGradient());
  }
  std::cout << '\n';
}
