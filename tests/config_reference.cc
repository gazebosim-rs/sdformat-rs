// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/parser.hh>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <iterator>
#include <string>

std::string hex(const std::string &value)
{
  if (value.empty()) return "-";
  static const char digits[] = "0123456789abcdef";
  std::string result;
  for (unsigned char c : value)
  {
    result += digits[c >> 4];
    result += digits[c & 15];
  }
  return result;
}
int main(int argc, char **argv)
{
  if (argc != 2) return 2;
  const std::filesystem::path directory(argv[1]);
  std::filesystem::create_directories(directory);
  std::string xml(std::istreambuf_iterator<char>(std::cin), {});
  std::ofstream(directory / "model.config") << xml;
  sdf::Errors errors;
  try
  {
    const auto result = sdf::getModelFilePath(errors, directory.string());
    const auto prefix = directory.string() + "/";
    const auto filename = result.rfind(prefix, 0) == 0 ? result.substr(prefix.size()) : result;
    std::cout << "ok " << hex(filename) << " " << errors.size();
    for (const auto &error : errors) std::cout << " " << static_cast<int>(error.Code());
    std::cout << "\n";
  }
  catch (const std::exception &error)
  {
    std::cout << "exception " << hex(error.what()) << "\n";
  }
}
