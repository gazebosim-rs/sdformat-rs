// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/Model.hh>
#include <sdf/Link.hh>
#include <sdf/Sensor.hh>
#include <sdf/Element.hh>
#include <iostream>
#include <iomanip>
#include <iterator>
void hex(const std::string &v) { for (unsigned char c:v) std::cout << std::hex << std::setfill('0') << std::setw(2) << unsigned(c); }
int main() {
 std::string xml((std::istreambuf_iterator<char>(std::cin)),{});
 sdf::Root root; auto errors=root.LoadSdfString(xml);
 if (!errors.empty()) { std::cout << "error\n"; for(const auto &e:errors) std::cerr << e.Message() << "\n"; return 0; }
 auto sensor=root.Model()->LinkByIndex(0)->SensorByIndex(0);
 std::cout << "accepted\n"; hex(sensor->Topic()); std::cout << "\n";
}
