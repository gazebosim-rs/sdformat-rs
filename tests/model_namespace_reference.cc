// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <cstdint>
#include <iostream>
#include <optional>
#include <string>
#include <sdf/Element.hh>
#include <sdf/Error.hh>
#include <sdf/Frame.hh>
#include <sdf/Model.hh>
#include <sdf/Root.hh>
#include <sdf/World.hh>

void hex(const std::string &value)
{
  constexpr char digits[] = "0123456789abcdef";
  std::cout << '"';
  for (unsigned char c : value)
    std::cout << digits[c >> 4] << digits[c & 15];
  std::cout << '"';
}
void optional(const std::optional<std::string> &value)
{
  if (value) hex(*value);
  else std::cout << "null";
}
void model(const sdf::Model &value)
{
  std::cout << "{\"name_hex\":"; hex(value.Name());
  std::cout << ",\"raw_hex\":"; optional(value.RawNamespace());
  std::cout << ",\"resolved_hex\":"; optional(value.Namespace());
  std::cout << ",\"original_version_hex\":";
  if (value.Element()) hex(value.Element()->OriginalVersion());
  else std::cout << "null";
  std::cout << ",\"links\":" << value.LinkCount() << ",\"frames_hex\":[";
  for (uint64_t i = 0; i < value.FrameCount(); ++i)
  {
    if (i) std::cout << ',';
    hex(value.FrameByIndex(i)->Name());
  }
  std::cout << "],\"models\":[";
  for (uint64_t i = 0; i < value.ModelCount(); ++i)
  {
    if (i) std::cout << ',';
    model(*value.ModelByIndex(i));
  }
  std::cout << "]}";
}
int main(int argc, char **argv)
{
  if (argc != 2) return 2;
  sdf::Root root;
  const auto errors = root.Load(argv[1]);
  std::cout << "{\"errors\":[";
  for (std::size_t i = 0; i < errors.size(); ++i)
  {
    if (i) std::cout << ',';
    std::cout << "{\"code\":" << static_cast<int>(errors[i].Code()) << ",\"message_hex\":";
    hex(errors[i].Message());
    std::cout << '}';
  }
  std::cout << "],\"root_model\":";
  if (root.Model()) model(*root.Model());
  else std::cout << "null";
  std::cout << ",\"worlds\":[";
  for (uint64_t i = 0; i < root.WorldCount(); ++i)
  {
    if (i) std::cout << ',';
    const auto *world = root.WorldByIndex(i);
    std::cout << "{\"name_hex\":"; hex(world->Name());
    std::cout << ",\"models\":[";
    for (uint64_t j = 0; j < world->ModelCount(); ++j)
    {
      if (j) std::cout << ',';
      model(*world->ModelByIndex(j));
    }
    std::cout << "]}";
  }
  std::cout << "]}\n";
  return 0;
}
