// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <cstdint>
#include <iostream>
#include <fstream>
#include <iterator>
#include <iomanip>
#include <optional>
#include <string>
#include <sdf/Element.hh>
#include <sdf/Error.hh>
#include <sdf/Frame.hh>
#include <sdf/Joint.hh>
#include <vector>
#include <sdf/Model.hh>
#include <sdf/Link.hh>
#include <sdf/Root.hh>
#include <sdf/Plugin.hh>
#include <sdf/SemanticPose.hh>
#include <sdf/World.hh>

void hex(const std::string &value)
{
  constexpr char digits[] = "0123456789abcdef";
  std::cout << '"';
  for (unsigned char c : value)
    std::cout << digits[c >> 4] << digits[c & 15];
  std::cout << '"';
}
void optional(const std::optional<std::string> &value)
{
  if (value) hex(*value);
  else std::cout << "null";
}
void resolved(const sdf::SemanticPose &value, const std::string &target)
{
  gz::math::Pose3d result;
  const auto errors = value.Resolve(result, target);
  std::cout << "{\"errors\":[";
  for (std::size_t i = 0; i < errors.size(); ++i)
  {
    if (i) std::cout << ',';
    std::cout << "{\"code\":" << static_cast<int>(errors[i].Code()) << ",\"message_hex\":";
    hex(errors[i].Message()); std::cout << '}';
  }
  std::cout << "],\"pose\":";
  if (errors.empty())
    std::cout << '[' << result.Pos().X() << ',' << result.Pos().Y() << ',' << result.Pos().Z()
              << ',' << result.Rot().X() << ',' << result.Rot().Y() << ',' << result.Rot().Z()
              << ',' << result.Rot().W() << ']';
  else std::cout << "null";
  std::cout << '}';
}
void targets(const sdf::SemanticPose &value)
{
  std::cout << "{\"default\":"; resolved(value, "");
  std::cout << ",\"root\":"; resolved(value, "__root__");
  std::cout << ",\"absolute_root\":"; resolved(value, "::__root__");
  std::cout << ",\"model\":"; resolved(value, "__model__"); std::cout << '}';
}
void model(const sdf::Model &value)
{
  std::cout << "{\"name_hex\":"; hex(value.Name());
  std::cout << ",\"uri_hex\":"; hex(value.Uri());
  std::cout << ",\"element_file_hex\":";
  if (value.Element()) hex(value.Element()->FilePath());
  else std::cout << "null";
  std::cout << ",\"raw_hex\":"; optional(value.RawNamespace());
  std::cout << ",\"resolved_hex\":"; optional(value.Namespace());
  std::cout << ",\"static\":" << (value.Static() ? "true" : "false");
  std::cout << ",\"canonical_hex\":"; hex(value.CanonicalLinkName());
  std::cout << ",\"placement_hex\":"; hex(value.PlacementFrameName());
  std::cout << ",\"relative_hex\":"; hex(value.PoseRelativeTo());
  const auto &pose = value.RawPose();
  std::cout << ",\"pose\":[" << pose.Pos().X() << ',' << pose.Pos().Y() << ',' << pose.Pos().Z()
            << ',' << pose.Rot().X() << ',' << pose.Rot().Y() << ',' << pose.Rot().Z()
            << ',' << pose.Rot().W() << ']';
  gz::math::Pose3d resolved;
  const auto poseErrors = value.SemanticPose().Resolve(resolved);
  std::cout << ",\"pose_errors\":[";
  for (std::size_t i = 0; i < poseErrors.size(); ++i)
  {
    if (i) std::cout << ',';
    std::cout << "{\"code\":" << static_cast<int>(poseErrors[i].Code()) << ",\"message_hex\":";
    hex(poseErrors[i].Message()); std::cout << '}';
  }
  std::cout << "],\"resolved_pose\":";
  if (poseErrors.empty())
    std::cout << '[' << resolved.Pos().X() << ',' << resolved.Pos().Y() << ',' << resolved.Pos().Z()
              << ',' << resolved.Rot().X() << ',' << resolved.Rot().Y() << ',' << resolved.Rot().Z()
              << ',' << resolved.Rot().W() << ']';
  else std::cout << "null";
  std::cout << ",\"plugins\":[";
  bool firstPlugin = true;
  for (const auto &plugin : value.Plugins())
  {
    if (!firstPlugin) std::cout << ',';
    firstPlugin = false;
    std::cout << "{\"name_hex\":"; hex(plugin.Name());
    std::cout << ",\"filename_hex\":"; hex(plugin.Filename()); std::cout << '}';
  }
  std::cout << ']';
  std::cout << ",\"links_data\":[";
  for (uint64_t i = 0; i < value.LinkCount(); ++i)
  {
    if (i) std::cout << ',';
    const auto *link = value.LinkByIndex(i);
    std::cout << "{\"name_hex\":"; hex(link->Name());
    std::cout << ",\"semantic\":"; targets(link->SemanticPose()); std::cout << '}';
  }
  std::cout << "],\"frames_data\":[";
  for (uint64_t i = 0; i < value.FrameCount(); ++i)
  {
    if (i) std::cout << ',';
    const auto *frame = value.FrameByIndex(i);
    std::cout << "{\"name_hex\":"; hex(frame->Name());
    std::cout << ",\"semantic\":"; targets(frame->SemanticPose()); std::cout << '}';
  }
  std::cout << ']';
  std::cout << ",\"original_version_hex\":";
  if (value.Element()) hex(value.Element()->OriginalVersion());
  else std::cout << "null";
  std::cout << ",\"links\":" << value.LinkCount() << ",\"frames_hex\":[";
  for (uint64_t i = 0; i < value.FrameCount(); ++i)
  {
    if (i) std::cout << ',';
    hex(value.FrameByIndex(i)->Name());
  }
  std::cout << "],\"models\":[";
  for (uint64_t i = 0; i < value.ModelCount(); ++i)
  {
    if (i) std::cout << ',';
    model(*value.ModelByIndex(i));
  }
  std::cout << "]}";
}

