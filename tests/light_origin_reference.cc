// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Light.hh>
#include <sdf/Element.hh>
#include <iostream>
int main() {
  sdf::Light value;
  std::cout << bool(value.Element()) << ' ';
  value.SetName("origin_light");
  sdf::Errors errors;
  auto emitted = value.ToElement(errors);
  if (!errors.empty() || !emitted) return 1;
  std::cout << bool(value.Element()) << ' ';
  emitted->SetFilePath("input-light.sdf");
  emitted->SetLineNumber(7);
  sdf::Light loaded;
  if (!loaded.Load(emitted).empty()) return 2;
  std::cout << bool(loaded.Element()) << ' ' << loaded.Element()->FilePath()
            << ' ' << loaded.Element()->LineNumber().value_or(-1) << ' ';
  loaded.SetCastShadows(true);
  auto edited = loaded.ToElement(errors);
  if (!errors.empty() || !edited) return 3;
  std::cout << bool(loaded.Element()) << ' ' << loaded.Element()->FilePath()
            << ' ' << loaded.Element()->LineNumber().value_or(-1) << '\n';
}
