// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <iostream>
#include <iterator>
#include <string>
#include <sdf/Model.hh>
#include <sdf/ParserConfig.hh>
#include <sdf/Root.hh>
#include <sdf/World.hh>

void model(const sdf::Model &value, const std::string &parent) {
  const auto path = parent + value.Name();
  std::cout << "MODEL " << path << ' ' << value.SelfCollide() << ' '
            << value.AllowAutoDisable() << ' ' << value.Static() << ' '
            << value.EnableWind() << '\n';
  for (uint64_t i = 0; i < value.ModelCount(); ++i)
    model(*value.ModelByIndex(i), path + "::");
}
int main(int argc, char **argv) {
  if (argc > 1 && std::string(argv[1]) == "--constructor") {
    sdf::Model value;
    model(value, "default");
    value.SetSelfCollide(true);
    value.SetAllowAutoDisable(false);
    model(value, "set");
    sdf::Model copy(value);
    value.SetSelfCollide(false);
    value.SetAllowAutoDisable(true);
    model(value, "original");
    model(copy, "copy");
    return 0;
  }
  const std::string xml{std::istreambuf_iterator<char>(std::cin), {}};
  sdf::ParserConfig config;
  if (argc > 1)
    config.SetFindCallback([&](const std::string &uri) {
      return uri == "fixture.sdf" ? std::string(argv[1]) : std::string{};
    });
  sdf::Root root;
  const auto errors = root.LoadSdfString(xml, config);
  std::cout << "ERRORS " << errors.size();
  for (const auto &error : errors) std::cout << ' ' << int(error.Code());
  std::cout << '\n';
  if (root.Model()) model(*root.Model(), "");
  for (uint64_t i = 0; i < root.WorldCount(); ++i) {
    const auto *world = root.WorldByIndex(i);
    for (uint64_t m = 0; m < world->ModelCount(); ++m)
      model(*world->ModelByIndex(m), world->Name() + "::");
  }
}
