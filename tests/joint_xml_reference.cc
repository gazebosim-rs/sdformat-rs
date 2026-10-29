// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
#include <sdf/Element.hh>
#include <sdf/Joint.hh>
#include <sdf/JointAxis.hh>
#include <sdf/SDFImpl.hh>
#include <sdf/parser.hh>

void text(const std::string &value)
{
  const char *digits = "0123456789abcdef";
  std::cout << ' ';
  if (value.empty()) std::cout << '-';
  for (unsigned char byte : value) std::cout << digits[byte >> 4] << digits[byte & 15];
}
void number(double value)
{
  uint64_t bits; std::memcpy(&bits, &value, sizeof(bits)); std::cout << ' ' << bits;
}
void axis(const sdf::JointAxis *value)
{
  std::cout << ' ' << (value != nullptr);
  if (!value) return;
  for (auto v : {value->Xyz().X(), value->Xyz().Y(), value->Xyz().Z()}) number(v);
  text(value->XyzExpressedIn());
  for (auto v : {value->Damping(), value->Friction(), value->SpringReference(), value->SpringStiffness(), value->Lower(), value->Upper(), value->Effort(), value->MaxVelocity(), value->Stiffness(), value->Dissipation()}) number(v);
  auto mimic=value->Mimic(); std::cout << ' ' << mimic.has_value();
  if (mimic) { text(mimic->Joint()); text(mimic->Axis()); number(mimic->Multiplier()); number(mimic->Offset()); number(mimic->Reference()); }
}
int main(int argc, char **argv)
{
  if (argc == 2 && std::string(argv[1]) == "--emit-subset")
  {
    sdf::Joint joint;
    joint.SetName("test_joint"); joint.SetType(sdf::JointType::BALL);
    joint.SetParentName("parent"); joint.SetChildName("child");
    joint.SetRawPose({-1,-2,-3,0,GZ_PI,0}); joint.SetPoseRelativeTo("link");
    sdf::JointAxis first, second;
    if (!first.SetXyz({1,0,0}).empty() || !second.SetXyz({0,1,0}).empty()) return 2;
    joint.SetAxis(0,first); joint.SetAxis(1,second);
    auto output=joint.ToElement(); if (!output) return 3;
    std::cout << output->ToString(""); return 0;
  }
  const std::string fragment{std::istreambuf_iterator<char>(std::cin),{}};
  sdf::SDFPtr doc(new sdf::SDF); sdf::init(doc); sdf::Errors errors;
  const bool parsed=sdf::readString("<sdf version='1.12'><model name='m'>"+fragment+"</model></sdf>",doc,errors);
  if (!parsed || !errors.empty() || !doc->Root()->GetElement("model")->HasElement("joint")) { std::cout << "REJECT\n"; return 0; }
  sdf::Joint joint; errors=joint.Load(doc->Root()->GetElement("model")->GetElement("joint"));
  if (!errors.empty()) { std::cout << "REJECT\n"; return 0; }
  std::cout << "OK";
  text(joint.Name()); std::cout << ' ' << static_cast<int>(joint.Type());
  text(joint.ParentName()); text(joint.ChildName()); text(joint.PoseRelativeTo());
  const auto pose=joint.RawPose();
  for (auto v : {pose.Pos().X(),pose.Pos().Y(),pose.Pos().Z(),pose.Rot().X(),pose.Rot().Y(),pose.Rot().Z(),pose.Rot().W()}) number(v);
  number(joint.ScrewThreadPitch()); axis(joint.Axis(0)); axis(joint.Axis(1));
  std::cout << '\n';
}
