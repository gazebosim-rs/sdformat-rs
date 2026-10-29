// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
#include <sdf/Element.hh>
#include <sdf/ForceTorque.hh>
#include <sdf/Joint.hh>
#include <sdf/Noise.hh>
#include <sdf/SDFImpl.hh>
#include <sdf/Sensor.hh>
#include <sdf/parser.hh>
void text(const std::string &value)
{
  std::cout << ' ';const char *digits="0123456789abcdef";
  if (value.empty()) std::cout << '-';
  for (unsigned char c:value) std::cout << digits[c>>4] << digits[c&15];
}
void number(double value) { uint64_t bits;std::memcpy(&bits,&value,sizeof(bits));std::cout << ' ' << bits; }
void config(const sdf::ForceTorque &value)
{
  std::cout << ' ' << static_cast<int>(value.Frame()) << ' ' << static_cast<int>(value.MeasureDirection());
  for (const auto &noise:{value.ForceXNoise(),value.ForceYNoise(),value.ForceZNoise(),value.TorqueXNoise(),value.TorqueYNoise(),value.TorqueZNoise()})
  {
    std::cout << ' ' << static_cast<int>(noise.Type());
    for(double v:{noise.Mean(),noise.StdDev(),noise.BiasMean(),noise.BiasStdDev(),noise.DynamicBiasStdDev(),noise.DynamicBiasCorrelationTime(),noise.Precision()}) number(v);
  }
}
void sensor(const sdf::Sensor &value)
{
  text(value.Name());text(value.TypeStr());text(value.FrameId());text(value.Topic());text(value.PoseRelativeTo());number(value.UpdateRate());
  std::cout << ' ' << value.EnableMetrics() << ' ' << value.Element()->Get<bool>("always_on") << ' ' << value.Element()->Get<bool>("visualize");
  const auto pose=value.RawPose();for(double v:{pose.Pos().X(),pose.Pos().Y(),pose.Pos().Z(),pose.Rot().X(),pose.Rot().Y(),pose.Rot().Z(),pose.Rot().W()}) number(v);
  if(!value.ForceTorqueSensor()) {std::cout << " NO_CONFIG";return;}
  config(*value.ForceTorqueSensor());
}
int main(int argc,char **argv)
{
  if(argc!=2)return 2;const std::string mode=argv[1];
  if(mode=="--emit-original")
  {
    sdf::ForceTorque value;sdf::Noise noise;noise.SetType(sdf::NoiseType::GAUSSIAN);noise.SetMean(1.2);noise.SetStdDev(2.3);noise.SetBiasMean(4.5);noise.SetBiasStdDev(6.7);noise.SetPrecision(8.9);
    value.SetForceXNoise(noise);value.SetForceYNoise(noise);value.SetForceZNoise(noise);value.SetTorqueXNoise(noise);value.SetTorqueYNoise(noise);value.SetTorqueZNoise(noise);
    value.SetFrame(sdf::ForceTorqueFrame::PARENT);value.SetMeasureDirection(sdf::ForceTorqueMeasureDirection::PARENT_TO_CHILD);
    sdf::Errors errors;auto node=value.ToElement(errors);if(!errors.empty()||!node)return 3;std::cout << node->ToString("");return 0;
  }
  std::string fragment{std::istreambuf_iterator<char>(std::cin),{}};
  if(mode=="config")fragment="<sensor name='s' type='force_torque'>"+fragment+"</sensor>";
  if(mode!="joint")fragment="<joint name='j' type='fixed'><parent>a</parent><child>b</child>"+fragment+"</joint>";
  sdf::SDFPtr document(new sdf::SDF);sdf::init(document);sdf::Errors errors;
  bool parsed=sdf::readString("<sdf version='1.12'><model name='m'>"+fragment+"</model></sdf>",document,errors);
  if(!parsed||!errors.empty()){std::cout << "REJECT\n";return 0;}
  auto modelNode=document->Root()->GetElement("model");
  if(!modelNode->HasElement("joint")){std::cout << "REJECT\n";return 0;}
  auto jointNode=modelNode->GetElement("joint");
  if(mode!="joint" && !jointNode->HasElement("sensor")){std::cout << "REJECT\n";return 0;}
  if(mode=="config" && !jointNode->GetElement("sensor")->HasElement("force_torque")){std::cout << "REJECT\n";return 0;}
  if(mode=="config")
  {
    sdf::ForceTorque value;errors=value.Load(jointNode->GetElement("sensor")->GetElement("force_torque"));
    if(!errors.empty()){std::cout << "REJECT\n";return 0;}std::cout << "OK";config(value);
  }
  else if(mode=="sensor")
  {
    sdf::Sensor value;errors=value.Load(jointNode->GetElement("sensor"));if(!errors.empty()){std::cout << "REJECT\n";return 0;}
    std::cout << "OK";sensor(value);
  }
  else if(mode=="joint")
  {
    sdf::Joint value;errors=value.Load(jointNode);if(!errors.empty()){std::cout << "REJECT\n";return 0;}
    std::cout << "OK " << value.SensorCount();for(uint64_t i=0;i<value.SensorCount();++i)sensor(*value.SensorByIndex(i));
  }
  else return 4;
  std::cout << '\n';
}
