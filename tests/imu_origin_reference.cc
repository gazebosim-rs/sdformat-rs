// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Imu.hh>
#include <sdf/Element.hh>
#include <sdf/Sensor.hh>
#include <sdf/SDFImpl.hh>
#include <sdf/parser.hh>
#include <iostream>
int constructed() {
  sdf::Imu value;
  std::cout << bool(value.Element()) << ' ';
  sdf::Errors errors;
  auto emitted = value.ToElement(errors);
  if (!errors.empty() || !emitted) return 1;
  std::cout << bool(value.Element()) << ' ';
  emitted->SetFilePath("input-imu.sdf");
  emitted->SetLineNumber(7);
  sdf::Imu loaded;
  if (!loaded.Load(emitted).empty()) return 2;
  std::cout << bool(loaded.Element()) << ' ' << loaded.Element()->FilePath()
            << ' ' << loaded.Element()->LineNumber().value_or(-1) << ' ';
  loaded.SetLocalization("NED");
  auto edited = loaded.ToElement(errors);
  if (!errors.empty() || !edited) return 3;
  std::cout << bool(loaded.Element()) << ' ' << loaded.Element()->FilePath()
            << ' ' << loaded.Element()->LineNumber().value_or(-1) << '\n';
  return 0;
}
int main() {
  std::cout << "constructed ";
  if (int result = constructed()) return result;
  for (auto config : {"", "<imu/>"}) {
    sdf::SDFPtr document(new sdf::SDF);
    sdf::init(document); sdf::Errors errors;
    std::string xml="<sdf version='1.12'><model name='m'><link name='l'><sensor name='s' type='imu'>";
    if (!sdf::readString(xml+config+"</sensor></link></model></sdf>", document, errors) || !errors.empty()) return 4;
    auto element=document->Root()->GetElement("model")->GetElement("link")->GetElement("sensor");
    sdf::Sensor sensor; if (!sensor.Load(element).empty()) return 5;
    auto imu=sensor.ImuSensor();
    std::cout << (config[0]?"explicit":"omitted") << ' ' << bool(imu) << ' ' << bool(imu && imu->Element()) << '\n';
  }
}
