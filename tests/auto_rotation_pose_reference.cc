// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Collision.hh>
#include <sdf/Element.hh>
#include <sdf/Link.hh>
#include <sdf/Model.hh>
#include <sdf/ParserConfig.hh>
#include <sdf/Root.hh>
#include <gz/math/Inertial.hh>
#include <cmath>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <iostream>
#include <iterator>
#include <string>

static unsigned long long Bits(double value)
{
  unsigned long long result;
  static_assert(sizeof(result) == sizeof(value));
  std::memcpy(&result, &value, sizeof(result));
  return result;
}

static bool Values(const gz::math::Inertiald &value)
{
  const auto &matrix = value.MassMatrix();
  bool finite = std::isfinite(matrix.Mass());
  std::cout << ' ' << Bits(matrix.Mass());
  for (const auto &vector : {matrix.DiagonalMoments(), matrix.OffDiagonalMoments(),
                           value.Pose().Pos()})
  {
    for (int i = 0; i < 3; ++i)
    {
      std::cout << ' ' << Bits(vector[i]);
      finite = finite && std::isfinite(vector[i]);
    }
  }
  const auto rotation = value.Pose().Rot();
  for (double entry : {rotation.X(), rotation.Y(), rotation.Z(), rotation.W()})
  {
    std::cout << ' ' << Bits(entry);
    finite = finite && std::isfinite(entry);
  }
  return finite;
}

int main()
{
  const std::string xml(std::istreambuf_iterator<char>(std::cin), {});
  sdf::Root root;
  sdf::ParserConfig config;
  const auto errors = root.LoadSdfString(xml, config);
  std::cout << "LOAD";
  for (const auto &error : errors)
    std::cout << ' ' << static_cast<int>(error.Code());
  std::cout << '\n';
  const auto *model = root.Model();
  if (!model || model->LinkCount() != 1)
    return 2;
  const auto *link = model->LinkByIndex(0);
  if (link->CollisionCount() != 1)
    return 3;
  const auto output = [&]()
  {
    std::cout << "VALUES " << link->AutoInertia() << ' ' << link->AutoInertiaSaved();
    const bool finite = Values(link->Inertial());
    std::cout << ' ' << link->Inertial().FluidAddedMass().has_value()
              << ' ' << link->Density().has_value() << '\n';
    std::cout << "ROOT_STATUS " << finite << ' '
              << link->Inertial().MassMatrix().IsValid() << '\n';
  };
  output();
  auto collision = *link->CollisionByIndex(0);
  sdf::Errors directErrors;
  gz::math::Inertiald direct;
  collision.CalculateInertial(directErrors, direct, config);
  std::cout << "COLLISION_ERRORS";
  for (const auto &error : directErrors)
    std::cout << ' ' << static_cast<int>(error.Code());
  std::cout << "\nCOLLISION_VALUES";
  const bool directFinite = Values(direct);
  std::cout << "\nCOLLISION_STATUS " << directFinite << ' '
            << direct.MassMatrix().IsValid() << '\n';
  std::cout << "COLLISION_MOI";
  const auto matrix = direct.Moi();
  for (int row = 0; row < 3; ++row)
    for (int column = 0; column < 3; ++column)
      std::cout << ' ' << Bits(matrix(row, column));
  std::cout << '\n';
  sdf::Errors resolve;
  root.ResolveAutoInertials(resolve, config);
  std::cout << "RESOLVE";
  for (const auto &error : resolve)
    std::cout << ' ' << static_cast<int>(error.Code());
  std::cout << '\n';
  output();
  if (const char *path = std::getenv("GZ_ROTATION_MAPS"))
  {
    std::ifstream maps("/proc/self/maps");
    std::ofstream file(path);
    file << maps.rdbuf();
    if (!file)
      return 4;
  }
}
