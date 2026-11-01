/*
 * Copyright 2012 Open Source Robotics Foundation
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
#include <sdf/Root.hh>
#include <sdf/ParserConfig.hh>
#include <sdf/World.hh>
#include <sdf/Model.hh>
#include <sdf/Link.hh>
#include <sdf/Joint.hh>
#include <sdf/Sensor.hh>
#include <sdf/Imu.hh>
#include <sdf/Element.hh>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>

void number(double value)
{
  uint64_t bits;
  std::memcpy(&bits, &value, sizeof(bits));
  std::cout << ' ' << bits;
}
void text(const std::string &value)
{
  const char *hex = "0123456789abcdef";
  std::cout << ' ';
  if (value.empty()) std::cout << '-';
  for (unsigned char c : value) std::cout << hex[c >> 4] << hex[c & 15];
}
void sensor(const sdf::Sensor &value)
{
  std::cout << "SENSOR " << value.Name() << ' ' << value.TypeStr() << ' '
            << bool(value.ImuSensor()) << '\n';
  std::cout << "ENVELOPE";
  text(value.Topic());
  text(value.FrameId());
  number(value.UpdateRate());
  std::cout << ' ' << value.EnableMetrics() << ' '
            << value.Element()->Get<bool>("always_on") << ' '
            << value.Element()->Get<bool>("visualize") << '\n';
  const auto *imu = value.ImuSensor();
  if (!imu) return;
  std::cout << "NOISE";
  for (const auto &noise : {imu->AngularVelocityXNoise(),
      imu->AngularVelocityYNoise(), imu->AngularVelocityZNoise(),
      imu->LinearAccelerationXNoise(), imu->LinearAccelerationYNoise(),
      imu->LinearAccelerationZNoise()})
  {
    std::cout << ' ' << int(noise.Type());
    for (double x : {noise.Mean(), noise.StdDev(), noise.BiasMean(),
        noise.BiasStdDev(), noise.DynamicBiasStdDev(),
        noise.DynamicBiasCorrelationTime(), noise.Precision()}) number(x);
  }
  std::cout << "\nORIENTATION " << imu->OrientationEnabled();
  text(imu->Localization());
  const auto rpy = imu->CustomRpy();
  for (double x : {rpy.X(), rpy.Y(), rpy.Z()}) number(x);
  text(imu->CustomRpyParentFrame());
  const auto gravity = imu->GravityDirX();
  for (double x : {gravity.X(), gravity.Y(), gravity.Z()}) number(x);
  text(imu->GravityDirXParentFrame());
  std::cout << '\n';
}
void model(const sdf::Model &value)
{
  std::cout << "MODEL " << value.Name() << '\n';
  for (uint64_t i = 0; i < value.LinkCount(); ++i)
  {
    const auto *link = value.LinkByIndex(i);
    std::cout << "LINK " << link->Name() << '\n';
    for (uint64_t j = 0; j < link->SensorCount(); ++j)
      sensor(*link->SensorByIndex(j));
  }
  for (uint64_t i = 0; i < value.JointCount(); ++i)
  {
    const auto *joint = value.JointByIndex(i);
    for (uint64_t j = 0; j < joint->SensorCount(); ++j)
      sensor(*joint->SensorByIndex(j));
  }
  for (uint64_t i = 0; i < value.ModelCount(); ++i)
    model(*value.ModelByIndex(i));
}
int main(int argc, char **argv)
{
  if (argc > 2) return 2;
  const std::string xml{std::istreambuf_iterator<char>(std::cin), {}};
  sdf::ParserConfig config;
  if (argc == 2)
    config.SetFindCallback([&](const std::string &uri) {
      return uri == "fixture.sdf" ? std::string(argv[1]) : std::string{};
    });
  sdf::Root root;
  const auto errors = root.LoadSdfString(xml, config);
  std::cout << "ERRORS";
  for (const auto &error : errors) std::cout << ' ' << int(error.Code());
  std::cout << '\n';
  if (root.Model()) model(*root.Model());
  for (uint64_t i = 0; i < root.WorldCount(); ++i)
  {
    const auto *world = root.WorldByIndex(i);
    std::cout << "WORLD " << world->Name() << '\n';
    for (uint64_t j = 0; j < world->ModelCount(); ++j)
      model(*world->ModelByIndex(j));
  }
}
