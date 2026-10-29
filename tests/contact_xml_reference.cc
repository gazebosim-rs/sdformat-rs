// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
#include <vector>
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
void config(const sdf::ElementPtr &node)
{
  std::vector<std::string> collisions;
  for (auto value = node->GetElement("collision"); value;
       value = value->GetNextElement("collision")) collisions.push_back(value->Get<std::string>());
  std::cout << ' ' << collisions.size();
  for (const auto &value : collisions) text(value);
  text(node->Get<std::string>("topic", "__default_topic__").first);
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
  config(value.Element()->GetElement("contact"));
}
int main(int argc, char **argv)
{
  if (argc != 2) return 2;
  const std::string mode = argv[1];
  if (mode != "config" && mode != "sensor" && mode != "producer") return 2;
  std::string fragment{std::istreambuf_iterator<char>(std::cin), {}};
  if (mode == "config") fragment = "<sensor name='s' type='contact'>" + fragment + "</sensor>";
  sdf::SDFPtr document(new sdf::SDF);
  sdf::init(document);
  sdf::Errors errors;
  if (!sdf::readString("<sdf version='1.12'><model name='m'><link name='l'>"+
      fragment+"</link></model></sdf>", document, errors) || !errors.empty())
  { std::cout << "REJECT\n"; return 0; }
  auto link = document->Root()->GetElement("model")->GetElement("link");
  if (!link->HasElement("sensor")) { std::cout << "REJECT\n"; return 0; }
  sdf::Sensor value;
  errors = value.Load(link->GetElement("sensor"));
  if (!errors.empty() || value.Type() != sdf::SensorType::CONTACT)
  { std::cout << "REJECT\n"; return 0; }
  if (mode == "producer")
  {
    auto output = value.ToElement(errors);
    std::cout << "PRODUCER " << static_cast<bool>(output) << ' ' << errors.size();
    for (const auto &error : errors) std::cout << ' ' << static_cast<int>(error.Code());
    std::cout << '\n'; return 0;
  }
  std::cout << "OK";
  if (mode == "config") config(value.Element()->GetElement("contact"));
  else sensor(value);
  std::cout << '\n';
}
