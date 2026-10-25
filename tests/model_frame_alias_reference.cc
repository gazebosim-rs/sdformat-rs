// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/Model.hh>
#include <sdf/World.hh>
#include <sdf/Link.hh>
#include <sdf/Joint.hh>
#include <sdf/JointAxis.hh>
#include <sdf/Frame.hh>
#include <sdf/Collision.hh>
#include <sdf/SemanticPose.hh>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
static void number(double value) {
  std::uint64_t bits;
  std::memcpy(&bits, &value, sizeof(bits));
  std::cout << ' ' << bits;
}
int main(int argc, char **argv) {
  if (argc != 2) return 2;
  const std::string kind = argv[1];
  const std::string xml((std::istreambuf_iterator<char>(std::cin)), {});
  sdf::Root root;
  const auto errors = root.LoadSdfString(xml);
  std::cout << errors.size();
  if (!errors.empty()) {
    for (const auto &error : errors) std::cout << ' ' << int(error.Code());
    std::cout << '\n'; return 0;
  }
  const auto *model = root.Model() ? root.Model() : root.WorldByIndex(0)->ModelByIndex(0);
  const auto semantic = kind == "frame" ? model->FrameByName("probe")->SemanticPose() :
      kind == "link" ? model->LinkByName("probe")->SemanticPose() :
      kind == "joint" ? model->JointByName("probe")->SemanticPose() :
      kind == "collision" ? model->LinkByName("probe")->CollisionByIndex(0)->SemanticPose() : model->SemanticPose();
  gz::math::Pose3d pose;
  const auto poseErrors = semantic.Resolve(pose, kind == "model" ? "" : "__model__");
  std::cout << ' ' << poseErrors.size();
  if (poseErrors.empty()) {
    for (double value : {pose.Pos().X(), pose.Pos().Y(), pose.Pos().Z(),
         pose.Rot().X(), pose.Rot().Y(), pose.Rot().Z(), pose.Rot().W()}) number(value);
  } else for (const auto &error : poseErrors) std::cout << ' ' << int(error.Code());
  if (kind == "joint") {
    const auto *axis = model->JointByName("probe")->Axis(0);
    std::cout << ' ' << (axis != nullptr);
    if (axis) {
      gz::math::Vector3d xyz;
      const auto errors = axis->ResolveXyz(xyz, "__model__");
      std::cout << ' ' << errors.size();
      if (errors.empty()) for (double value : {xyz.X(), xyz.Y(), xyz.Z()}) number(value);
      else for (const auto &error : errors) std::cout << ' ' << int(error.Code());
    }
  }
  std::cout << '\n';
}
