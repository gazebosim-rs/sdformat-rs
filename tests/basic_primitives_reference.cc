// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Box.hh>
#include <sdf/Collision.hh>
#include <sdf/Cylinder.hh>
#include <sdf/Geometry.hh>
#include <sdf/Link.hh>
#include <sdf/Model.hh>
#include <sdf/Plane.hh>
#include <sdf/Root.hh>
#include <sdf/Sphere.hh>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
#include <vector>

int main()
{
  std::string xml((std::istreambuf_iterator<char>(std::cin)), {});
  sdf::Root root;
  if (!root.LoadSdfString(xml).empty())
  {
    std::cout << "error\n";
    return 0;
  }
  const auto *geometry = root.Model()->LinkByIndex(0)->CollisionByIndex(0)->Geom();
  std::vector<double> values;
  if (const auto *box = geometry->BoxShape())
  {
    const auto size = box->Size();
    values = {size.X(), size.Y(), size.Z()};
  }
  else if (const auto *sphere = geometry->SphereShape())
    values = {sphere->Radius()};
  else if (const auto *cylinder = geometry->CylinderShape())
    values = {cylinder->Radius(), cylinder->Length()};
  else if (const auto *plane = geometry->PlaneShape())
  {
    const auto normal = plane->Normal();
    const auto size = plane->Size();
    values = {normal.X(), normal.Y(), normal.Z(), size.X(), size.Y()};
  }
  else
    return 2;
  for (double value : values)
  {
    std::uint64_t bits;
    static_assert(sizeof(bits) == sizeof(value));
    std::memcpy(&bits, &value, sizeof(bits));
    std::cout << bits << '\n';
  }
}
