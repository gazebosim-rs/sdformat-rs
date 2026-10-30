// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Material.hh>
#include <sdf/Pbr.hh>
#include <sdf/Element.hh>
#include <sdf/ParserConfig.hh>
#include <sdf/SDFImpl.hh>
#include <sdf/parser.hh>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
#include <stdexcept>
#include <vector>

void text(const std::string &value)
{
  if (value.empty()) { std::cout << " -"; return; }
  const char *hex = "0123456789abcdef";
  std::cout << ' ';
  for (unsigned char c : value) std::cout << hex[c >> 4] << hex[c & 15];
}
void number(double value)
{
  uint64_t bits;
  std::memcpy(&bits, &value, sizeof(bits));
  std::cout << ' ' << bits;
}
void small(float value)
{
  uint32_t bits;
  std::memcpy(&bits, &value, sizeof(bits));
  std::cout << ' ' << bits;
}
void workflow(const sdf::PbrWorkflow *value)
{
  std::cout << "FLOW " << bool(value);
  if (!value) { std::cout << '\n'; return; }
  std::cout << ' ' << int(value->Type());
  for (const auto &path : {value->AlbedoMap(), value->NormalMap(),
      value->EnvironmentMap(), value->AmbientOcclusionMap(), value->EmissiveMap(),
      value->LightMap(), value->RoughnessMap(), value->MetalnessMap(),
      value->SpecularMap(), value->GlossinessMap()}) text(path);
  std::cout << ' ' << int(value->NormalMapType()) << ' ' << value->LightMapTexCoordSet();
  number(value->Roughness());
  number(value->Metalness());
  number(value->Glossiness());
  std::cout << '\n';
}
void values(const sdf::Material &value)
{
  std::cout << "BASE";
  for (const auto &color : {value.Ambient(), value.Diffuse(), value.Specular(), value.Emissive()})
  {
    small(color.R()); small(color.G()); small(color.B()); small(color.A());
  }
  number(value.Shininess()); small(value.RenderOrder());
  std::cout << ' ' << value.Lighting() << ' ' << value.DoubleSided() << ' ' << int(value.Shader());
  text(value.NormalMap()); text(value.ScriptUri()); text(value.ScriptName());
  std::cout << '\n';
  std::cout << "RAW_SCRIPT";
  const auto origin = value.Element();
  std::vector<std::string> uris;
  if (origin && origin->HasElement("script"))
  {
    auto script = origin->GetElement("script");
    if (script->HasElement("uri"))
      for (auto uri = script->GetElement("uri"); uri; uri = uri->GetNextElement("uri"))
        uris.push_back(uri->Get<std::string>());
  }
  std::cout << ' ' << uris.size();
  for (const auto &uri : uris) text(uri);
  std::cout << '\n';
  std::cout << "PBR " << bool(value.PbrMaterial()) << '\n';
  workflow(value.PbrMaterial() ? value.PbrMaterial()->Workflow(sdf::PbrWorkflowType::METAL) : nullptr);
  workflow(value.PbrMaterial() ? value.PbrMaterial()->Workflow(sdf::PbrWorkflowType::SPECULAR) : nullptr);
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
    throw std::runtime_error("unexpected asset resolution");
  });
  const std::string mode = argc == 2 ? argv[1] : "";
  if (mode == "--fixture-metal" || mode == "--fixture-specular" || mode == "--default")
  {
    sdf::Material material;
    if (mode != "--default")
    {
      material.SetAmbient({0.1f, 0.2f, 0.3f, 1.0f});
      material.SetDiffuse({0.4f, 0.5f, 0.6f, 0.5f});
      material.SetSpecular({0.6f, 0.4f, 0.8f, 0.8f});
      material.SetEmissive({0.2f, 0.7f, 0.9f, 0.1f});
      material.SetRenderOrder(0.5f);
      material.SetLighting(false); material.SetDoubleSided(true);
      material.SetScriptUri("my-uri"); material.SetScriptName("my-script-name");
      material.SetShader(sdf::ShaderType::VERTEX); material.SetNormalMap("my-normal-map");
      sdf::PbrWorkflow flow;
      flow.SetAlbedoMap("albedo"); flow.SetNormalMap("normal");
      flow.SetEnvironmentMap("env"); flow.SetAmbientOcclusionMap("ambient");
      flow.SetRoughnessMap("rough"); flow.SetMetalnessMap("metalness");
      flow.SetEmissiveMap("emissive"); flow.SetGlossinessMap("gloss");
      flow.SetSpecularMap("gloss"); flow.SetLightMap("light", 1u);
      flow.SetMetalness(1.0); flow.SetRoughness(0.1);
      const auto kind = mode == "--fixture-metal" ?
          sdf::PbrWorkflowType::METAL : sdf::PbrWorkflowType::SPECULAR;
      flow.SetType(kind);
      sdf::Pbr pbr;
      pbr.SetWorkflow(kind, flow);
      material.SetPbrMaterial(pbr);
    }
    values(material);
    sdf::Errors e;
    auto element = material.ToElement(e);
    std::cout << "EMIT " << bool(element);
    errors(e);
    std::cout << "XML_BEGIN\n" << element->ToString("") << "XML_END\n";
    sdf::Material loaded;
    e = loaded.Load(element, config);
    std::cout << "LOAD";
    errors(e);
    values(loaded);
    return 0;
  }
  const std::string input{std::istreambuf_iterator<char>(std::cin), {}};
  const std::string document =
      "<sdf version='1.12'><model name='m'><link name='l'><visual name='v'><geometry><box/></geometry>" +
      input + "</visual></link></model></sdf>";
  sdf::SDFPtr doc(new sdf::SDF);
  sdf::init(doc);
  sdf::Errors e;
  const bool read = sdf::readString(document, doc, e);
  std::cout << "READ " << read;
  errors(e);
  if (!read || !e.empty()) return 0;
  auto element = doc->Root()->GetElement("model")->GetElement("link")
      ->GetElement("visual")->GetElement("material");
  sdf::Material loaded;
  e = loaded.Load(element, config);
  std::cout << "LOAD";
  errors(e);
  values(loaded);
}
