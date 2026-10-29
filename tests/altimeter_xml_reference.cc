// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
#include <sdf/Altimeter.hh>
#include <sdf/Element.hh>
#include <sdf/Noise.hh>
#include <sdf/SDFImpl.hh>
#include <sdf/Sensor.hh>
#include <sdf/parser.hh>

void number(double value)
{
  uint64_t bits;
  std::memcpy(&bits, &value, sizeof(bits));
  std::cout << ' ' << bits;
}
void text(const std::string &value)
{
  const char *digits = "0123456789abcdef";
  std::cout << ' ';
  if (value.empty()) std::cout << '-';
  for (unsigned char c : value) std::cout << digits[c >> 4] << digits[c & 15];
}
void config(const sdf::Altimeter &value)
{
  for (const auto &noise : {value.VerticalPositionNoise(), value.VerticalVelocityNoise()})
  {
    std::cout << ' ' << static_cast<int>(noise.Type());
    for (double v : {noise.Mean(), noise.StdDev(), noise.BiasMean(), noise.BiasStdDev(),
        noise.DynamicBiasStdDev(), noise.DynamicBiasCorrelationTime(), noise.Precision()})
      number(v);
  }
}
void sensor(const sdf::Sensor &value)
{
  text(value.Name()); text(value.TypeStr()); text(value.FrameId());
  text(value.Topic()); text(value.PoseRelativeTo()); number(value.UpdateRate());
  std::cout << ' ' << value.EnableMetrics()
            << ' ' << value.Element()->Get<bool>("always_on")
            << ' ' << value.Element()->Get<bool>("visualize");
  auto pose = value.RawPose();
  for (double v : {pose.Pos().X(), pose.Pos().Y(), pose.Pos().Z(),
      pose.Rot().X(), pose.Rot().Y(), pose.Rot().Z(), pose.Rot().W()}) number(v);
  if (!value.AltimeterSensor()) { std::cout << " NO_CONFIG"; return; }
  config(*value.AltimeterSensor());
}
int main(int argc, char **argv)
{
  if (argc != 2) return 2;
  const std::string mode = argv[1];
  if (mode == "--emit-original")
  {
    sdf::Altimeter value;
    sdf::Noise noise;
    noise.SetType(sdf::NoiseType::GAUSSIAN);
    noise.SetMean(1.2); noise.SetStdDev(2.3); noise.SetBiasMean(4.5);
    noise.SetBiasStdDev(6.7); noise.SetPrecision(8.9);
    value.SetVerticalPositionNoise(noise); value.SetVerticalVelocityNoise(noise);
    sdf::Errors errors;
    auto node = value.ToElement(errors);
    if (!errors.empty() || !node) return 3;
    std::cout << node->ToString("");
    return 0;
  }
  if (mode != "config" && mode != "sensor") return 4;
  std::string fragment{std::istreambuf_iterator<char>(std::cin), {}};
  if (mode == "config") fragment = "<sensor name='s' type='altimeter'>" + fragment + "</sensor>";
  sdf::SDFPtr document(new sdf::SDF);
  sdf::init(document);
  sdf::Errors errors;
  bool parsed = sdf::readString("<sdf version='1.12'><model name='m'><link name='l'>" +
      fragment + "</link></model></sdf>", document, errors);
  if (!parsed || !errors.empty()) { std::cout << "REJECT\n"; return 0; }
  auto link = document->Root()->GetElement("model")->GetElement("link");
  if (!link->HasElement("sensor")) { std::cout << "REJECT\n"; return 0; }
  auto node = link->GetElement("sensor");
  if (mode == "config")
  {
    if (!node->HasElement("altimeter")) { std::cout << "REJECT\n"; return 0; }
    sdf::Altimeter value;
    errors = value.Load(node->GetElement("altimeter"));
    if (!errors.empty()) { std::cout << "REJECT\n"; return 0; }
    std::cout << "OK"; config(value);
  }
  else
  {
    sdf::Sensor value;
    errors = value.Load(node);
    if (!errors.empty()) { std::cout << "REJECT\n"; return 0; }
    std::cout << "OK"; sensor(value);
  }
  std::cout << '\n';
}
