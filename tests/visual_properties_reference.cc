// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/Model.hh>
#include <sdf/Link.hh>
#include <sdf/Visual.hh>
#include <cfenv>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
int main()
{
  if (std::fegetround()!=FE_TONEAREST) return 8;
  const std::string xml((std::istreambuf_iterator<char>(std::cin)), {});
  sdf::Root root;
  if (!root.LoadSdfString(xml).empty()) { std::cout << "error\n"; return 0; }
  const auto *visual=root.Model()->LinkByIndex(0)->VisualByIndex(0);
  const float transparency=visual->Transparency();
  const double retro=visual->LaserRetro();
  std::uint32_t transparencyBits;
  std::uint64_t retroBits;
  static_assert(sizeof(transparencyBits)==sizeof(transparency));
  static_assert(sizeof(retroBits)==sizeof(retro));
  std::memcpy(&transparencyBits,&transparency,sizeof(transparency));
  std::memcpy(&retroBits,&retro,sizeof(retro));
  std::cout << visual->CastShadows() << ' ' << transparencyBits << ' '
    << visual->VisibilityFlags() << ' ' << visual->HasLaserRetro() << ' ' << retroBits << '\n';
}
