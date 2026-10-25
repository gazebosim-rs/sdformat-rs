// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/Model.hh>
#include <sdf/Link.hh>
#include <sdf/Sensor.hh>
#include <sdf/Imu.hh>
#include <sdf/Noise.hh>
#include <sdf/SDFImpl.hh>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>

static std::uint64_t bits(double value)
{
  std::uint64_t result;
  static_assert(sizeof(result) == sizeof(value));
  std::memcpy(&result, &value, sizeof(result));
  return result;
}

int main(int argc, char **argv)
{
  if (argc == 2 && std::string(argv[1]) == "--noise-spec")
  {
    std::cout << sdf::SDF::EmbeddedSpec("noise.sdf", false);
    return 0;
  }
  if (argc != 1)
    return 2;
  std::string xml((std::istreambuf_iterator<char>(std::cin)), {});
  sdf::Root root;
  const auto errors = root.LoadSdfString(xml);
  std::cout << errors.size();
  if (!errors.empty())
  {
    for (const auto &error : errors)
      std::cout << ' ' << static_cast<int>(error.Code());
    std::cout << '\n';
    return 0;
  }
  const auto *imu = root.Model()->LinkByIndex(0)->SensorByIndex(0)->ImuSensor();
  std::cout << ' ' << (imu != nullptr);
  if (imu)
  {
    for (const auto *noise : {
        &imu->AngularVelocityXNoise(), &imu->AngularVelocityYNoise(),
        &imu->AngularVelocityZNoise(), &imu->LinearAccelerationXNoise(),
        &imu->LinearAccelerationYNoise(), &imu->LinearAccelerationZNoise()})
    {
      std::cout << ' ' << static_cast<int>(noise->Type())
        << ' ' << bits(noise->Mean()) << ' ' << bits(noise->StdDev())
        << ' ' << bits(noise->BiasMean()) << ' ' << bits(noise->BiasStdDev())
        << ' ' << bits(noise->DynamicBiasStdDev())
        << ' ' << bits(noise->DynamicBiasCorrelationTime())
        << ' ' << bits(noise->Precision());
    }
  }
  std::cout << '\n';
}
