// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/Model.hh>
#include <sdf/Link.hh>
#include <sdf/Collision.hh>
#include <sdf/Geometry.hh>
#include <sdf/Heightmap.hh>
#include <sdf/Polyline.hh>
#include <cstring>
#include <iostream>
#include <iterator>
#include <string>
void number(double v) { uint64_t b; std::memcpy(&b, &v, sizeof b); std::cout << ' ' << b; }
void text(const std::string &v) { std::cout << ' '; if (v.empty()) std::cout << '-'; for (unsigned char c:v) { const char *h="0123456789abcdef"; std::cout << h[c/16] << h[c%16]; } }
int main() {
  std::string xml{std::istreambuf_iterator<char>(std::cin), {}};
  sdf::Root root;
  if (!root.LoadSdfString(xml).empty()) { std::cout << "error\n"; return 0; }
  const auto *g=root.Model()->LinkByIndex(0)->CollisionByIndex(0)->Geom();
  if (auto h=g->HeightmapShape()) {
    std::cout << "heightmap"; text(h->Uri());
    for (int i=0;i<3;++i) number(h->Size()[i]);
    for (int i=0;i<3;++i) number(h->Position()[i]);
    std::cout << ' ' << h->UseTerrainPaging() << ' ' << h->Sampling() << ' ' << h->TextureCount();
    for (uint64_t i=0;i<h->TextureCount();++i) { auto t=h->TextureByIndex(i); number(t->Size());text(t->Diffuse());text(t->Normal()); }
    std::cout << ' ' << h->BlendCount();
    for (uint64_t i=0;i<h->BlendCount();++i) {auto b=h->BlendByIndex(i);number(b->MinHeight());number(b->FadeDistance());}
  } else {
    std::cout << "polyline " << g->PolylineShape().size();
    for (auto &p:g->PolylineShape()) {number(p.Height());std::cout << ' ' << p.PointCount(); for (auto &v:p.Points()) {number(v.X());number(v.Y());}}
  }
  std::cout << '\n';
}
