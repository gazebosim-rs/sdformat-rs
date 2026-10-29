// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
#include <sdf/Lidar.hh>
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
void config(const sdf::Lidar &value)
{
  std::cout << ' ' << value.HorizontalScanSamples();
  for (double v : {value.HorizontalScanResolution(), value.HorizontalScanMinAngle().Radian(),
      value.HorizontalScanMaxAngle().Radian()}) number(v);
  std::cout << ' ' << value.VerticalScanSamples();
  for (double v : {value.VerticalScanResolution(), value.VerticalScanMinAngle().Radian(),
      value.VerticalScanMaxAngle().Radian(), value.RangeMin(), value.RangeMax(),
      value.RangeResolution()}) number(v);
  std::cout << ' ' << value.VisibilityMask();
  const auto &noise = value.LidarNoise();
  std::cout << ' ' << static_cast<int>(noise.Type());
  for (double v : {noise.Mean(), noise.StdDev(), noise.BiasMean(), noise.BiasStdDev(),
      noise.DynamicBiasStdDev(), noise.DynamicBiasCorrelationTime(), noise.Precision()}) number(v);
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
  if (!value.LidarSensor()) { std::cout << " NO_CONFIG"; return; }
  config(*value.LidarSensor());
}
int main(int argc, char **argv)
{
  if (argc != 2) return 2;
  const std::string mode = argv[1];
  if (mode == "--emit-original")
  {
    sdf::Lidar value;
    value.SetHorizontalScanSamples(123); value.SetHorizontalScanResolution(0.45);
    value.SetHorizontalScanMinAngle(gz::math::Angle(0.67));
    value.SetHorizontalScanMaxAngle(gz::math::Angle(0.89));
    value.SetVerticalScanSamples(98); value.SetVerticalScanResolution(0.76);
    value.SetVerticalScanMinAngle(gz::math::Angle(0.54));
    value.SetVerticalScanMaxAngle(gz::math::Angle(0.321));
    value.SetRangeMin(1.2); value.SetRangeMax(3.4); value.SetRangeResolution(5.6);
    value.SetVisibilityMask(123);
    sdf::Noise noise; noise.SetMean(6.5); noise.SetStdDev(3.79);
    value.SetLidarNoise(noise);
    auto node = value.ToElement();
    if (!node) return 3;
    std::cout << node->ToString("");
    return 0;
  }
  if (mode != "config" && mode != "sensor") return 4;
  std::string fragment{std::istreambuf_iterator<char>(std::cin), {}};
  if (mode == "config") fragment = "<sensor name='s' type='lidar'>" + fragment + "</sensor>";
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
    if (!node->HasElement("lidar")) { std::cout << "REJECT\n"; return 0; }
    sdf::Lidar value;
    errors = value.Load(node->GetElement("lidar"));
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
