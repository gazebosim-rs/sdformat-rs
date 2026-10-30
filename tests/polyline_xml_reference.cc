// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Polyline.hh>
#include <sdf/Geometry.hh>
#include <sdf/Element.hh>
#include <sdf/SDFImpl.hh>
#include <sdf/parser.hh>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>

void number(double value)
{
  uint64_t bits;
  std::memcpy(&bits, &value, sizeof(bits));
  std::cout << ' ' << bits;
}
void values(const std::vector<sdf::Polyline> &lines)
{
  std::cout << "VALUES " << lines.size();
  for (const auto &line : lines)
  {
    number(line.Height());
    std::cout << ' ' << line.PointCount();
    for (uint64_t i = 0; i < line.PointCount(); ++i)
    {
      number(line.PointByIndex(i)->X());
      number(line.PointByIndex(i)->Y());
    }
  }
  std::cout << '\n';
}
void errors(const sdf::Errors &errors)
{
  for (const auto &e : errors) std::cout << ' ' << int(e.Code());
  std::cout << '\n';
}
int main(int argc, char **argv)
{
  const std::string mode = argc == 2 ? argv[1] : "";
  if (mode == "--fixture" || mode == "--empty")
  {
    sdf::Polyline line;
    if (mode == "--fixture")
    {
      line.SetHeight(1.2);
      std::cout << "ADD " << line.AddPoint({1, 2}) << ' '
          << line.AddPoint({3, 4}) << '\n';
    }
    sdf::Errors e;
    auto elem = line.ToElement(e);
    std::cout << "EMIT " << bool(elem);
    errors(e);
    std::cout << "XML_BEGIN\n" << elem->ToString("") << "XML_END\n";
    values({line});
    sdf::Polyline loaded;
    e = loaded.Load(elem);
    std::cout << "LOAD";
    errors(e);
    values({loaded});
    return 0;
  }
  const std::string input{std::istreambuf_iterator<char>(std::cin), {}};
  const std::string geometry = mode == "--geometry" ? input :
      "<geometry>" + input + "</geometry>";
  const std::string document = mode == "--document" ? input :
      "<sdf version='1.12'><model name='m'><link name='l'><collision name='c'>" +
      geometry + "</collision></link></model></sdf>";
  sdf::SDFPtr doc(new sdf::SDF);
  sdf::init(doc);
  sdf::Errors e;
  const bool read = sdf::readString(document, doc, e);
  std::cout << "READ " << read;
  errors(e);
  if (!read || !e.empty()) return 0;
  auto elem = doc->Root()->GetElement("model")->GetElement("link")
      ->GetElement("collision")->GetElement("geometry");
  sdf::Geometry loaded;
  e = loaded.Load(elem);
  std::cout << "LOAD";
  errors(e);
  values(loaded.PolylineShape());
}
