// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/Model.hh>
#include <sdf/Link.hh>
#include <iostream>
#include <iterator>
#include <iomanip>
void hex(const std::string &v) {for(unsigned char c:v)std::cout<<std::hex<<std::setfill('0')<<std::setw(2)<<unsigned(c);}
int main() {
 std::string xml((std::istreambuf_iterator<char>(std::cin)),{}); sdf::Root root;auto errors=root.LoadSdfString(xml);
 if(!errors.empty()){ std::cout << "error\n"; for(auto &e:errors)std::cerr<<e.Message()<<"\n";return 0; }
 auto result=root.Model()->CanonicalLinkAndRelativeName();std::cout << (result.first?1:0) << "\n";hex(result.second);std::cout << "\n";
 if(result.first)hex(result.first->Name());std::cout << "\n";
}