void pose(const gz::math::Pose3d &value)
{
  std::cout << '[' << value.Pos().X() << ',' << value.Pos().Y() << ','
            << value.Pos().Z() << ',' << value.Rot().X() << ','
            << value.Rot().Y() << ',' << value.Rot().Z() << ','
            << value.Rot().W() << ']';
}
void body(const std::string &value, const sdf::Errors &errors)
{
  std::cout << "{\"errors\":[";
  for (std::size_t i = 0; i < errors.size(); ++i)
  {
    if (i) std::cout << ',';
    std::cout << "{\"code\":" << static_cast<int>(errors[i].Code())
              << ",\"message_hex\":";
    hex(errors[i].Message()); std::cout << '}';
  }
  std::cout << "],\"body_hex\":";
  if (errors.empty()) hex(value);
  else std::cout << "null";
  std::cout << '}';
}
void worldTargets(const sdf::SemanticPose &value, const sdf::World &world)
{
  std::vector<std::string> names{"", "world", "__root__"};
  for (uint64_t i = 0; i < world.ModelCount(); ++i)
  {
    names.push_back(world.ModelByIndex(i)->Name());
    names.push_back(world.ModelByIndex(i)->Name() + "::link");
  }
  for (uint64_t i = 0; i < world.FrameCount(); ++i)
    names.push_back(world.FrameByIndex(i)->Name());
  for (uint64_t i = 0; i < world.JointCount(); ++i)
    names.push_back(world.JointByIndex(i)->Name());
  std::cout << '[';
  for (std::size_t i = 0; i < names.size(); ++i)
  {
    if (i) std::cout << ',';
    std::cout << "{\"target_hex\":"; hex(names[i]);
    std::cout << ",\"resolved\":"; resolved(value, names[i]); std::cout << '}';
  }
  std::cout << ']';
}
void worldExtra(const sdf::World &world)
{
  std::cout << ",\"element_file_hex\":";
  if (world.Element()) hex(world.Element()->FilePath());
  else std::cout << "null";
  std::cout << ",\"original_version_hex\":";
  if (world.Element()) hex(world.Element()->OriginalVersion());
  else std::cout << "null";
  std::cout << ",\"frames_data\":[";
  for (uint64_t i = 0; i < world.FrameCount(); ++i)
  {
    if (i) std::cout << ',';
    const auto &frame = *world.FrameByIndex(i);
    std::cout << "{\"name_hex\":"; hex(frame.Name());
    std::cout << ",\"attached_to_hex\":"; hex(frame.AttachedTo());
    std::cout << ",\"relative_hex\":"; hex(frame.PoseRelativeTo());
    std::cout << ",\"element_file_hex\":";
    if (frame.Element()) hex(frame.Element()->FilePath());
    else std::cout << "null";
    std::cout << ",\"raw_pose\":"; pose(frame.RawPose());
    std::string result;
    const auto errors = frame.ResolveAttachedToBody(result);
    std::cout << ",\"attached_body\":"; body(result, errors);
    std::cout << ",\"semantic\":"; worldTargets(frame.SemanticPose(), world);
    std::cout << '}';
  }
  std::cout << "],\"joints_data\":[";
  for (uint64_t i = 0; i < world.JointCount(); ++i)
  {
    if (i) std::cout << ',';
    const auto &joint = *world.JointByIndex(i);
    std::cout << "{\"name_hex\":"; hex(joint.Name());
    std::cout << ",\"type\":" << static_cast<int>(joint.Type());
    std::cout << ",\"parent_hex\":"; hex(joint.ParentName());
    std::cout << ",\"child_hex\":"; hex(joint.ChildName());
    std::cout << ",\"relative_hex\":"; hex(joint.PoseRelativeTo());
    std::cout << ",\"element_file_hex\":";
    if (joint.Element()) hex(joint.Element()->FilePath());
    else std::cout << "null";
    std::cout << ",\"raw_pose\":"; pose(joint.RawPose());
    std::string parent, child;
    const auto parentErrors = joint.ResolveParentLink(parent);
    const auto childErrors = joint.ResolveChildLink(child);
    std::cout << ",\"parent_body\":"; body(parent, parentErrors);
    std::cout << ",\"child_body\":"; body(child, childErrors);
    std::cout << ",\"semantic\":"; worldTargets(joint.SemanticPose(), world);
    std::cout << '}';
  }
  std::cout << "],\"plugins\":[";
  bool first = true;
  for (const auto &plugin : world.Plugins())
  {
    if (!first) std::cout << ',';
    first = false;
    std::cout << "{\"name_hex\":"; hex(plugin.Name());
    std::cout << ",\"filename_hex\":"; hex(plugin.Filename()); std::cout << '}';
  }
  std::cout << ']';
}

