// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/ParserConfig.hh>
#include <sdf/Model.hh>
#include <sdf/Joint.hh>
#include <sdf/Link.hh>
#include <sdf/Sensor.hh>
#include <sdf/ForceTorque.hh>
#include <sdf/Element.hh>
#include <sdf/SDFImpl.hh>
#include <sdf/SemanticPose.hh>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
static std::uint64_t bits(double value) {
  std::uint64_t result;
  static_assert(sizeof(result) == sizeof(value));
  std::memcpy(&result, &value, sizeof(result));
  return result;
}
static void hex(const std::string &value) {
  std::cout << ' ';
  if (value.empty()) { std::cout << '-'; return; }
  for (unsigned char c : value) {
    const char *digits = "0123456789abcdef";
    std::cout << digits[c / 16] << digits[c % 16];
  }
}
static void pose(const gz::math::Pose3d &p) {
  for (double value : {p.Pos().X(), p.Pos().Y(), p.Pos().Z(),
       p.Rot().X(), p.Rot().Y(), p.Rot().Z(), p.Rot().W()})
    std::cout << ' ' << bits(value);
}
int main(int argc, char **argv) {
  if (argc == 3 && std::string(argv[1]) == "--spec") {
    std::cout << sdf::SDF::EmbeddedSpec(argv[2], false);
    return 0;
  }
  if (argc != 1 && argc != 2) return 2;
  std::string xml((std::istreambuf_iterator<char>(std::cin)), {});
  sdf::Root root;
  sdf::ParserConfig config;
  if (argc == 2) {
    const std::string resource = argv[1];
    config.SetFindCallback([resource](const std::string &) { return resource; });
  }
  const auto errors = root.LoadSdfString(xml, config);
  std::cout << errors.size();
  if (!errors.empty()) {
    for (const auto &error : errors) std::cout << ' ' << int(error.Code());
    std::cout << '\n'; return 0;
  }
  const auto *model = root.Model();
  if (!model->JointCount() && !model->LinkCount() && model->ModelCount())
    model = model->ModelByIndex(0);
  const bool onJoint = model->JointCount() && model->JointByIndex(0)->SensorCount();
  const auto *sensor = onJoint ? model->JointByIndex(0)->SensorByIndex(0) : model->LinkByIndex(0)->SensorByIndex(0);
  hex(sensor->TypeStr());
  const auto *forceTorque = sensor->ForceTorqueSensor();
  std::cout << ' ' << (forceTorque != nullptr) << ' ' << (sensor->ImuSensor() != nullptr)
    << ' ' << (onJoint ? model->JointByIndex(0)->SensorCount() : model->LinkByIndex(0)->SensorCount());
  if (forceTorque) {
    std::cout << ' ' << int(forceTorque->Frame()) << ' ' << int(forceTorque->MeasureDirection());
    for (const auto *noise : {&forceTorque->ForceXNoise(), &forceTorque->ForceYNoise(),
         &forceTorque->ForceZNoise(), &forceTorque->TorqueXNoise(), &forceTorque->TorqueYNoise(), &forceTorque->TorqueZNoise()}) {
      std::cout << ' ' << int(noise->Type());
      for (double value : {noise->Mean(), noise->StdDev(), noise->BiasMean(), noise->BiasStdDev(),
           noise->DynamicBiasStdDev(), noise->DynamicBiasCorrelationTime(), noise->Precision()})
        std::cout << ' ' << bits(value);
    }
  }
  std::cout << ' ' << bits(sensor->UpdateRate()) << ' '
    << sensor->Element()->Get<bool>("always_on",false).first << ' ' << sensor->EnableMetrics();
  hex(sensor->Topic()); hex(sensor->FrameId());
  pose(sensor->RawPose());
  for (const std::string target : {"__model__", ""}) {
    gz::math::Pose3d resolved;
    const auto errors = sensor->SemanticPose().Resolve(resolved, target);
    std::cout << ' ' << errors.size();
    if (errors.empty()) pose(resolved);
    else for (const auto &error : errors) std::cout << ' ' << int(error.Code());
  }
  std::cout << '\n';
}
