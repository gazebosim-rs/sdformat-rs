/*
 * Copyright (C) 2019 Open Source Robotics Foundation
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
#include <sdf/Lidar.hh>
#include <sdf/Element.hh>
#include <sdf/parser.hh>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
void number(double v) { uint64_t bits; std::memcpy(&bits, &v, sizeof(bits)); std::cout << ' ' << bits; }
void values(const sdf::Lidar &v) {
  std::cout << "VALUES " << v.HorizontalScanSamples();
  for (double n : {v.HorizontalScanResolution(), v.HorizontalScanMinAngle().Radian(), v.HorizontalScanMaxAngle().Radian()}) number(n);
  std::cout << ' ' << v.VerticalScanSamples();
  for (double n : {v.VerticalScanResolution(), v.VerticalScanMinAngle().Radian(), v.VerticalScanMaxAngle().Radian(), v.RangeMin(), v.RangeMax(), v.RangeResolution()}) number(n);
  std::cout << ' ' << v.VisibilityMask() << ' ' << int(v.LidarNoise().Type());
  const auto &noise = v.LidarNoise();
  for (double n : {noise.Mean(), noise.StdDev(), noise.BiasMean(), noise.BiasStdDev(), noise.DynamicBiasStdDev(), noise.DynamicBiasCorrelationTime(), noise.Precision()}) number(n);
  std::cout << '\n';
}
void errors(const char *name, const sdf::Errors &e) { std::cout << name; for (const auto &v : e) std::cout << ' ' << int(v.Code()); std::cout << '\n'; }
int main(int argc, char **argv) {
  sdf::Lidar value;
  if (argc == 2) {
    sdf::ElementPtr node;
    if (std::string(argv[1]) == "--bad") { node.reset(new sdf::Element); node->SetName("bad"); }
    else if (std::string(argv[1]) != "--null") return 2;
    errors("LOAD", value.Load(node));
    std::cout << "ELEMENT " << bool(value.Element()) << '\n'; values(value); return 0;
  }
  if (argc != 4) return 2;
  const std::string root = argv[1], version = argv[2];
  sdf::ElementPtr node(new sdf::Element);
  if (!sdf::initFile(argv[3], node)) return 3;
  node->SetName(root);
  const std::string input{std::istreambuf_iterator<char>(std::cin), {}};
  sdf::Errors e;
  bool parsed = sdf::readString("<sdf version='" + version + "'>" + input + "</sdf>", node, e);
  std::cout << "READ " << parsed << '\n'; errors("SCHEMA", e);
  if (!parsed) return 0;
  errors("LOAD", value.Load(node));
  std::cout << "ELEMENT " << bool(value.Element()) << " TAG " << node->GetName() << " VERSION " << node->OriginalVersion() << '\n';
  values(value);
}
