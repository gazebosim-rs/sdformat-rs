// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iomanip>
#include <iterator>
#include <sdf/Box.hh>
#include <sdf/Collision.hh>
#include <sdf/Cylinder.hh>
#include <sdf/Element.hh>
#include <sdf/Geometry.hh>
#include <sdf/Link.hh>
#include <sdf/Joint.hh>
#include <sdf/JointAxis.hh>
#include <sdf/Material.hh>
#include <sdf/Model.hh>
#include <sdf/Root.hh>
#include <sdf/SemanticPose.hh>
#include <sdf/Sphere.hh>
#include <sdf/Surface.hh>
#include <sdf/Visual.hh>
void number(double value) {
  uint64_t bits;
  std::memcpy(&bits, &value, sizeof(bits));
  std::cout << ' ' << bits;
}
void text(const std::string &value) {
  std::cout << ' ';
  if (value.empty()) { std::cout << '-'; return; }
  const char *digits = "0123456789abcdef";
  for (unsigned char byte : value) std::cout << digits[byte >> 4] << digits[byte & 15];
}
void pose(const gz::math::Pose3d &value) {
  for (double n : {value.Pos().X(), value.Pos().Y(), value.Pos().Z(),
                  value.Rot().X(), value.Rot().Y(), value.Rot().Z(), value.Rot().W()}) number(n);
}
void resolved(const char *name, const sdf::SemanticPose &value) {
  gz::math::Pose3d result;
  const auto errors = value.Resolve(result, "__model__");
  std::cout << "RESOLVED " << name << ' ' << errors.size();
  if (errors.empty()) pose(result);
  std::cout << '\n';
}
void geometry(const sdf::Geometry *value) {
  if (!value) { std::cout << "NONE"; return; }
  if (value->SphereShape()) { std::cout << "SPHERE"; number(value->SphereShape()->Radius()); }
  else if (value->BoxShape()) {
    std::cout << "BOX"; const auto size = value->BoxShape()->Size();
    number(size.X()); number(size.Y()); number(size.Z());
  } else if (value->CylinderShape()) {
    std::cout << "CYLINDER"; number(value->CylinderShape()->Radius()); number(value->CylinderShape()->Length());
  } else { std::cout << "OTHER " << int(value->Type()); }
}
int main(int argc, char **argv) {
  const std::string xml{std::istreambuf_iterator<char>(std::cin), {}};
  sdf::Root root;
  sdf::Errors errors;
  try { errors = root.LoadSdfString(xml); }
  catch (const std::exception &error) {
    std::cout << "EXCEPTION"; text(error.what()); std::cout << '\n'; return 2;
  }
  if (argc > 1 && std::string(argv[1]) == "--xml") {
    if (errors.empty() && root.Element()) std::cout << root.Element()->ToString("");
    return 0;
  }
  std::cout << "ERRORS " << errors.size();
  for (const auto &error : errors) std::cout << ' ' << int(error.Code());
  std::cout << '\n';
  const auto *model = root.Model();
  if (!model) return 0;
  if (argc > 1 && std::string(argv[1]) == "--frames") {
    if (!errors.empty() || !model->CanonicalLink()) return 3;
    const auto *joint = model->JointByIndex(0);
    if (!joint || !joint->Axis()) return 4;
    std::cout << std::setprecision(17) << "CANONICAL " << model->CanonicalLink()->Name() << '\n';
    for (const auto &target : {std::string(), joint->ParentName(), joint->ChildName(), std::string("__model__")}) {
      gz::math::Pose3d value;
      const auto poseErrors = joint->SemanticPose().Resolve(value, target);
      gz::math::Vector3d axis;
      const auto axisErrors = joint->Axis()->ResolveXyz(axis, target);
      std::cout << "TARGET " << (target.empty() ? "<default>" : target) << ' ' << poseErrors.size() << ' ' << axisErrors.size();
      for (double n : {value.Pos().X(), value.Pos().Y(), value.Pos().Z(), value.Rot().X(), value.Rot().Y(), value.Rot().Z(), value.Rot().W(), axis.X(), axis.Y(), axis.Z()}) std::cout << ' ' << n;
      std::cout << '\n';
    }
    return 0;
  }
  if (argc > 1 && std::string(argv[1]) == "--clone") {
    const auto *original = model->JointByIndex(0);
    if (!original) return 3;
    sdf::Joint copied(*original), assigned;
    assigned = copied;
    copied.SetName("changed");
    copied.SetParentName("changed_parent");
    copied.SetRawPose(gz::math::Pose3d(4, 5, 6, 0.1, 0.2, 0.3));
    auto axis = *copied.Axis();
    const auto errors = axis.SetXyz(gz::math::Vector3d::UnitY);
    if (!errors.empty()) return 4;
    copied.SetAxis(0, axis);
    for (const sdf::Joint *value : {original, static_cast<const sdf::Joint *>(&assigned), static_cast<const sdf::Joint *>(&copied)}) {
      std::cout << "COPY"; text(value->Name()); text(value->ParentName());
      pose(value->RawPose()); const auto xyz = value->Axis()->Xyz();
      number(xyz.X()); number(xyz.Y()); number(xyz.Z()); std::cout << '\n';
    }
    return 0;
  }
  std::cout << "ROOT"; text(root.Version()); std::cout << '\n';
  std::cout << "MODEL"; text(model->Name());
  std::cout << ' ' << model->LinkCount() << ' ' << model->JointCount();
  pose(model->RawPose()); std::cout << '\n';
  std::cout << "CANONICAL"; text(model->CanonicalLinkName()); std::cout << '\n';
  std::cout << "ORDER";
  for (auto value = model->Element()->GetFirstElement(); value; value = value->GetNextElement()) {
    text(value->GetName());
  }
  std::cout << '\n';
  for (uint64_t i = 0; i < model->LinkCount(); ++i) {
    const auto *link = model->LinkByIndex(i);
    std::cout << "LINK"; text(link->Name()); pose(link->RawPose());
    std::cout << ' ' << link->VisualCount() << ' ' << link->CollisionCount() << '\n';
    const auto inertial = link->Inertial(); const auto mass = inertial.MassMatrix();
    std::cout << "INERTIAL";
    for (double n : {mass.Mass(), mass.Ixx(), mass.Iyy(), mass.Izz(), mass.Ixy(), mass.Ixz(), mass.Iyz()}) number(n);
    pose(inertial.Pose()); std::cout << '\n';
    std::cout << "LINKFRAME"; text(link->PoseRelativeTo()); std::cout << '\n';
    resolved("link", link->SemanticPose());
    gz::math::Inertiald global;
    const auto inertiaErrors = link->ResolveInertial(global, "__model__");
    std::cout << "RESOLVED inertial " << inertiaErrors.size();
    if (inertiaErrors.empty()) pose(global.Pose());
    std::cout << '\n';
    for (uint64_t j = 0; j < link->VisualCount(); ++j) {
      const auto *visual = link->VisualByIndex(j);
      std::cout << "VISUAL"; text(visual->Name()); pose(visual->RawPose());
      std::cout << ' '; geometry(visual->Geom()); std::cout << '\n';
      if (const auto *material = visual->Material()) {
        std::cout << "MATERIAL"; text(material->ScriptName()); text(material->ScriptUri()); std::cout << '\n';
      }
      resolved("visual", visual->SemanticPose());
    }
    for (uint64_t j = 0; j < link->CollisionCount(); ++j) {
      const auto *collision = link->CollisionByIndex(j);
      std::cout << "COLLISION"; text(collision->Name()); pose(collision->RawPose());
      std::cout << ' '; geometry(collision->Geom()); std::cout << '\n';
      const auto *friction = collision->Surface()->Friction();
      std::cout << "MU";
      number(friction && friction->ODE() ? friction->ODE()->Mu() : 1.);
      number(friction && friction->ODE() ? friction->ODE()->Mu2() : 1.);
      std::cout << '\n';
      resolved("collision", collision->SemanticPose());
    }
  }
  for (uint64_t i = 0; i < model->JointCount(); ++i) {
    const auto *joint = model->JointByIndex(i);
    std::cout << "JOINT"; text(joint->Name()); std::cout << ' ' << int(joint->Type());
    text(joint->ParentName()); text(joint->ChildName()); text(joint->PoseRelativeTo());
    pose(joint->RawPose()); std::cout << '\n';
    resolved("joint", joint->SemanticPose());
    std::string parent, child;
    auto parentErrors = joint->ResolveParentLink(parent);
    auto childErrors = joint->ResolveChildLink(child);
    std::cout << "ENDPOINTS " << parentErrors.size() << ' ' << childErrors.size();
    text(parent); text(child); std::cout << '\n';
    if (const auto *axis = joint->Axis()) {
      std::cout << "AXIS";
      const auto xyz = axis->Xyz();
      for (double n : {xyz.X(), xyz.Y(), xyz.Z(), axis->Lower(), axis->Upper(),
                      axis->Effort(), axis->MaxVelocity(), axis->Damping(), axis->Friction(),
                      axis->SpringReference(), axis->SpringStiffness(), axis->Stiffness(), axis->Dissipation()}) number(n);
      text(axis->XyzExpressedIn()); std::cout << '\n';
      gz::math::Vector3d global;
      auto axisErrors = axis->ResolveXyz(global, "__model__");
      std::cout << "RESOLVED axis " << axisErrors.size();
      if (axisErrors.empty()) { number(global.X()); number(global.Y()); number(global.Z()); }
      std::cout << '\n';
    }
  }
}
