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
#include <sdf/Atmosphere.hh>
#include <sdf/Element.hh>
#include <sdf/SDFImpl.hh>
#include <sdf/parser.hh>
#include <gz/math/Temperature.hh>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
void values(const sdf::Atmosphere &atmosphere)
{
  std::cout << "VALUES " << int(atmosphere.Type());
  for (double value : {atmosphere.Temperature().Kelvin(),
      atmosphere.TemperatureGradient(), atmosphere.Pressure()})
  {
    uint64_t bits;
    std::memcpy(&bits, &value, sizeof(bits));
    std::cout << ' ' << bits;
  }
  std::cout << '\n';
}
void errors(const sdf::Errors &errors)
{
  for (const auto &error : errors) std::cout << ' ' << int(error.Code());
  std::cout << '\n';
}
int main(int argc, char **argv)
{
  sdf::Atmosphere atmosphere;
  if (argc == 2 && std::string(argv[1]) == "--fixture")
  {
    atmosphere.SetType(sdf::AtmosphereType::ADIABATIC);
    atmosphere.SetTemperature(gz::math::Temperature(123));
    atmosphere.SetTemperatureGradient(1.34);
    atmosphere.SetPressure(2.65);
    sdf::Errors result;
    auto element = atmosphere.ToElement(result);
    std::cout << "EMIT " << bool(element);
    errors(result);
    if (!element) return 2;
    std::cout << "XML_BEGIN\n" << element->ToString("") << "XML_END\n";
    sdf::Atmosphere loaded;
    result = loaded.Load(element);
    std::cout << "LOAD";
    errors(result);
    values(loaded);
    return 0;
  }
  const std::string input{std::istreambuf_iterator<char>(std::cin), {}};
  sdf::SDFPtr document(new sdf::SDF);
  sdf::init(document);
  sdf::Errors result;
  const bool ok = sdf::readString(
      "<sdf version='1.12'><world name='w'>" + input + "</world></sdf>",
      document, result);
  std::cout << "READ " << ok;
  errors(result);
  if (!ok) return 0;
  auto element = document->Root()->GetElement("world")->GetElement("atmosphere");
  result = atmosphere.Load(element);
  std::cout << "LOAD";
  errors(result);
  values(atmosphere);
}
