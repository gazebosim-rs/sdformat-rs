// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Light.hh>
#include <sdf/World.hh>
#include <gz/math/Pose3.hh>
#include <iostream>
#include <string>

int main()
{
  // Collection mutation compares names only, unlike XML admission.
  for (const std::string name : {"", "light", "Light", "__world__", "a::b", "a b"})
  {
    sdf::World world;
    sdf::Light light;
    light.SetName(name);
    light.SetIntensity(3);
    light.SetRawPose(gz::math::Pose3d(1, 2, 3, 0, 0, 0));
    if (!world.AddLight(light) || world.LightCount() != 1) return 1;
    light.SetIntensity(9);
    if (world.AddLight(light) || world.LightCount() != 1) return 2;
    const auto *stored = world.LightByIndex(0);
    if (!stored || stored->Name() != name || stored->Intensity() != 3 ||
        stored->RawPose() != gz::math::Pose3d(1, 2, 3, 0, 0, 0)) return 3;
    world.ClearLights();
    if (world.LightCount() != 0 || world.LightByIndex(0)) return 4;
    if (!world.AddLight(light) || world.LightByIndex(0)->Intensity() != 9) return 5;
  }
  sdf::World world;
  sdf::Light light;
  light.SetName("light");
  if (!world.AddLight(light)) return 6;
  light.SetName("Light");
  if (!world.AddLight(light) || world.LightCount() != 2) return 7;
  world.LightByIndex(0)->SetName("renamed");
  if (!world.LightNameExists("renamed") || world.LightNameExists("light")) return 8;
  light.SetName("light");
  if (!world.AddLight(light) || world.LightCount() != 3) return 9;
  light.SetName("renamed");
  if (world.AddLight(light)) return 10;
  auto copy = world;
  copy.LightByIndex(0)->SetIntensity(12);
  copy.ClearLights();
  if (world.LightCount() != 3 || world.LightByIndex(0)->Intensity() != 1 ||
      world.LightByIndex(0)->Name() != "renamed" ||
      world.LightByIndex(1)->Name() != "Light" ||
      world.LightByIndex(2)->Name() != "light") return 11;
  std::cout << "6 name domains: insert duplicate first-value copy clear reinsert; "
               "case-sensitive order rename relookup world-copy independence\n";
}
