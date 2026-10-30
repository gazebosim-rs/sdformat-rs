// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Surface.hh>
#include <sdf/Element.hh>
#include <sdf/SDFImpl.hh>
#include <sdf/parser.hh>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <iterator>
void number(double value) {
  uint64_t bits; std::memcpy(&bits, &value, sizeof(bits)); std::cout << ' ' << bits;
}
void direction(const gz::math::Vector3d &v) {
  for (double value : {v.X(), v.Y(), v.Z()}) number(value);
}
void values(const sdf::Surface &surface) {
  auto contact = surface.Contact(); auto friction = surface.Friction();
  std::cout << "OK " << contact->CollideBitmask() << ' ';
  if (contact->CategoryBitmask()) std::cout << *contact->CategoryBitmask(); else std::cout << '-';
  auto ode = friction->ODE();
  for (double value : {ode->Mu(), ode->Mu2(), ode->Slip1(), ode->Slip2()}) number(value);
  direction(ode->Fdir1());
  auto bullet = friction->BulletFriction(); std::cout << ' ' << bool(bullet);
  if (bullet) {
    for (double value : {bullet->Friction(), bullet->Friction2(), bullet->RollingFriction()}) number(value);
    direction(bullet->Fdir1());
  }
  auto torsional = friction->Torsional(); std::cout << ' ' << bool(torsional);
  if (torsional) {
    for (double value : {torsional->Coefficient(), torsional->PatchRadius(), torsional->SurfaceRadius(), torsional->ODESlip()}) number(value);
    std::cout << ' ' << torsional->UsePatchRadius();
  }
  std::cout << '\n';
}
int main(int argc, char **argv) {
  if (argc == 2) {
    const std::string mode = argv[1];
    if (mode != "--fixture" && mode != "--quiet-fixture") return 2;
    sdf::Surface surface; sdf::Contact contact; sdf::Friction friction; sdf::ODE ode;
    contact.SetCollideBitmask(0x12); contact.SetCategoryBitmask(0x21);
    ode.SetMu(0.1); ode.SetMu2(0.2); ode.SetSlip1(3); ode.SetSlip2(4); ode.SetFdir1({1,2,3});
    friction.SetODE(ode);
    if (mode == "--fixture") {
      sdf::BulletFriction bullet; bullet.SetFriction(0.3); bullet.SetFriction2(0.5);
      bullet.SetFdir1({2,1,4}); bullet.SetRollingFriction(1.3); friction.SetBulletFriction(bullet);
      sdf::Torsional torsional; torsional.SetCoefficient(0.5); torsional.SetUsePatchRadius(false);
      torsional.SetPatchRadius(0.1); torsional.SetSurfaceRadius(0.3); torsional.SetODESlip(0.2);
      friction.SetTorsional(torsional);
    }
    surface.SetContact(contact); surface.SetFriction(friction); sdf::Errors errors;
    auto element = surface.ToElement(errors); if (!element || !errors.empty()) return 3;
    std::cout << element->ToString(""); return 0;
  }
  if (argc != 1) return 2;
  const std::string input{std::istreambuf_iterator<char>(std::cin), {}};
  sdf::SDFPtr document(new sdf::SDF); sdf::init(document); sdf::Errors errors;
  bool parsed = sdf::readString("<sdf version='1.12'><model name='m'><link name='l'><collision name='c'><geometry><box/></geometry>" + input + "</collision></link></model></sdf>", document, errors);
  if (!parsed || !errors.empty()) {std::cout << "REJECT\n"; return 0;}
  auto collision = document->Root()->GetElement("model")->GetElement("link")->GetElement("collision");
  if (!collision->HasElement("surface")) {std::cout << "REJECT\n"; return 0;}
  sdf::Surface surface; errors = surface.Load(collision->GetElement("surface"));
  if (!errors.empty()) {std::cout << "REJECT\n"; return 0;}
  values(surface);
}
