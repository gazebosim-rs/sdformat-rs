// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
#include <sdf/Root.hh>
#include <sdf/World.hh>
void bits(double value) {
  uint64_t result;
  std::memcpy(&result, &value, sizeof(result));
  std::cout << ' ' << result;
}
void coordinates(const sdf::World &world) {
  const auto *value = world.SphericalCoordinates();
  std::cout << "COORDS " << bool(value) << '\n';
  if (!value) return;
  std::cout << "SURFACE " << int(value->Surface()) << '\n';
  std::cout << "RADIANS";
  bits(value->LatitudeReference().Radian());
  bits(value->LongitudeReference().Radian());
  bits(value->HeadingOffset().Radian());
  std::cout << "\nDEGREES";
  bits(value->LatitudeReference().Degree());
  bits(value->LongitudeReference().Degree());
  bits(value->HeadingOffset().Degree());
  std::cout << "\nMETRES";
  bits(value->ElevationReference());
  bits(value->SurfaceAxisEquatorial());
  bits(value->SurfaceAxisPolar());
  std::cout << '\n';
}
int main(int argc, char **) {
  if (argc > 1) { coordinates(sdf::World{}); return 0; }
  const std::string xml{std::istreambuf_iterator<char>(std::cin), {}};
  sdf::Root root;
  auto errors = root.LoadSdfString(xml);
  std::cout << "ERRORS " << errors.size();
  for (const auto &error : errors) std::cout << ' ' << int(error.Code());
  std::cout << '\n';
  if (root.WorldCount()) coordinates(*root.WorldByIndex(0));
}
