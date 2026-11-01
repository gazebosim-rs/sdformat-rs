// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Element.hh>
#include <sdf/Light.hh>
#include <sdf/SDFImpl.hh>
#include <sdf/parser.hh>
#include <sdf/Root.hh>
#include <sdf/World.hh>
#include <sdf/Model.hh>
#include <sdf/Link.hh>
#include <sdf/SemanticPose.hh>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
void text(const std::string &s) { if(s.empty()) {std::cout<<" -";return;}std::cout<<' ';const char*d="0123456789abcdef";for(unsigned char c:s)std::cout<<d[c>>4]<<d[c&15]; }
void number(double v){uint64_t b;std::memcpy(&b,&v,sizeof(b));std::cout<<' '<<b;}
void channel(float v){uint32_t b;std::memcpy(&b,&v,sizeof(b));std::cout<<' '<<b;}
void light(const sdf::Light &v) {
 std::cout<<"OK";text(v.Name());std::cout<<' '<<int(v.Type());
 auto p=v.RawPose();for(double n:{p.Pos().X(),p.Pos().Y(),p.Pos().Z(),p.Rot().X(),p.Rot().Y(),p.Rot().Z(),p.Rot().W()})number(n);
 text(v.PoseRelativeTo());std::cout<<' '<<v.LightOn()<<' '<<v.Visualize()<<' '<<v.CastShadows();
 for(auto c:{v.Diffuse(),v.Specular()})for(float n:{c.R(),c.G(),c.B(),c.A()})channel(n);
 for(double n:{v.AttenuationRange(),v.SpotFalloff(),v.SpotInnerAngle().Radian(),v.SpotOuterAngle().Radian(),v.LinearAttenuationFactor(),v.ConstantAttenuationFactor(),v.QuadraticAttenuationFactor(),v.Direction().X(),v.Direction().Y(),v.Direction().Z(),v.Intensity()})number(n);
 std::cout<<'\n';
}
void resolved(const std::string &label, const sdf::SemanticPose &semantic, const std::string &target) {
  gz::math::Pose3d p; auto errors=semantic.Resolve(p,target);
  std::cout << "POSE " << label;
  if(!errors.empty()) {std::cout << " REJECT\n";return;}
  std::cout << " OK";
  for(double v:{p.Pos().X(),p.Pos().Y(),p.Pos().Z(),p.Rot().X(),p.Rot().Y(),p.Rot().Z(),p.Rot().W()})number(v);
  std::cout << '\n';
}
// The complete getter projection above matches document_lights_reference.cc.
#include <sdf/ParserConfig.hh>
int main(int argc, char **argv) {
  if (argc != 2) return 2;
  sdf::ParserConfig config;
  config.SetFindCallback([&](const std::string &uri) {
    std::cout << "REQUEST"; text(uri); std::cout << '\n';
    return std::string(argv[1]);
  });
  const std::string xml{std::istreambuf_iterator<char>(std::cin), {}};
  sdf::Root root;
  const auto errors = root.LoadSdfString(xml, config);
  std::cout << "ERRORS " << errors.size() << '\n';
  for (const auto &error : errors) {
    std::cout << "ERROR " << int(error.Code()); text(error.Message()); std::cout << '\n';
  }
  const auto *world = root.WorldByIndex(0);
  if (!world) return 0;
  std::cout << "COUNT " << world->LightCount() << '\n';
  for (uint64_t i = 0; i < world->LightCount(); ++i) {
    const auto *value = world->LightByIndex(i);
    light(*value);
    std::cout << "SOURCE"; text(value->Element()->FilePath());
    std::cout << ' ' << value->Element()->LineNumber().value_or(-1);
    text(value->Element()->OriginalVersion()); std::cout << '\n';
    resolved("world", value->SemanticPose(), "world");
  }
}
