// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/Model.hh>
#include <sdf/Link.hh>
#include <sdf/ParserConfig.hh>
#include <sdf/World.hh>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
static unsigned long long bits(double value) {
  unsigned long long result; static_assert(sizeof(result) == sizeof(value));
  std::memcpy(&result, &value, sizeof(result)); return result;
}
int main() {
  const std::string xml(std::istreambuf_iterator<char>(std::cin), {});
  sdf::Root root; sdf::ParserConfig config;
  auto errors = root.LoadSdfString(xml, config);
  std::cout << "LOAD";
  for (const auto &error : errors) std::cout << ' ' << static_cast<int>(error.Code());
  std::cout << '\n';
  const auto *model = root.Model();
  if (!model && root.WorldCount()) model = root.WorldByIndex(0)->ModelByIndex(0);
  if (!model || !model->LinkCount()) return 0;
  const auto *link = model->LinkByIndex(0);
  const auto output = [&]() {
    const auto &inertial = link->Inertial(); const auto &matrix = inertial.MassMatrix();
    std::cout << "VALUES " << link->AutoInertia() << ' ' << link->AutoInertiaSaved();
    std::cout << ' ' << bits(matrix.Mass());
    const auto diagonal = matrix.DiagonalMoments(); const auto off = matrix.OffDiagonalMoments();
    for (int i=0; i<3; ++i) std::cout << ' ' << bits(diagonal[i]);
    for (int i=0; i<3; ++i) std::cout << ' ' << bits(off[i]);
    const auto pose = inertial.Pose();
    for (int i=0; i<3; ++i) std::cout << ' ' << bits(pose.Pos()[i]);
    const auto rotation = pose.Rot();
    for (double v : {rotation.X(), rotation.Y(), rotation.Z(), rotation.W()}) std::cout << ' ' << bits(v);
    std::cout << ' ' << inertial.FluidAddedMass().has_value() << ' ' << link->Density().has_value() << '\n';
  };
  output(); sdf::Errors resolve; root.ResolveAutoInertials(resolve, config);
  std::cout << "RESOLVE";
  for (const auto &error : resolve) std::cout << ' ' << static_cast<int>(error.Code());
  std::cout << '\n'; output();
}
