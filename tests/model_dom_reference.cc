// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/Model.hh>
#include <sdf/Link.hh>
#include <iostream>
#include <iterator>
#include <map>
#include <string>

void collect(const sdf::Model &model, const std::string &path,
             std::map<const void *, std::string> &addresses)
{
  addresses[&model] = path;
  for (std::uint64_t i = 0; i < model.LinkCount(); ++i)
    addresses[model.LinkByIndex(i)] = path + "#" + std::to_string(i);
  for (std::uint64_t i = 0; i < model.ModelCount(); ++i)
    collect(*model.ModelByIndex(i), path.empty() ? std::to_string(i)
      : path + "/" + std::to_string(i), addresses);
}
int main(int argc, char **argv)
{
  if (argc != 4) return 2;
  std::string xml((std::istreambuf_iterator<char>(std::cin)), {});
  sdf::Root root;
  auto errors = root.LoadSdfString(xml);
  if (!errors.empty() || !root.Model()) return 3;
  const auto *model = root.Model();
  std::map<const void *, std::string> addresses;
  collect(*model, "", addresses);
  if (std::string(argv[1]) != "") model = model->ModelByName(argv[1]);
  if (!model) return 4;
  const void *found = nullptr;
  if (std::string(argv[2]) == "model") found = model->ModelByName(argv[3]);
  else if (std::string(argv[2]) == "link") found = model->LinkByName(argv[3]);
  else return 5;
  std::cout << (found ? addresses.at(found) : "-") << '\n';
}
