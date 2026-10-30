/*
 * Copyright (C) 2018 Open Source Robotics Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
*/

// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Collision.hh>
#include <sdf/Geometry.hh>
#include <sdf/Surface.hh>
#include <sdf/Element.hh>
#include <sdf/SDFImpl.hh>
#include <sdf/parser.hh>
#include <iostream>
#include <iterator>
int main(int argc, char **argv) {
  if (argc > 1 && std::string(argv[1]) == "ctor") {
    sdf::Collision value; value.SetName("my-collision"); value.SetGeom(sdf::Geometry());
    value.SetRawPose(gz::math::Pose3d(1, 2, 3, .1, .2, .3));
    sdf::Surface surface; sdf::Contact contact; contact.SetCollideBitmask(123);
    surface.SetContact(contact); sdf::Friction friction; sdf::ODE ode; ode.SetMu(1.23);
    friction.SetODE(ode); surface.SetFriction(friction); value.SetSurface(surface);
    sdf::Errors errors; auto element = value.ToElement(errors);
    if (!errors.empty() || !element || element->HasElement("density")) return 4;
    std::cout << element->ToString(""); return 0;
  }
  std::string xml{std::istreambuf_iterator<char>(std::cin), {}};
  sdf::SDFPtr doc(new sdf::SDF); sdf::init(doc); sdf::Errors errors;
  bool ok = sdf::readString("<sdf version='1.12'><model name='m'><link name='l'>" + xml + "</link></model></sdf>", doc, errors);
  if (!ok || !errors.empty()) return 1;
  auto element = doc->Root()->GetElement("model")->GetElement("link")->GetElement("collision");
  sdf::Collision value; errors = value.Load(element);
  if (!errors.empty() || value.Name() != "my-collision" || value.RawPose() != gz::math::Pose3d(1, 2, 3, .1, .2, .3) || !value.Geom() || value.Geom()->Type() != sdf::GeometryType::EMPTY || !value.Surface() || !value.Surface()->Contact() || value.Surface()->Contact()->CollideBitmask() != 123 || !value.Surface()->Friction() || !value.Surface()->Friction()->ODE() || value.Surface()->Friction()->ODE()->Mu() != 1.23) return 2;
  bool has = element->HasElement("density");
  if (has != (argc > 1) || (has && (element->Get<double>("density") != 1234.5 || value.Density() != 1234.5))) return 3;
  std::cout << "ok\n";
}
