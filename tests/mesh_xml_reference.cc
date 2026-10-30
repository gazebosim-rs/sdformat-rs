// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Mesh.hh>
#include <sdf/Element.hh>
#include <sdf/ParserConfig.hh>
#include <sdf/SDFImpl.hh>
#include <sdf/parser.hh>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>

void text(const std::string &value)
{
  if (value.empty()) { std::cout << " -"; return; }
  std::cout << ' ';
  const char *hex = "0123456789abcdef";
  for (unsigned char c : value) std::cout << hex[c >> 4] << hex[c & 15];
}
void number(double value)
{
  uint64_t bits;
  std::memcpy(&bits, &value, sizeof(bits));
  std::cout << ' ' << bits;
}
void values(const sdf::Mesh &mesh)
{
  std::cout << "VALUES";
  text(mesh.Uri());
  text(mesh.OptimizationStr());
  std::cout << ' ' << int(mesh.Optimization());
  for (double v : {mesh.Scale().X(), mesh.Scale().Y(), mesh.Scale().Z()}) number(v);
  text(mesh.Submesh());
  std::cout << ' ' << mesh.CenterSubmesh() << ' ' << bool(mesh.ConvexDecomposition());
  if (auto d = mesh.ConvexDecomposition())
    std::cout << ' ' << d->MaxConvexHulls() << ' ' << d->VoxelResolution();
  std::cout << '\n';
}
void errors(const sdf::Errors &errors)
{
  for (const auto &error : errors) std::cout << ' ' << int(error.Code());
  std::cout << '\n';
}
int main(int argc, char **argv)
{
  sdf::ParserConfig config;
  config.SetStoreResolvedURIs(false);
  config.SetFindCallback([](const std::string &) -> std::string {
    throw std::runtime_error("unexpected resource resolution");
  });
  const std::string mode = argc == 2 ? argv[1] : "";
  if (mode == "--fixture" || mode == "--quiet" || mode == "--empty")
  {
    sdf::Mesh mesh;
    if (mode != "--empty")
    {
      mesh.SetUri("mesh-uri");
      mesh.SetScale({1, 2, 3});
      mesh.SetSubmesh("submesh");
      mesh.SetCenterSubmesh(false);
      if (!mesh.SetOptimization(mode == "--quiet" ? "convex_hull" : "convex_decomposition")) return 3;
      if (mode != "--quiet")
      {
        sdf::ConvexDecomposition d;
        d.SetMaxConvexHulls(10);
        d.SetVoxelResolution(300000);
        mesh.SetConvexDecomposition(d);
      }
    }
    sdf::Errors e;
    auto element = mesh.ToElement(e);
    std::cout << "EMIT " << bool(element);
    errors(e);
    if (!element) return 4;
    std::cout << "XML_BEGIN\n" << element->ToString("") << "XML_END\n";
    sdf::Mesh loaded;
    e = loaded.Load(element, config);
    std::cout << "LOAD";
    errors(e);
    values(loaded);
    return 0;
  }
  const std::string input{std::istreambuf_iterator<char>(std::cin), {}};
  const std::string geometry = mode == "--geometry" ? input : "<geometry>" + input + "</geometry>";
  const std::string document = mode == "--document" ? input :
      "<sdf version='1.12'><model name='m'><link name='l'><collision name='c'>" +
      geometry + "</collision></link></model></sdf>";
  sdf::SDFPtr doc(new sdf::SDF);
  sdf::init(doc);
  sdf::Errors e;
  const bool parsed = sdf::readString(document, doc, e);
  std::cout << "READ " << parsed;
  errors(e);
  if (!parsed || !e.empty()) return 0;
  auto element = doc->Root()->GetElement("model")->GetElement("link")
      ->GetElement("collision")->GetElement("geometry")->GetElement("mesh");
  sdf::Mesh mesh;
  e = mesh.Load(element, config);
  std::cout << "LOAD";
  errors(e);
  values(mesh);
}
