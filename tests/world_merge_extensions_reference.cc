// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/World.hh>
#include <sdf/Element.hh>
#include <iostream>

int main(int, char **argv)
{
  sdf::Root root;
  const auto errors = root.Load(argv[1]);
  for (const auto &error : errors)
    std::cerr << int(error.Code()) << " " << error.Message() << '\n';
  const auto *world = root.WorldByIndex(0);
  if (!world)
  {
    std::cout << "NO_WORLD\n";
    return 0;
  }
  std::cout << "models=" << world->ModelCount() << " frames=" << world->FrameCount() << '\n';
  if (world->Element())
    std::cout << world->Element()->ToString("");
}
