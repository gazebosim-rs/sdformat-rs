// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/Model.hh>
#include <sdf/Link.hh>
#include <sdf/Sensor.hh>
#include <sdf/Element.hh>
#include <iostream>
#include <iterator>
#include <string>
#include <vector>
#include <cstring>
static unsigned long long bits(double value) { unsigned long long out; std::memcpy(&out,&value,sizeof(out)); return out; }
void hex(const std::string &value) {
  if (value.empty()) { std::cout << " -"; return; }
  std::cout << ' ';
  for (unsigned char c : value) { const char *digits="0123456789abcdef"; std::cout<<digits[c/16]<<digits[c%16]; }
}
int main() {
  std::string xml((std::istreambuf_iterator<char>(std::cin)),{});
  sdf::Root root;
  auto errors=root.LoadSdfString(xml);
  std::cout<<errors.size();
  if (!errors.empty()) { for(const auto &e:errors) std::cout<<' '<<static_cast<int>(e.Code()); std::cout<<'\n'; return 0; }
  const auto *s=root.Model()->LinkByIndex(0)->SensorByIndex(0);
  hex(s->TypeStr()); std::cout<<' '<<(s->ImuSensor()!=nullptr);
  const auto element=s->Element();
  std::cout<<' '<<element->HasElement("contact");
  if(s->Type()==sdf::SensorType::CONTACT) {
    auto contact=element->GetElement("contact");
    std::vector<std::string> collisions;
    for(auto c=contact->GetElement("collision");c;c=c->GetNextElement("collision")) collisions.push_back(c->Get<std::string>());
    std::cout<<' '<<collisions.size(); for(const auto &c:collisions) hex(c);
    hex(contact->Get<std::string>("topic","__default_topic__").first);
  }
  hex(s->Topic()); std::cout<<' '<<bits(s->UpdateRate())<<' '
    <<element->Get<bool>("always_on",false).first<<' '<<s->EnableMetrics();
  hex(s->FrameId()); std::cout<<'\n';
}
