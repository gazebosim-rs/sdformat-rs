// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
#include <sdf/Camera.hh>
#include <sdf/Element.hh>
#include <sdf/Noise.hh>
#include <sdf/SDFImpl.hh>
#include <sdf/Sensor.hh>
#include <sdf/parser.hh>
void text(const std::string &value)
{std::cout << ' ';const char *digits="0123456789abcdef";if(value.empty())std::cout << '-';for(unsigned char c:value)std::cout << digits[c>>4] << digits[c&15];}
void number(double value) {uint64_t bits;std::memcpy(&bits,&value,sizeof(bits));std::cout << ' ' << bits;}
void camera(const sdf::Camera &value)
{
  text(value.Name());text(value.PoseRelativeTo());text(value.SaveFramesPath());text(value.CameraInfoTopic());text(value.TriggerTopic());text(value.PixelFormatStr());
  std::cout << ' ' << value.Triggered() << ' ' << value.SaveFrames() << ' ' << value.ImageWidth() << ' ' << value.ImageHeight() << ' ' << value.AntiAliasingValue() << ' ' << value.VisibilityMask();
  for(double v:{value.HorizontalFov().Radian(),value.NearClip(),value.FarClip()})number(v);
  const auto pose=value.RawPose();for(double v:{pose.Pos().X(),pose.Pos().Y(),pose.Pos().Z(),pose.Rot().X(),pose.Rot().Y(),pose.Rot().Z(),pose.Rot().W()})number(v);
  std::cout << ' ' << value.HasDepthNearClip() << ' ' << value.HasDepthFarClip();number(value.DepthNearClip());number(value.DepthFarClip());
  std::cout << ' ' << value.HasSegmentationType() << ' ' << value.HasBoundingBoxType();text(value.SegmentationType());text(value.BoundingBoxType());
  auto noise=value.ImageNoise();std::cout << ' ' << static_cast<int>(noise.Type());for(double v:{noise.Mean(),noise.StdDev(),noise.BiasMean(),noise.BiasStdDev(),noise.DynamicBiasStdDev(),noise.DynamicBiasCorrelationTime(),noise.Precision()})number(v);
  for(double v:{value.DistortionK1(),value.DistortionK2(),value.DistortionK3(),value.DistortionP1(),value.DistortionP2(),value.DistortionCenter().X(),value.DistortionCenter().Y()})number(v);
  text(value.LensType());text(value.LensFunction());std::cout << ' ' << value.LensScaleToHfov() << ' ' << value.LensEnvironmentTextureSize() << ' ' << value.HasLensIntrinsics() << ' ' << value.HasLensProjection();
  for(double v:{value.LensC1(),value.LensC2(),value.LensC3(),value.LensFocalLength(),value.LensCutoffAngle().Radian(),value.LensIntrinsicsFx(),value.LensIntrinsicsFy(),value.LensIntrinsicsCx(),value.LensIntrinsicsCy(),value.LensIntrinsicsSkew(),value.LensProjectionFx(),value.LensProjectionFy(),value.LensProjectionCx(),value.LensProjectionCy(),value.LensProjectionTx(),value.LensProjectionTy()})number(v);
}
void sensor(const sdf::Sensor &value)
{
  text(value.Name()); text(value.TypeStr()); text(value.FrameId());
  text(value.Topic()); text(value.PoseRelativeTo()); number(value.UpdateRate());
  std::cout << ' ' << value.EnableMetrics()
            << ' ' << value.Element()->Get<bool>("always_on")
            << ' ' << value.Element()->Get<bool>("visualize");
  const auto pose = value.RawPose();
  for (double v : {pose.Pos().X(), pose.Pos().Y(), pose.Pos().Z(),
      pose.Rot().X(), pose.Rot().Y(), pose.Rot().Z(), pose.Rot().W()}) number(v);
  camera(*value.CameraSensor());
}
int main(int argc, char **argv)
{
  if (argc > 2 || (argc == 2 && std::string(argv[1]) != "--document")) return 2;
  const std::string fragment{std::istreambuf_iterator<char>(std::cin), {}};
  const std::string input = argc == 2 ? fragment :
      "<sdf version='1.12'><model name='m'><link name='l'>"+
      fragment+"</link></model></sdf>";
  sdf::SDFPtr document(new sdf::SDF);
  sdf::init(document);
  sdf::Errors errors;
  if (!sdf::readString(input, document, errors) || !errors.empty())
  { std::cout << "REJECT\n"; return 0; }
  auto link = document->Root()->GetElement("model")->GetElement("link");
  if (!link->HasElement("sensor")) { std::cout << "REJECT\n"; return 0; }
  sdf::Sensor value;
  errors = value.Load(link->GetElement("sensor"));
  if (!errors.empty() || !value.CameraSensor()) { std::cout << "REJECT\n"; return 0; }
  std::cout << "OK"; sensor(value); std::cout << '\n';
}
