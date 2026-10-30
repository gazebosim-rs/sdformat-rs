/*
 * Copyright (C) 2018 Open Source Robotics Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
*/

// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Visual.hh>
#include <sdf/Geometry.hh>
#include <sdf/Material.hh>
#include <sdf/Plugin.hh>
#include <sdf/Element.hh>
#include <sdf/SDFImpl.hh>
#include <sdf/parser.hh>
#include <iostream>
#include <iterator>
#include <cstring>
#include <cstdint>
#include <iomanip>

void number(double value)
{
  uint64_t bits;
  std::memcpy(&bits, &value, sizeof(bits));
  std::cout << ' ' << bits;
}
void state(const sdf::Visual &v)
{
  std::cout << "VALUES " << std::quoted(v.Name()) << ' '
            << std::quoted(v.PoseRelativeTo());
  const auto pose = v.RawPose();
  for (double value : {pose.Pos().X(), pose.Pos().Y(), pose.Pos().Z(),
      pose.Rot().X(), pose.Rot().Y(), pose.Rot().Z(), pose.Rot().W()})
    number(value);
  std::cout << ' ' << v.CastShadows();
  number(v.Transparency());
  std::cout << ' ' << v.VisibilityFlags() << ' ' << v.HasLaserRetro();
  number(v.LaserRetro());
  std::cout << ' ' << bool(v.Geom()) << ' ' << int(v.Geom()->Type())
            << ' ' << bool(v.Material()) << ' ' << v.Plugins().size();
  for (const auto &p : v.Plugins())
    std::cout << ' ' << std::quoted(p.Name()) << ' ' << std::quoted(p.Filename());
  std::cout << '\n';
}
void errors(const sdf::Errors &errors)
{
  for (const auto &error : errors) std::cout << ' ' << int(error.Code());
  std::cout << '\n';
}
int main(int argc, char **argv)
{
  sdf::Visual v;
  if (argc > 1 && std::string(argv[1]) == "ctor")
  {
    v.SetName("my-visual");
    v.SetCastShadows(true);
    v.SetTransparency(.2f);
    v.SetRawPose(gz::math::Pose3d(1, 2, 3, .1, .2, .3));
    v.SetVisibilityFlags(1234u);
    v.SetHasLaserRetro(true);
    v.SetLaserRetro(1.2);
    v.SetGeom(sdf::Geometry());
    v.SetMaterial(sdf::Material());
    sdf::Plugin plugin;
    plugin.SetName("name1");
    plugin.SetFilename("filename1");
    v.AddPlugin(plugin);
    state(v);
    auto element = v.ToElement();
    std::cout << "XML_BEGIN\n" << element->ToString("") << "XML_END\n";
    sdf::Visual loaded;
    const auto result = loaded.Load(element);
    std::cout << "LOAD";
    errors(result);
    state(loaded);
    return 0;
  }
  const std::string xml{std::istreambuf_iterator<char>(std::cin), {}};
  sdf::SDFPtr doc(new sdf::SDF);
  sdf::init(doc);
  sdf::Errors result;
  const bool ok = sdf::readString(
      "<sdf version='1.12'><model name='m'><link name='l'>" + xml +
      "</link></model></sdf>", doc, result);
  std::cout << "READ " << ok;
  errors(result);
  if (!ok) return 0;
  auto element = doc->Root()->GetElement("model")->GetElement("link")
      ->GetElement("visual");
  result = v.Load(element);
  std::cout << "LOAD";
  errors(result);
  state(v);
}
