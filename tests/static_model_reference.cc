// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include "FrameSemantics.hh"
#include "ScopedGraph.hh"
#include <memory>
#include <sdf/World.hh>
#include <sdf/Model.hh>
#include <sdf/Link.hh>
#include <sdf/Frame.hh>
#include <sdf/Joint.hh>
#include <sdf/SemanticPose.hh>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
static void text(const std::string &value) {
  std::cout << ' ';
  if (value.empty()) std::cout << '-';
  for (unsigned char c : value) { const char *h="0123456789abcdef"; std::cout<<h[c/16]<<h[c%16]; }
}
static void number(double value) { std::uint64_t bits; std::memcpy(&bits,&value,sizeof(bits)); std::cout<<' '<<bits; }
static void errors(const sdf::Errors &errors) {
  std::cout << ' ' << errors.size();
  for (const auto &error : errors) std::cout << ' ' << int(error.Code());
}
template<typename T>
static void frames(const T &owner, const std::string &target="__model__") {
  std::cout << ' ' << owner.FrameCount();
  for(std::uint64_t i=0;i<owner.FrameCount();++i) {
    const auto *frame=owner.FrameByIndex(i); text(frame->Name());
    std::string body; errors(frame->ResolveAttachedToBody(body)); text(body);
    gz::math::Pose3d pose; const auto result=frame->SemanticPose().Resolve(pose,target);
    errors(result);
    if(result.empty())for(double v:{pose.Pos().X(),pose.Pos().Y(),pose.Pos().Z(),pose.Rot().X(),pose.Rot().Y(),pose.Rot().Z(),pose.Rot().W()}) number(v);
  }
}
static void model(const sdf::Model &value, const sdf::ScopedGraph<sdf::FrameAttachedToGraph> &graph, const std::string &path) {
  text(value.Name());std::cout<<' '<<value.Static();
  std::string body; errors(sdf::resolveFrameAttachedToBody(body,graph,path));text(body);
  const auto canonical=value.CanonicalLinkAndRelativeName();
  std::cout<<' '<<(canonical.first!=nullptr);text(canonical.second);
  frames(value);
  std::cout<<' '<<value.JointCount();
  for(std::uint64_t i=0;i<value.JointCount();++i) {
    const auto *joint=value.JointByIndex(i);text(joint->Name());text(joint->ParentName());text(joint->ChildName());
    std::string parent,child; errors(joint->ResolveParentLink(parent));text(parent);errors(joint->ResolveChildLink(child));text(child);
  }
  std::cout<<' '<<value.ModelCount();for(std::uint64_t i=0;i<value.ModelCount();++i)model(*value.ModelByIndex(i),graph,path+"::"+value.ModelByIndex(i)->Name());
}
int main() {
  const std::string xml((std::istreambuf_iterator<char>(std::cin)),{});sdf::Root root;
  const auto result=root.LoadSdfString(xml);std::cout<<result.size();
  if(!result.empty()){for(const auto &error:result)std::cout<<' '<<int(error.Code());std::cout<<'\n';return 0;}
  std::cout<<' '<<(root.Model()!=nullptr);if(root.Model()) {
    auto owned=std::make_shared<sdf::FrameAttachedToGraph>();sdf::ScopedGraph<sdf::FrameAttachedToGraph> graph(owned);
    const auto built=sdf::buildFrameAttachedToGraph(graph,root.Model());errors(built);errors(sdf::validateFrameAttachedToGraph(graph));model(*root.Model(),graph,root.Model()->Name());
  }
  std::cout<<' '<<root.WorldCount();for(std::uint64_t i=0;i<root.WorldCount();++i){const auto *world=root.WorldByIndex(i);text(world->Name());frames(*world,"world");
    auto owned=std::make_shared<sdf::FrameAttachedToGraph>();sdf::ScopedGraph<sdf::FrameAttachedToGraph> graph(owned);errors(sdf::buildFrameAttachedToGraph(graph,world));errors(sdf::validateFrameAttachedToGraph(graph));std::cout<<' '<<world->ModelCount();for(std::uint64_t j=0;j<world->ModelCount();++j)model(*world->ModelByIndex(j),graph,world->ModelByIndex(j)->Name());}
  std::cout<<'\n';
}
