// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Heightmap.hh>
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
void values(const sdf::Heightmap &value)
{
  std::cout << "VALUES";
  text(value.Uri());
  for (double v : {value.Size().X(), value.Size().Y(), value.Size().Z(),
      value.Position().X(), value.Position().Y(), value.Position().Z()}) number(v);
  std::cout << ' ' << value.UseTerrainPaging() << ' ' << value.Sampling()
      << ' ' << value.TextureCount();
  for (uint64_t i = 0; i < value.TextureCount(); ++i)
  {
    const auto *t = value.TextureByIndex(i);
    number(t->Size()); text(t->Diffuse()); text(t->Normal());
  }
  std::cout << ' ' << value.BlendCount();
  for (uint64_t i = 0; i < value.BlendCount(); ++i)
  {
    const auto *b = value.BlendByIndex(i);
    number(b->MinHeight()); number(b->FadeDistance());
  }
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
    sdf::Heightmap heightmap;
    if (mode != "--empty")
    {
      heightmap.SetUri("https://test-uri.org");
      heightmap.SetSize({1, 2, 3});
      heightmap.SetPosition({4, 5, 6});
      heightmap.SetUseTerrainPaging(true);
      heightmap.SetSampling(2);
      sdf::HeightmapTexture texture;
      texture.SetSize(1.2);
      texture.SetDiffuse("diffuse_map");
      texture.SetNormal("normal_map");
      heightmap.AddTexture(texture);
      sdf::HeightmapBlend blend;
      blend.SetMinHeight(1.2);
      blend.SetFadeDistance(3.4);
      heightmap.AddBlend(blend);
    }
    sdf::Errors e;
    auto element = heightmap.ToElement(e);
    std::cout << "EMIT " << bool(element);
    errors(e);
    if (!element) return 4;
    std::cout << "XML_BEGIN\n" << element->ToString("") << "XML_END\n";
    sdf::Heightmap loaded;
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
      ->GetElement("collision")->GetElement("geometry")->GetElement("heightmap");
  sdf::Heightmap heightmap;
  e = heightmap.Load(element, config);
  std::cout << "LOAD";
  errors(e);
  values(heightmap);
}
