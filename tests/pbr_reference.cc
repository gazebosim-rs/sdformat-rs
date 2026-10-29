// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
#include <sdf/Link.hh>
#include <sdf/Material.hh>
#include <sdf/Model.hh>
#include <sdf/Pbr.hh>
#include <sdf/Root.hh>
#include <sdf/Visual.hh>

void text(const std::string &value)
{
  static const char *digits = "0123456789abcdef";
  std::cout << ' ';
  if (value.empty()) std::cout << '-';
  for (unsigned char byte : value)
    std::cout << digits[byte >> 4] << digits[byte & 15];
}
void scalar(double value)
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
  if (!errors.empty()) { std::cout << '\n'; return 0; }
  const auto *material = root.Model()->LinkByIndex(0)->VisualByIndex(0)->Material();
  const auto *pbr = material->PbrMaterial();
  std::cout << ' ' << (pbr != nullptr);
  if (pbr)
    for (auto type : {sdf::PbrWorkflowType::METAL, sdf::PbrWorkflowType::SPECULAR})
    {
      const auto *flow = pbr->Workflow(type);
      std::cout << ' ' << (flow != nullptr);
      if (!flow) continue;
      std::cout << ' ' << static_cast<int>(flow->Type());
      for (const auto &value : {flow->AlbedoMap(), flow->NormalMap(),
          flow->EnvironmentMap(), flow->AmbientOcclusionMap(), flow->EmissiveMap(),
          flow->LightMap(), flow->RoughnessMap(), flow->MetalnessMap(),
          flow->SpecularMap(), flow->GlossinessMap()}) text(value);
      scalar(flow->Roughness()); scalar(flow->Metalness()); scalar(flow->Glossiness());
      std::cout << ' ' << flow->LightMapTexCoordSet();
    }
  std::cout << '\n';
}
