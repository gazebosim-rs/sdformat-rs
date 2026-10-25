// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/Model.hh>
#include <sdf/Link.hh>
#include <sdf/Joint.hh>
#include <sdf/JointAxis.hh>
#include <sdf/Frame.hh>
#include <sdf/Plugin.hh>
#include <sdf/Collision.hh>
#include <filesystem>
#include <sdf/ParserConfig.hh>
#include <sdf/SemanticPose.hh>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
void text(const std::string &v) {std::cout << ' ';
  if(v.empty())std::cout << '-';
  for(unsigned char c:v){const char *h="0123456789abcdef";
    std::cout << h[c/16] << h[c%16];
}}
void number(double value){uint64_t bits;
  std::memcpy(&bits,&value,sizeof bits);
  std::cout << ' ' << bits;
}
void pose(const gz::math::Pose3d &v){for(int i=0;i<3;++i)number(v.Pos()[i]);
  number(v.Rot().X());
  number(v.Rot().Y());
  number(v.Rot().Z());
  number(v.Rot().W());
}
void semantic(const sdf::SemanticPose &p){gz::math::Pose3d result;
  auto errors=p.Resolve(result);
  std::cout << ' ' << (!errors.empty());
  if(errors.empty())pose(result);
}
void model(const sdf::Model &m){
  std::cout << " M";
  text(m.Name());
  std::cout << ' ' << m.Static();
  text(m.CanonicalLinkAndRelativeName().second);
  text(m.PoseRelativeTo());
  pose(m.RawPose());
  semantic(m.SemanticPose());
  std::cout << " L " << m.LinkCount();
  for(uint64_t i=0;i<m.LinkCount();++i){auto l=m.LinkByIndex(i);
    text(l->Name());
    text(l->PoseRelativeTo());
    pose(l->RawPose());
    semantic(l->SemanticPose());
    std::cout << " X " << l->CollisionCount();
    for(uint64_t j=0;j<l->CollisionCount();++j){auto c=l->CollisionByIndex(j);
      text(c->Name());
      text(c->PoseRelativeTo());
      pose(c->RawPose());
      semantic(c->SemanticPose());
  }}
  std::cout << " F " << m.FrameCount();
  for(uint64_t i=0;i<m.FrameCount();++i){auto f=m.FrameByIndex(i);
    text(f->Name());
    text(f->AttachedTo());
    text(f->PoseRelativeTo());
    pose(f->RawPose());
    semantic(f->SemanticPose());
    std::string body;
    auto errors=f->ResolveAttachedToBody(body);
    std::cout << ' ' << errors.size();
    text(body);
  }
  std::cout << " J " << m.JointCount();
  for(uint64_t i=0;i<m.JointCount();++i){auto j=m.JointByIndex(i);
    text(j->Name());
    text(j->ParentName());
    text(j->ChildName());
    text(j->PoseRelativeTo());
    pose(j->RawPose());
    semantic(j->SemanticPose());
    for(unsigned int a=0;a<2;++a){auto axis=j->Axis(a);
      std::cout << ' ' << (axis!=nullptr);
      if(axis){text(axis->XyzExpressedIn());
        for(int k=0;k<3;++k)number(axis->Xyz()[k]);
        gz::math::Vector3d result;
        auto errors=axis->ResolveXyz(result);
        std::cout << ' ' << errors.size();
        if(errors.empty())for(int k=0;k<3;++k)number(result[k]);
  }}}
  std::cout << " P " << m.Plugins().size();
  for(auto &p:m.Plugins()){text(p.Name());
    text(p.Filename());
  }
  std::cout << " C " << m.ModelCount();
  for(uint64_t i=0;i<m.ModelCount();++i)model(*m.ModelByIndex(i));
}
int main(int argc,char **argv){if(argc!=2)return 2;
  sdf::ParserConfig config;
  config.SetFindCallback([&](const std::string &uri){if(uri=="model://grandchild")return (std::filesystem::path(argv[1]).parent_path()/"grandchild.sdf").string();return std::string(argv[1]);});
  std::string xml{std::istreambuf_iterator<char>(std::cin),{}};
  sdf::Root root;
  auto errors=root.LoadSdfString(xml,config);
  std::cout << errors.size();
  if(errors.empty())model(*root.Model());
  else for(auto &e:errors)std::cout << ' ' << static_cast<int>(e.Code());
  std::cout << '\n';
}
