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
int main(int argc,char **argv){
  if(argc!=2)return 2;
  const std::string mode=argv[1], input{std::istreambuf_iterator<char>(std::cin),{}};
  if(mode=="--light") {
    sdf::SDFPtr document(new sdf::SDF);sdf::init(document);sdf::Errors errors;
    bool read=sdf::readString("<sdf version='1.12'><world name='w'>"+input+"</world></sdf>",document,errors);
    if(!read||!errors.empty()){std::cout<<"REJECT\n";return 0;}
    sdf::Light v;errors=v.Load(document->Root()->GetElement("world")->GetElement("light"));
    if(!errors.empty()){std::cout<<"REJECT\n";return 0;}
    light(v);return 0;
  }
  sdf::Root root;auto errors=root.LoadSdfString(input);
  if(!errors.empty()){std::cout<<"REJECT\n";return 0;}
  const auto *world=root.WorldByIndex(0);
  if(mode=="--single-document") {
    const sdf::Light *v=world?world->LightByIndex(0):nullptr;
    if(!v) {
      const auto *model=world?world->ModelByIndex(0):root.Model();
      if(model && model->LinkByIndex(0))v=model->LinkByIndex(0)->LightByIndex(0);
    }
    if(!v)return 3;
    light(*v);resolved("default",v->SemanticPose(),"");return 0;
  }
  if(mode=="--offset-document") {
    const auto *model=world?world->ModelByIndex(0):nullptr;
    const auto *link=model?model->LinkByIndex(0):nullptr;
    const auto *v=link?link->LightByIndex(0):nullptr;
    if(!v)return 7;
    light(*v);
    resolved("frame",v->SemanticPose(),"f");
    resolved("model",v->SemanticPose(),"__model__");
    resolved("world",v->SemanticPose(),"world");
    resolved("default",v->SemanticPose(),"");
    resolved("unknown",v->SemanticPose(),"missing");
    return 0;
  }
  if(mode!="--document"||!world||world->LightCount()!=3||world->ModelCount()!=1)return 4;
  for(unsigned i=0;i<3;++i){std::cout<<"LIGHT world"<<i<<' ';light(*world->LightByIndex(i));}
  const auto *model=world->ModelByIndex(0);
  if(!model||model->LinkCount()!=1)return 5;
  const auto *link=model->LinkByIndex(0);
  if(!link||link->LightCount()!=1)return 6;
  std::cout<<"LIGHT link0 ";light(*link->LightByIndex(0));
  auto p=model->RawPose();std::cout<<"MODEL";text(model->Name());text(model->PoseRelativeTo());
  for(double v:{p.Pos().X(),p.Pos().Y(),p.Pos().Z(),p.Rot().X(),p.Rot().Y(),p.Rot().Z(),p.Rot().W()})number(v);
  std::cout<<'\n';
  resolved("point_world",world->LightByIndex(0)->SemanticPose(),"world");
  resolved("point_default",world->LightByIndex(0)->SemanticPose(),"");
  resolved("directional_frame1",world->LightByIndex(2)->SemanticPose(),"frame1");
  resolved("directional_world",world->LightByIndex(2)->SemanticPose(),"world");
  resolved("directional_default",world->LightByIndex(2)->SemanticPose(),"");
  resolved("model_frame1",model->SemanticPose(),"frame1");
  resolved("model_world",model->SemanticPose(),"world");
  resolved("model_default",model->SemanticPose(),"");
  resolved("link_frame2",link->LightByIndex(0)->SemanticPose(),"frame2");
  resolved("link_model",link->LightByIndex(0)->SemanticPose(),"__model__");
  resolved("link_default",link->LightByIndex(0)->SemanticPose(),"");
  resolved("point_unknown",world->LightByIndex(0)->SemanticPose(),"missing");
  resolved("link_unknown",link->LightByIndex(0)->SemanticPose(),"missing");
}
