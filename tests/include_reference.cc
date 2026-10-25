// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <sdf/Root.hh>
#include <sdf/Model.hh>
#include <sdf/World.hh>
#include <sdf/Plugin.hh>
#include <sdf/Element.hh>
#include <sdf/ParserConfig.hh>
#include <iostream>
#include <iomanip>
#include <iterator>
#include <cstdint>
#include <cstring>
void Hex(const std::string &s){for(unsigned char c:s)std::cout<<std::hex<<std::setw(2)<<std::setfill('0')<<unsigned(c);std::cout<<std::dec;}
void Field(const std::string &key,const std::string &value){std::cout<<key<<"\t";Hex(value);std::cout<<"\n";}
void Pair(const std::string &key,const std::string &name,const std::string &file){std::cout<<key<<"\t";Hex(name);std::cout<<"\t";Hex(file);std::cout<<"\n";}
int main(int argc,char **argv){
 if(argc!=2)return 2;
 try {
 sdf::ParserConfig config;config.SetFindCallback([&](const std::string &uri){Field("request",uri);return std::string(argv[1]);});
 std::string xml((std::istreambuf_iterator<char>(std::cin)),{});sdf::Root root;auto errors=root.LoadSdfString(xml,config);
 if(!errors.empty()){std::cout<<"accepted\t0\n";return 0;}
 std::cout<<"accepted\t1\n";auto model=root.Model();if(!model)model=root.WorldByIndex(0)->ModelByIndex(0);else if(model->ModelCount())model=model->ModelByIndex(0);
 Field("name",model->Name());Field("uri",model->Uri());Field("model_static",model->Static()?"true":"false");for(auto &plugin:model->Plugins())Pair("model_plugin",plugin.Name(),plugin.Filename());
 auto include=model->Element()->GetIncludeElement();std::cout<<"include\t"<<(include?1:0)<<"\n";if(!include)return 0;
 for(auto name:{"uri","name","static","placement_frame"}){
  bool present=include->HasElement(name);std::cout<<"has_"<<name<<"\t"<<present<<"\n";
  if(present)Field(std::string("include_")+name,include->Get<std::string>(name));
 }
 bool hasPose=include->HasElement("pose");std::cout<<"has_pose\t"<<hasPose<<"\n";
 if(hasPose){auto pose=include->Get<gz::math::Pose3d>("pose");double values[]={pose.Pos().X(),pose.Pos().Y(),pose.Pos().Z(),pose.Rot().X(),pose.Rot().Y(),pose.Rot().Z(),pose.Rot().W()};std::cout<<"pose_bits";for(double value:values){std::uint64_t bits;std::memcpy(&bits,&value,8);std::cout<<"\t"<<bits;}std::cout<<"\n";auto elem=include->GetElement("pose");Field("pose_relative_to",elem->Get<std::string>("relative_to","fixture-default").first);}
 if(include->HasElement("plugin"))for(auto plugin=include->GetElement("plugin");plugin;plugin=plugin->GetNextElement("plugin"))Pair("include_plugin",plugin->Get<std::string>("name"),plugin->Get<std::string>("filename"));
 } catch(const std::exception &error) {Field("exception",error.what());}
}
