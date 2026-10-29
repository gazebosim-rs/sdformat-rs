// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
#include <sdf/Element.hh>
#include <sdf/JointAxis.hh>
#include <sdf/SDFImpl.hh>
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
  for (unsigned char byte : value)
    std::cout << digits[byte >> 4] << digits[byte & 15];
}
int main(int argc, char **argv)
{
  if (argc == 2 && std::string(argv[1]) == "--emit-original")
  {
    sdf::JointAxis axis;
    if (!axis.SetXyz({0, 1, 0}).empty()) return 2;
    axis.SetXyzExpressedIn("test");
    axis.SetDamping(0.2); axis.SetFriction(1.3);
    axis.SetSpringReference(2.4); axis.SetSpringStiffness(-1.2);
    axis.SetLower(-10.8); axis.SetUpper(123.4);
    axis.SetEffort(3.2); axis.SetMaxVelocity(54.2);
    axis.SetStiffness(1e2); axis.SetDissipation(1.5);
    axis.SetMimic(sdf::MimicConstraint("test_joint", "axis2", 5, 1, 2));
    sdf::Errors errors;
    auto element = axis.ToElement(errors);
    if (!errors.empty() || !element) return 3;
    std::cout << element->ToString("");
    return 0;
  }
  const std::string axisXml{std::istreambuf_iterator<char>(std::cin), {}};
  const std::string xml = "<sdf version='1.12'><model name='m'><link name='a'/><link name='b'/><joint name='j' type='revolute'><parent>a</parent><child>b</child>" + axisXml + "</joint></model></sdf>";
  sdf::SDFPtr document(new sdf::SDF);
  sdf::init(document);
  sdf::Errors errors;
  const bool parsed = sdf::readString(xml, document, errors);
  if (!parsed || !errors.empty()) { std::cout << "REJECT\n"; return 0; }
  auto joint = document->Root()->GetElement("model")->GetElement("joint");
  if (!joint->HasElement("axis")) { std::cout << "REJECT\n"; return 0; }
  sdf::JointAxis axis;
  errors = axis.Load(joint->GetElement("axis"));
  if (!errors.empty()) { std::cout << "REJECT\n"; return 0; }
  std::cout << "OK";
  for (auto value : {axis.Xyz().X(), axis.Xyz().Y(), axis.Xyz().Z()}) number(value);
  text(axis.XyzExpressedIn());
  for (auto value : {axis.Damping(), axis.Friction(), axis.SpringReference(), axis.SpringStiffness(), axis.Lower(), axis.Upper(), axis.Effort(), axis.MaxVelocity(), axis.Stiffness(), axis.Dissipation()}) number(value);
  const auto mimic = axis.Mimic();
  std::cout << ' ' << mimic.has_value();
  if (mimic) {
    text(mimic->Joint()); text(mimic->Axis());
    number(mimic->Multiplier()); number(mimic->Offset()); number(mimic->Reference());
  }
  std::cout << '\n';
}
