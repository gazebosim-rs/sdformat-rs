// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
#include <sdf/Joint.hh>
#include <sdf/Model.hh>
#include <sdf/Root.hh>
#include <sdf/SemanticPose.hh>
#include <sdf/World.hh>
int main(int argc, char **argv)
{
  if (argc != 5) return 2;
  const std::string xml{std::istreambuf_iterator<char>(std::cin),{}};
  sdf::Root root; auto errors=root.LoadSdfString(xml);
  if (!errors.empty()) { std::cout << "LOAD_REJECT\n";return 0; }
  const std::string worldName=argv[1], modelName=argv[2], jointName=argv[3];
  const sdf::World *world=worldName.empty()?nullptr:root.WorldByName(worldName);
  const sdf::Model *model=nullptr;
  if (!modelName.empty())
  {
    if (world) model=world->ModelByName(modelName);
    else if (root.Model())
    {
      if (modelName == root.Model()->Name()) model=root.Model();
      else if (modelName.rfind(root.Model()->Name()+"::",0)==0)
        model=root.Model()->ModelByName(modelName.substr(root.Model()->Name().size()+2));
    }
  }
  const sdf::Joint *joint=model?model->JointByName(jointName):(world?world->JointByName(jointName):nullptr);
  if (!joint) { std::cout << "MISSING\n";return 0; }
  gz::math::Pose3d pose;errors=joint->SemanticPose().Resolve(pose,argv[4]);
  if (!errors.empty()) { std::cout << "REJECT " << static_cast<int>(errors.front().Code()) << '\n';return 0; }
  std::cout << "OK";
  for (double value : {pose.Pos().X(),pose.Pos().Y(),pose.Pos().Z(),pose.Rot().X(),pose.Rot().Y(),pose.Rot().Z(),pose.Rot().W()})
  { uint64_t bits;std::memcpy(&bits,&value,sizeof(bits));std::cout << ' ' << bits; }
  std::cout << '\n';
}