int main(int argc, char **argv)
{
  if (argc != 2) return 2;
  std::cout << std::setprecision(17);
  sdf::Root root;
  std::ifstream input(argv[1]);
  if (!input) return 3;
  const std::string xml((std::istreambuf_iterator<char>(input)), {});
  const auto errors = root.LoadSdfString(xml);
  std::cout << "{\"errors\":[";
  for (std::size_t i = 0; i < errors.size(); ++i)
  {
    if (i) std::cout << ',';
    std::cout << "{\"code\":" << static_cast<int>(errors[i].Code()) << ",\"message_hex\":";
    hex(errors[i].Message());
    std::cout << '}';
  }
  std::cout << "],\"root_model\":";
  if (root.Model()) model(*root.Model());
  else std::cout << "null";
  std::cout << ",\"worlds\":[";
  for (uint64_t i = 0; i < root.WorldCount(); ++i)
  {
    if (i) std::cout << ',';
    const auto *world = root.WorldByIndex(i);
    std::cout << "{\"name_hex\":"; hex(world->Name());
    std::cout << ",\"models\":[";
    for (uint64_t j = 0; j < world->ModelCount(); ++j)
    {
      if (j) std::cout << ',';
      model(*world->ModelByIndex(j));
    }
    std::cout << ']';
    worldExtra(*world);
    std::cout << '}';
  }
  std::cout << "]}\n";
  return 0;
}
