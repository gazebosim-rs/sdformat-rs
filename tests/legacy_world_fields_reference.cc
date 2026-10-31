// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Element.hh>
#include <sdf/Physics.hh>
#include <sdf/Root.hh>
#include <sdf/SDFImpl.hh>
#include <sdf/World.hh>
#include <sdf/parser.hh>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>

void vector(const gz::math::Vector3d &value)
{
  for (double scalar : {value.X(), value.Y(), value.Z()})
  {
    uint64_t bits;
    std::memcpy(&bits, &scalar, sizeof(bits));
    std::cout << ' ' << bits;
  }
}

unsigned int count(const sdf::ElementPtr &parent, const std::string &name)
{
  unsigned int result = 0;
  if (parent)
    for (auto child = parent->GetFirstElement(); child;
         child = child->GetNextElement())
      result += child->GetName() == name;
  return result;
}

void element(const sdf::ElementPtr &world)
{
  std::cout << "ELEMENT " << count(world, "gravity") << ' '
            << count(world, "magnetic_field");
  if (world)
    for (auto child = world->GetFirstElement(); child;
         child = child->GetNextElement())
      if (child->GetName() == "physics")
        std::cout << ' ' << count(child, "gravity") << ' '
                  << count(child, "magnetic_field");
  std::cout << '\n';
}

int main(int argc, char **argv)
{
  std::string xml{std::istreambuf_iterator<char>(std::cin), {}};
  if (argc == 2 && std::string(argv[1]) == "--convert")
  {
    sdf::SDFPtr document(new sdf::SDF);
    sdf::init(document);
    const bool success = sdf::convertString(xml, "1.6", document);
    std::cout << "CONVERT " << success << '\n';
    if (success)
    {
      auto root = document->Root();
      auto world = root->GetElement("world");
      auto physics = world->GetElement("physics");
      std::cout << "VERSION " << root->Get<std::string>("version") << ' '
                << document->OriginalVersion() << ' ' << root->OriginalVersion()
                << ' ' << world->OriginalVersion() << ' '
                << physics->OriginalVersion() << '\n';
      std::cout << "VALUES";
      vector(world->Get<gz::math::Vector3d>("gravity"));
      vector(world->Get<gz::math::Vector3d>("magnetic_field"));
      std::cout << '\n';
      element(world);
    }
    return 0;
  }
  if (argc != 1)
    return 2;
  sdf::Root root;
  auto errors = root.LoadSdfString(xml);
  std::cout << "ERRORS";
  for (const auto &error : errors)
    std::cout << ' ' << static_cast<int>(error.Code());
  std::cout << '\n';
  const auto *world = root.WorldByIndex(0);
  if (world)
  {
    std::cout << "VALUES";
    vector(world->Gravity());
    vector(world->MagneticField());
    std::cout << "\nPHYSICS " << world->PhysicsCount() << '\n';
    element(world->Element());
  }
}
