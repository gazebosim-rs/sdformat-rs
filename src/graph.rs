use crate::{Diagnostic, ErrorCode, Frame, Model, Pose, Source, Transform, World};
use petgraph::{algo::toposort, graphmap::DiGraphMap};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Root,
    World,
    Model,
    Body,
    Joint,
    Frame,
}
#[derive(Debug, Clone)]
enum Reference {
    Absolute(String),
    Scoped(String),
}
#[derive(Debug)]
struct Node {
    scope: String,
    pose_parent: Reference,
    attachment: Option<Reference>,
    pose: Transform,
    kind: Kind,
    source: Source,
}
#[derive(Debug, Default)]
pub(crate) struct Graphs {
    nodes: BTreeMap<String, Node>,
    poses: BTreeMap<String, Transform>,
    bodies: BTreeMap<String, String>,
}
impl Graphs {
    pub(crate) fn build(
        worlds: &[World],
        models: &[Model],
        source: &Source,
    ) -> Result<Self, Diagnostic> {
        let mut result = Self::default();
        result.insert(
            "::",
            Node {
                scope: String::new(),
                pose_parent: Reference::Absolute(String::new()),
                attachment: None,
                pose: Transform::default(),
                kind: Kind::Root,
                source: source.clone(),
            },
        )?;
        for world in worlds {
            result.insert(
                &world.name,
                Node {
                    scope: String::new(),
                    pose_parent: Reference::Absolute("::".into()),
                    attachment: None,
                    pose: Transform::default(),
                    kind: Kind::World,
                    source: world.source.clone(),
                },
            )?;
            result.frames(&world.frames, &world.name)?;
            for model in &world.models {
                result.model(model, &world.name)?;
            }
        }
        for model in models {
            result.model(model, "::")?;
        }
        result.validate()?;
        Ok(result)
    }
    fn insert(&mut self, name: &str, node: Node) -> Result<(), Diagnostic> {
        if self.nodes.contains_key(name) {
            return Err(error(
                ErrorCode::DuplicateName,
                &node.source,
                format!("duplicate frame name {name}"),
            ));
        }
        self.nodes.insert(name.into(), node);
        Ok(())
    }
    fn model(&mut self, model: &Model, parent: &str) -> Result<(), Diagnostic> {
        let name = &model.scoped_name;
        let attachment = if model.is_static {
            Reference::Absolute(self.lookup("world", parent, &model.source)?)
        } else {
            let name = if model.canonical_link.is_empty() {
                model
                    .links
                    .first()
                    .map(|link| link.name.clone())
                    .or_else(|| model.models.first().map(|nested| nested.name.clone()))
                    .ok_or_else(|| {
                        error(
                            ErrorCode::Missing,
                            &model.source,
                            "dynamic model requires a link or nested model",
                        )
                    })?
            } else {
                model.canonical_link.clone()
            };
            Reference::Scoped(name)
        };
        self.insert(
            name,
            Node {
                scope: parent.into(),
                pose_parent: reference(&model.pose.relative_to, Reference::Absolute(parent.into())),
                attachment: Some(attachment),
                pose: model.pose.transform,
                kind: Kind::Model,
                source: model.source.clone(),
            },
        )?;
        for link in &model.links {
            self.insert(
                &link.scoped_name,
                Node {
                    scope: name.clone(),
                    pose_parent: reference(
                        &link.pose.relative_to,
                        Reference::Absolute(name.clone()),
                    ),
                    attachment: None,
                    pose: link.pose.transform,
                    kind: Kind::Body,
                    source: link.source.clone(),
                },
            )?;
        }
        self.frames(&model.frames, name)?;
        for joint in &model.joints {
            self.insert(
                &join(name, &joint.name),
                Node {
                    scope: name.clone(),
                    pose_parent: reference(
                        &joint.pose.relative_to,
                        Reference::Scoped(joint.child.clone()),
                    ),
                    attachment: Some(Reference::Scoped(joint.child.clone())),
                    pose: joint.pose.transform,
                    kind: Kind::Joint,
                    source: joint.source.clone(),
                },
            )?;
        }
        for nested in &model.models {
            self.model(nested, name)?;
        }
        Ok(())
    }
    fn frames(&mut self, frames: &[Frame], scope: &str) -> Result<(), Diagnostic> {
        for frame in frames {
            let attached = reference(&frame.attached_to, Reference::Absolute(scope.into()));
            self.insert(
                &join(scope, &frame.name),
                Node {
                    scope: scope.into(),
                    pose_parent: reference(&frame.pose.relative_to, attached.clone()),
                    attachment: Some(attached),
                    pose: frame.pose.transform,
                    kind: Kind::Frame,
                    source: frame.source.clone(),
                },
            )?;
        }
        Ok(())
    }
    fn lookup(&self, value: &str, scope: &str, source: &Source) -> Result<String, Diagnostic> {
        let name = if value == "__model__" {
            scope.into()
        } else if value == "world" {
            let mut ancestor = scope;
            loop {
                if self
                    .nodes
                    .get(ancestor)
                    .is_some_and(|n| n.kind == Kind::World)
                {
                    break ancestor.to_string();
                }
                match ancestor.rsplit_once("::") {
                    Some((parent, _)) => ancestor = parent,
                    None => break "::".into(),
                }
            }
        } else if value == "::" || value == scope || value.starts_with(&format!("{scope}::")) {
            value.into()
        } else {
            join(scope, value)
        };
        if !self.nodes.contains_key(&name) {
            return Err(error(
                ErrorCode::UnknownReference,
                source,
                format!("unknown frame {value} in {scope}"),
            ));
        }
        Ok(name)
    }
    fn schema_lookup(
        &self,
        value: &str,
        scope: &str,
        source: &Source,
    ) -> Result<String, Diagnostic> {
        let kind = self.nodes.get(scope).map(|node| node.kind);
        let name = if (value == "__model__" && kind == Some(Kind::Model))
            || (value == "world" && kind == Some(Kind::World))
        {
            scope.into()
        } else {
            join(scope, value)
        };
        if !self.nodes.contains_key(&name) {
            return Err(error(
                ErrorCode::UnknownReference,
                source,
                format!("unknown local frame {value} in {scope}"),
            ));
        }
        Ok(name)
    }
    fn resolve_reference(
        &self,
        reference: &Reference,
        scope: &str,
        source: &Source,
    ) -> Result<String, Diagnostic> {
        match reference {
            Reference::Scoped(value) => self.schema_lookup(value, scope, source),
            Reference::Absolute(value) => {
                if !self.nodes.contains_key(value) {
                    return Err(error(
                        ErrorCode::UnknownReference,
                        source,
                        format!("unknown default frame {value}"),
                    ));
                }
                Ok(value.clone())
            }
        }
    }
    fn validate(&mut self) -> Result<(), Diagnostic> {
        let mut pose_graph = DiGraphMap::<&str, ()>::new();
        let mut attachment_graph = DiGraphMap::<&str, ()>::new();
        let mut parents = BTreeMap::new();
        let mut attachments = BTreeMap::new();
        for (name, node) in &self.nodes {
            pose_graph.add_node(name);
            attachment_graph.add_node(name);
            if node.kind != Kind::Root {
                let parent =
                    self.resolve_reference(&node.pose_parent, &node.scope, &node.source)?;
                parents.insert(name.clone(), parent);
                pose_graph.add_edge(
                    name,
                    self.nodes
                        .get_key_value(&parents[name])
                        .expect("validated lookup")
                        .0
                        .as_str(),
                    (),
                );
            }
            if let Some(attachment) = &node.attachment {
                // A model's canonical link is resolved inside the model itself.
                let scope = if node.kind == Kind::Model {
                    name
                } else {
                    &node.scope
                };
                let target = self.resolve_reference(attachment, scope, &node.source)?;
                attachments.insert(name.clone(), target);
                attachment_graph.add_edge(
                    name,
                    self.nodes
                        .get_key_value(&attachments[name])
                        .expect("validated lookup")
                        .0
                        .as_str(),
                    (),
                );
            }
        }
        let pose_order = toposort(&pose_graph, None).map_err(|cycle| {
            error(
                ErrorCode::Cycle,
                &self.nodes[cycle.node_id()].source,
                "pose graph cycle",
            )
        })?;
        let attachment_order = toposort(&attachment_graph, None).map_err(|cycle| {
            error(
                ErrorCode::Cycle,
                &self.nodes[cycle.node_id()].source,
                "attachment graph cycle",
            )
        })?;
        for name in pose_order.into_iter().rev() {
            let node = &self.nodes[name];
            let pose = parents
                .get(name)
                .map_or(node.pose, |parent| self.poses[parent].compose(node.pose));
            pose.validate(&node.source)?;
            self.poses.insert(name.into(), pose);
        }
        for name in attachment_order.into_iter().rev() {
            let body = attachments
                .get(name)
                .map_or_else(|| name.to_string(), |target| self.bodies[target].clone());
            self.bodies.insert(name.into(), body);
        }
        Ok(())
    }
    pub(crate) fn resolve_name(&self, scope: &str, value: &str) -> Result<String, Diagnostic> {
        self.lookup(value, scope, &self.nodes["::"].source)
    }
    pub(crate) fn validate_reference(
        &self,
        value: &str,
        scope: &str,
        source: &Source,
    ) -> Result<String, Diagnostic> {
        self.schema_lookup(value, scope, source)
    }
    pub(crate) fn joint_parent(
        &self,
        value: &str,
        scope: &str,
        source: &Source,
    ) -> Result<String, Diagnostic> {
        if value == "world" {
            self.lookup(value, scope, source)
        } else {
            self.schema_lookup(value, scope, source)
        }
    }
    pub(crate) fn axis_frame(&self, scope: &str, value: &str) -> Result<String, Diagnostic> {
        self.schema_lookup(value, scope, &self.nodes["::"].source)
    }
    pub(crate) fn pose_of(&self, name: &str, target: &str) -> Result<Transform, Diagnostic> {
        let source = &self.nodes["::"].source;
        let a = self.poses.get(name).ok_or_else(|| {
            error(
                ErrorCode::UnknownReference,
                source,
                format!("unknown frame {name}"),
            )
        })?;
        let b = self.poses.get(target).ok_or_else(|| {
            error(
                ErrorCode::UnknownReference,
                source,
                format!("unknown frame {target}"),
            )
        })?;
        let pose = b.inverse().compose(*a);
        pose.validate(source)?;
        Ok(pose)
    }
    pub(crate) fn resolve_pose(
        &self,
        pose: &Pose,
        scope: &str,
        default_frame: &str,
        target: &str,
    ) -> Result<Transform, Diagnostic> {
        pose.transform.validate(&self.nodes["::"].source)?;
        let parent = self.resolve_reference(
            &reference(&pose.relative_to, Reference::Absolute(default_frame.into())),
            scope,
            &self.nodes["::"].source,
        )?;
        let resolved = self.pose_of(&parent, target)?.compose(pose.transform);
        resolved.validate(&self.nodes["::"].source)?;
        Ok(resolved)
    }
    pub(crate) fn attached_body(&self, name: &str) -> Result<&str, Diagnostic> {
        self.bodies.get(name).map(String::as_str).ok_or_else(|| {
            error(
                ErrorCode::UnknownReference,
                &self.nodes["::"].source,
                format!("unknown frame {name}"),
            )
        })
    }
}
pub(crate) fn join(scope: &str, name: &str) -> String {
    if scope == "::" || scope.is_empty() {
        name.into()
    } else {
        format!("{scope}::{name}")
    }
}
fn reference(value: &str, default: Reference) -> Reference {
    if value.is_empty() {
        default
    } else {
        Reference::Scoped(value.into())
    }
}
pub(crate) fn error(code: ErrorCode, source: &Source, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        code,
        source: source.clone(),
        message: message.into(),
    }
}
