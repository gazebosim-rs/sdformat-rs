// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Element.hh>
#include <sdf/Light.hh>
#include <sdf/SDFImpl.hh>
#include <sdf/parser.hh>
#include <sdf/Root.hh>
#include <iostream>
int main() {
  for (auto type : {"point", "spot", "directional"}) {
    sdf::ElementPtr raw(new sdf::Element);
    raw->SetName("light"); raw->AddAttribute("name","string","n",true);
    raw->AddAttribute("type","string",type,true);
    sdf::Light direct; auto errors=direct.Load(raw);
    bool missing=false;
    for (const auto &error : errors) if (error.Message().find("<direction> is required")!=std::string::npos) missing=true;
    std::cout << "raw " << type << ' ' << missing << ' ' << raw->HasElement("direction") << ' ' << direct.Direction() << " errors";
    for (const auto &error : errors) std::cout << ' ' << int(error.Code());
    std::cout << '\n';
    sdf::SDFPtr document(new sdf::SDF); sdf::init(document); errors.clear();
    if (!sdf::readString(std::string("<sdf version='1.12'><world name='w'><light name='n' type='")+type+"'/></world></sdf>",document,errors) || !errors.empty()) return 1;
    auto element=document->Root()->GetElement("world")->GetElement("light");
    bool before=element->HasElement("direction"); sdf::Light loaded; errors=loaded.Load(element);
    std::cout << "schema " << type << ' ' << before << ' ' << errors.size() << ' ' << loaded.Direction() << '\n';
  }
  {
    sdf::ElementPtr raw(new sdf::Element); raw->SetName("light");
    raw->AddAttribute("name","string","n",true); raw->AddAttribute("type","string","point",true);
    sdf::ElementPtr attenuation(new sdf::Element); attenuation->SetName("attenuation");raw->InsertElement(attenuation);
    sdf::Light light; bool missing=false;
    for(const auto &error:light.Load(raw))if(error.Message().find("requires a <range>")!=std::string::npos)missing=true;
    std::cout << "raw-empty-attenuation " << missing << '\n';
  }
  for (const auto &xml : {"<sdf version='1.12'><world name='w'><light name='l'/></world></sdf>", "<sdf version='1.12'><model name='m'><link name='l'><light name='light'/></link></model></sdf>"}) {
    sdf::Root root; std::cout << "missing-type";
    for (const auto &error : root.LoadSdfString(xml)) std::cout << ' ' << int(error.Code());
    std::cout << '\n';
  }

}
