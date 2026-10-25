// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/World.hh>
#include <sdf/Model.hh>
#include <sdf/Link.hh>
#include <sdf/Sensor.hh>
#include <sdf/ForceTorque.hh>
#include <sdf/Joint.hh>
#include <sdf/JointAxis.hh>
#include <sdf/Physics.hh>
#include <sdf/Plugin.hh>
#include <cstring>
#include <cstdint>
#include <iostream>
#include <iterator>
#include <string>
static void hex(const std::string &value) {
  std::cout << ' ';
  if (value.empty()) std::cout << '-';
  for (unsigned char c : value) {
    const char *digits = "0123456789abcdef";
    std::cout << digits[c / 16] << digits[c % 16];
  }
}
static void number(double value) {
  std::uint64_t bits;
  std::memcpy(&bits, &value, sizeof(bits));
  std::cout << ' ' << bits;
}
int main() {
  const std::string xml((std::istreambuf_iterator<char>(std::cin)), {});
  sdf::Root root;
  const auto errors = root.LoadSdfString(xml);
  std::cout << errors.size();
  if (!errors.empty()) {
    for (const auto &error : errors) std::cout << ' ' << int(error.Code());
    std::cout << '\n'; return 0;
  }
  const auto *model = root.Model() ? root.Model() : root.WorldByIndex(0)->ModelByIndex(0);
  hex(model->Name());
  std::cout << ' ' << model->Plugins().size();
  for (const auto &plugin : model->Plugins()) { hex(plugin.Name()); hex(plugin.Filename()); }
  std::cout << ' ' << model->LinkCount();
  const auto *link = model->LinkByIndex(0);
  hex(link->Name());
  std::cout << ' ' << link->SensorCount();
  const auto *sensor = link->SensorByIndex(0);
  hex(sensor->Name()); hex(sensor->TypeStr());
  const auto *config = sensor->ForceTorqueSensor();
  std::cout << ' ' << (config != nullptr);
  if (config) std::cout << ' ' << int(config->Frame()) << ' ' << int(config->MeasureDirection());
  hex(sensor->Topic());
  std::cout << ' ' << model->JointCount();
  if (model->JointCount()) {
    const auto *axis = model->JointByIndex(0)->Axis(0);
    std::cout << ' ' << (axis != nullptr);
    if (axis) { number(axis->Damping()); number(axis->Friction()); }
  }
  std::cout << ' ' << root.WorldCount();
  if (root.WorldCount()) number(root.WorldByIndex(0)->PhysicsByIndex(0)->MaxStepSize());
  std::cout << '\n';
}
