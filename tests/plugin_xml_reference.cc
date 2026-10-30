/*
 * Copyright (C) 2021 Open Source Robotics Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
*/

// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Plugin.hh>
#include <sdf/Element.hh>
#include <sdf/SDFImpl.hh>
#include <sdf/parser.hh>
#include <iostream>
#include <iterator>
void text(const std::string &value)
{
  if (value.empty()) { std::cout << " -"; return; }
  const char *hex = "0123456789abcdef";
  std::cout << ' ';
  for (unsigned char c : value) std::cout << hex[c >> 4] << hex[c & 15];
}
void values(const sdf::Plugin &plugin)
{
  std::cout << "VALUES";
  text(plugin.Name());
  text(plugin.Filename());
  std::cout << ' ' << plugin.Contents().size();
  for (const auto &child : plugin.Contents()) text(child->GetName());
  std::cout << '\n';
}
void errors(const sdf::Errors &errors)
{
  for (const auto &error : errors) std::cout << ' ' << int(error.Code());
  std::cout << '\n';
}
int main(int argc, char **argv)
{
  sdf::Plugin plugin;
  if (argc == 2 && std::string(argv[1]) == "--fixture")
  {
    plugin.SetName("my-plugin");
    plugin.SetFilename("filename.so");
    sdf::ElementPtr content(new sdf::Element);
    content->SetName("an-element");
    plugin.InsertContent(content);
    values(plugin);
    sdf::Errors result;
    auto element = plugin.ToElement(result);
    std::cout << "EMIT " << bool(element);
    errors(result);
    if (!element) return 2;
    std::cout << "XML_BEGIN\n" << element->ToString("") << "XML_END\n";
    sdf::Plugin loaded;
    result = loaded.Load(element);
    std::cout << "LOAD";
    errors(result);
    values(loaded);
    return 0;
  }
  const std::string input{std::istreambuf_iterator<char>(std::cin), {}};
  sdf::SDFPtr document(new sdf::SDF);
  sdf::init(document);
  sdf::Errors result;
  const bool ok = sdf::readString(
      "<sdf version='1.12'><model name='m'><link name='l'/>" + input +
      "</model></sdf>", document, result);
  std::cout << "READ " << ok;
  errors(result);
  if (!ok) return 0;
  auto element = document->Root()->GetElement("model")->GetElement("plugin");
  result = plugin.Load(element);
  std::cout << "LOAD";
  errors(result);
  values(plugin);
}
