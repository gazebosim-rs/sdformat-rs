// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/Model.hh>
#include <sdf/Link.hh>
#include <sdf/Sensor.hh>
#include <sdf/Imu.hh>
#include <sdf/Noise.hh>
#include <sdf/Collision.hh>
#include <sdf/Geometry.hh>
#include <sdf/Mesh.hh>
#include <iostream>
#include <iterator>
#include <cstring>
#include <cstdint>
int main() {
 std::string xml((std::istreambuf_iterator<char>(std::cin)),{});sdf::Root root;
 auto errors=root.LoadSdfString(xml);if(!errors.empty()){std::cout<<"error\n";return 0;}
 auto *model=root.Model();auto *link=model->LinkByIndex(0);
 double value=link->SensorCount() ? link->SensorByIndex(0)->ImuSensor()->AngularVelocityXNoise().Mean() :
  link->CollisionCount() ? link->CollisionByIndex(0)->Geom()->MeshShape()->Scale().X() : model->RawPose().Pos().X();
 std::uint64_t bits;std::memcpy(&bits,&value,sizeof(bits));std::cout<<bits<<"\n";
}
