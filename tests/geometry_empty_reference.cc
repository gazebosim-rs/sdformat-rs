// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Geometry.hh>
#include <sdf/Polyline.hh>
#include <sdf/Collision.hh>
#include <sdf/Visual.hh>
#include <sdf/Root.hh>
#include <sdf/Model.hh>
#include <sdf/Link.hh>
#include <sdf/Element.hh>
#include <sdf/SDFImpl.hh>
#include <sdf/parser.hh>
#include <iostream>
#include <iterator>

void errors(const char *name, const sdf::Errors &errors)
{
  std::cout << name;
  for (const auto &error : errors) std::cout << ' ' << int(error.Code());
  std::cout << '\n';
}
void values(const sdf::Geometry &geometry)
{
  const unsigned mask = bool(geometry.BoxShape()) |
      (bool(geometry.CapsuleShape()) << 1) |
      (bool(geometry.ConeShape()) << 2) |
      (bool(geometry.CylinderShape()) << 3) |
      (bool(geometry.EllipsoidShape()) << 4) |
      (bool(geometry.SphereShape()) << 5) |
      (bool(geometry.PlaneShape()) << 6) |
      (bool(geometry.MeshShape()) << 7) |
      (bool(geometry.HeightmapShape()) << 8) |
      (!geometry.PolylineShape().empty() << 9);
  std::cout << "VALUES " << int(geometry.Type()) << ' ' << mask << ' '
            << geometry.PolylineShape().size() << '\n';
}
int main(int argc, char **argv)
{
  const std::string mode = argc > 1 ? argv[1] : "geometry";
  if (mode == "constructor")
  {
    sdf::Geometry geometry;
    values(geometry);
    sdf::Errors result;
    auto element = geometry.ToElement(result);
    std::cout << "EMIT " << bool(element) << '\n';
    errors("ERRORS", result);
    if (!element) return 2;
    std::cout << "XML_BEGIN\n" << element->ToString("") << "XML_END\n";
    sdf::Geometry loaded;
    result = loaded.Load(element);
    errors("RELOAD", result);
    values(loaded);
    return 0;
  }
  const std::string input{std::istreambuf_iterator<char>(std::cin), {}};
  const std::string document = mode == "document" ? input :
      "<sdf version='1.12'><model name='m'><link name='l'>" +
      (mode == "geometry" ? "<collision name='c'>" + input + "</collision>" : input) +
      "</link></model></sdf>";
  sdf::SDFPtr schema(new sdf::SDF);
  sdf::init(schema);
  sdf::Errors result;
  const bool ok = sdf::readString(document, schema, result);
  std::cout << "READ " << ok << '\n';
  errors("SCHEMA", result);
  if (!ok) return 0;
  auto link = schema->Root()->GetElement("model")->GetElement("link");
  auto container = link->HasElement("visual") ? link->GetElement("visual") :
      link->GetElement("collision");
  std::cout << "DECLARED_GEOMETRY " << container->HasElement("geometry") << '\n';
  sdf::Geometry geometry;
  result = geometry.Load(container->GetElement("geometry"));
  errors("GEOMETRY_LOAD", result);
  values(geometry);
  sdf::Root root;
  result = root.LoadSdfString(document);
  errors("ROOT", result);
  if (root.Model() && root.Model()->LinkByIndex(0))
  {
    const auto *loadedLink = root.Model()->LinkByIndex(0);
    if (const auto *visual = loadedLink->VisualByIndex(0))
    {
      std::cout << "VISUAL " << bool(visual->Geom()) << '\n';
      values(*visual->Geom());
    }
    if (const auto *collision = loadedLink->CollisionByIndex(0))
    {
      std::cout << "COLLISION " << bool(collision->Geom()) << '\n';
      values(*collision->Geom());
    }
  }
}
