// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Element.hh>
#include <sdf/NavSat.hh>
#include <sdf/Lidar.hh>
#include <cstdint>
#include <cstring>
#include <sdf/SDFImpl.hh>
#include <sdf/Sensor.hh>
#include <sdf/parser.hh>
#include <iostream>
#include <iterator>
#include <string>
void number(double value) {uint64_t bits; std::memcpy(&bits,&value,sizeof(bits)); std::cout << ' ' << bits;}
void text(const std::string &value) {
  std::cout << ' '; const char *digits="0123456789abcdef";
  if(value.empty()) std::cout << '-';
  for(unsigned char c:value) std::cout << digits[c>>4] << digits[c&15];
}
void navsat_config(const sdf::NavSat &value)
{
  for (const auto &noise : {value.HorizontalPositionNoise(), value.VerticalPositionNoise(), value.HorizontalVelocityNoise(), value.VerticalVelocityNoise()})
  {
    std::cout << ' ' << static_cast<int>(noise.Type());
    for (double v : {noise.Mean(), noise.StdDev(), noise.BiasMean(), noise.BiasStdDev(),
        noise.DynamicBiasStdDev(), noise.DynamicBiasCorrelationTime(), noise.Precision()})
      number(v);
  }
}
void lidar_config(const sdf::Lidar &value)
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

int main()
{
  const std::string fragment{std::istreambuf_iterator<char>(std::cin), {}};
  sdf::SDFPtr document(new sdf::SDF);
  sdf::init(document);
  sdf::Errors errors;
  if (!sdf::readString("<sdf version='1.12'><model name='m'><link name='l'>"+
      fragment+"</link></model></sdf>", document, errors) || !errors.empty()) return 2;
  auto element = document->Root()->GetElement("model")->GetElement("link")->GetElement("sensor");
  sdf::Sensor value;
  errors = value.Load(element);
  if (!errors.empty()) return 3;
  std::cout << "OK";
  text(value.Name()); text(value.Element()->Get<std::string>("type")); text(value.TypeStr());
  text(value.FrameId()); text(value.Topic()); text(value.PoseRelativeTo()); number(value.UpdateRate());
  std::cout << ' ' << value.EnableMetrics() << ' ' << value.Element()->Get<bool>("always_on")
            << ' ' << value.Element()->Get<bool>("visualize");
  const auto pose=value.RawPose();
  for(double v:{pose.Pos().X(),pose.Pos().Y(),pose.Pos().Z(),pose.Rot().X(),pose.Rot().Y(),pose.Rot().Z(),pose.Rot().W()}) number(v);
  if(value.NavSatSensor()) navsat_config(*value.NavSatSensor());
  else if(value.LidarSensor()) lidar_config(*value.LidarSensor());
  else return 4;
  std::cout << '\n';
}
