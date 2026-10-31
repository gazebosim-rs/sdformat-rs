// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <cstdint>
#include <cstring>
#include <iomanip>
#include <iostream>
#include <iterator>
#include <string>
#include <sdf/Element.hh>
#include <sdf/Link.hh>
#include <sdf/Model.hh>
#include <sdf/Root.hh>
void bits(double value) {
  uint64_t result;
  std::memcpy(&result, &value, sizeof(result));
  std::cout << ' ' << result;
}
void text(const std::string &value) {
  std::cout << std::dec << value.size() << ':';
  for (unsigned char byte : value)
    std::cout << std::hex << std::setw(2) << std::setfill('0') << unsigned(byte);
  std::cout << std::dec;
}
void decay(const sdf::Link &link) {
  std::cout << "LINK "; text(link.Name());
  const auto element = link.Element();
  const bool present = element && element->HasElement("velocity_decay");
  std::cout << " DECAY " << present << '\n';
  if (!present) return;
  const auto value = element->GetElement("velocity_decay");
  std::cout << "CHILDREN " << value->HasElement("linear") << ' '
            << value->HasElement("angular") << " VALUES";
  bits(value->Get<double>("linear"));
  bits(value->Get<double>("angular"));
  std::cout << '\n';
}
int main(int argc, char **argv) {
  const std::string mode = argc > 1 ? argv[1] : "";
  if (mode == "--ctor") { decay(sdf::Link{}); return 0; }
  const std::string xml{std::istreambuf_iterator<char>(std::cin), {}};
  sdf::Root root;
  const auto errors = root.LoadSdfString(xml);
  if (mode == "--project" && errors.empty() && root.Model()) {
    std::cout << "<sdf version='1.12'>\n  <model name='" << root.Model()->Name() << "'>\n";
    for (uint64_t i = 0; i < root.Model()->LinkCount(); ++i) {
      const auto *link = root.Model()->LinkByIndex(i);
      std::cout << "    <link name='" << link->Name() << "'>\n";
      if (link->Element()->HasElement("velocity_decay"))
        std::cout << link->Element()->GetElement("velocity_decay")->ToString("      ");
      std::cout << "    </link>\n";
    }
    std::cout << "  </model>\n</sdf>\n";
    return 0;
  }
  std::cout << "ERRORS " << errors.size();
  for (const auto &error : errors) std::cout << ' ' << int(error.Code());
  std::cout << '\n';
  const auto *model = root.Model();
  if (!model) return 0;
  std::cout << "MODEL "; text(model->Name()); std::cout << " LINKS " << model->LinkCount() << '\n';
  for (uint64_t i = 0; i < model->LinkCount(); ++i) decay(*model->LinkByIndex(i));
  if (mode == "--clone" && model->LinkCount()) {
    const auto *link = model->LinkByIndex(0);
    sdf::Link copy(*link);
    std::cout << "COPY\n"; decay(copy);
    std::cout << "SHARED " << (link->Element() == copy.Element()) << '\n';
    if (link->Element()->HasElement("velocity_decay")) {
      link->Element()->GetElement("velocity_decay")->GetElement("linear")->Set(0.75);
      std::cout << "AFTER\n"; decay(*link); decay(copy);
    }
  }
}
