use crate::graph::Graphs;
use glam::{DQuat, DVec3};

/// Source identity is supplied by the caller, or the injected include resolver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub uri: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Xml,
    Version,
    Missing,
    InvalidValue,
    DuplicateName,
    UnknownReference,
    Cycle,
    Resource,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: ErrorCode,
    pub source: Source,
    pub message: String,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}: {:?}: {}",
            self.source.uri, self.source.line, self.source.column, self.code, self.message
        )
    }
}
impl std::error::Error for Diagnostic {}

#[derive(Debug, Clone)]
pub struct Resource {
    pub canonical_uri: String,
    pub xml: String,
}

/// f64 metres and unit quaternion [x, y, z, w].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub translation: [f64; 3],
    pub rotation: [f64; 4],
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: [0.; 3],
            rotation: [0., 0., 0., 1.],
        }
    }
}
impl Transform {
    /// Check numeric invariants for caller-constructed transforms.
    pub fn validate(&self, source: &Source) -> Result<(), Diagnostic> {
        let norm = DQuat::from_array(self.rotation).length_squared();
        if self
            .translation
            .iter()
            .chain(&self.rotation)
            .any(|v| !v.is_finite())
            || (norm - 1.).abs() > 1e-12
        {
            return Err(crate::graph::error(
                ErrorCode::InvalidValue,
                source,
                "transform requires finite values and a unit quaternion",
            ));
        }
        Ok(())
    }

    pub(crate) fn compose(self, rhs: Self) -> Self {
        let q = DQuat::from_array(self.rotation);
        Self {
            translation: (DVec3::from_array(self.translation)
                + q * DVec3::from_array(rhs.translation))
            .to_array(),
            rotation: (q * DQuat::from_array(rhs.rotation)).to_array(),
        }
    }
    pub(crate) fn inverse(self) -> Self {
        let q = DQuat::from_array(self.rotation).conjugate();
        Self {
            translation: (q * -DVec3::from_array(self.translation)).to_array(),
            rotation: q.to_array(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Pose {
    pub transform: Transform,
    pub relative_to: String,
}

#[derive(Debug)]
pub struct Sdf {
    pub version: String,
    pub worlds: Vec<World>,
    pub models: Vec<Model>,
    pub(crate) graphs: Graphs,
}
impl Sdf {
    /// Resolve a semantic frame into another frame. Names are fully scoped,
    /// e.g. `world::robot::link`; the global root is `::`.
    pub fn pose_of(&self, name: &str, target: &str) -> Result<Transform, Diagnostic> {
        self.graphs.pose_of(name, target)
    }
    /// Resolve a collision, sensor or inertial pose. `scope` is its owning model,
    /// `default_frame` is its owning link, and `target` is fully scoped.
    pub fn resolve_pose(
        &self,
        pose: &Pose,
        scope: &str,
        default_frame: &str,
        target: &str,
    ) -> Result<Transform, Diagnostic> {
        self.graphs.resolve_pose(pose, scope, default_frame, target)
    }
    /// Resolve an SDF reference (`world`, `__model__`, or a scoped local name).
    pub fn resolve_name(&self, scope: &str, reference: &str) -> Result<String, Diagnostic> {
        self.graphs.resolve_name(scope, reference)
    }
    /// Rotate a joint axis into a fully scoped target frame, without translation.
    pub fn resolve_axis(
        &self,
        axis: &Axis,
        joint_frame: &str,
        target: &str,
    ) -> Result<[f64; 3], Diagnostic> {
        let frame = if axis.expressed_in.is_empty() {
            joint_frame.into()
        } else {
            let scope = joint_frame
                .rsplit_once("::")
                .map_or("::", |(scope, _)| scope);
            self.resolve_name(scope, &axis.expressed_in)?
        };
        let pose = self.pose_of(&frame, target)?;
        Ok((DQuat::from_array(pose.rotation) * DVec3::from_array(axis.xyz)).to_array())
    }
    pub fn attached_body(&self, name: &str) -> Result<&str, Diagnostic> {
        self.graphs.attached_body(name)
    }
}

#[derive(Debug)]
pub struct World {
    pub name: String,
    pub gravity: [f64; 3],
    pub models: Vec<Model>,
    pub frames: Vec<Frame>,
    pub plugins: Vec<Plugin>,
    pub source: Source,
}
#[derive(Debug)]
pub struct Model {
    pub name: String,
    pub scoped_name: String,
    pub pose: Pose,
    pub is_static: bool,
    pub canonical_link: String,
    pub links: Vec<Link>,
    pub joints: Vec<Joint>,
    pub models: Vec<Model>,
    pub frames: Vec<Frame>,
    pub plugins: Vec<Plugin>,
    pub source: Source,
}
#[derive(Debug)]
pub struct Frame {
    pub name: String,
    pub attached_to: String,
    pub pose: Pose,
    pub source: Source,
}
#[derive(Debug)]
pub struct Link {
    pub name: String,
    pub scoped_name: String,
    pub pose: Pose,
    pub inertial: Inertial,
    pub collisions: Vec<Collision>,
    pub sensors: Vec<Sensor>,
    pub plugins: Vec<Plugin>,
    pub source: Source,
}
#[derive(Debug)]
pub struct Inertial {
    pub mass: f64,
    /// ixx, iyy, izz, ixy, ixz, iyz in the inertial frame.
    pub inertia: [f64; 6],
    pub pose: Pose,
}
impl Default for Inertial {
    fn default() -> Self {
        Self {
            mass: 1.,
            inertia: [1., 1., 1., 0., 0., 0.],
            pose: Pose::default(),
        }
    }
}
#[derive(Debug)]
pub struct Collision {
    pub name: String,
    pub pose: Pose,
    pub geometry: Geometry,
    pub source: Source,
}
#[derive(Debug, PartialEq)]
pub enum Geometry {
    Box { size: [f64; 3] },
    Sphere { radius: f64 },
    Cylinder { radius: f64, length: f64 },
    Plane { normal: [f64; 3], size: [f64; 2] },
}
#[derive(Debug, PartialEq, Eq)]
pub enum JointKind {
    Fixed,
    Revolute,
    Prismatic,
    Continuous,
}
#[derive(Debug)]
pub struct Joint {
    pub name: String,
    pub kind: JointKind,
    pub parent: String,
    pub child: String,
    pub pose: Pose,
    pub axis: Option<Axis>,
    pub source: Source,
}
#[derive(Debug)]
pub struct Axis {
    pub xyz: [f64; 3],
    pub expressed_in: String,
    pub lower: f64,
    pub upper: f64,
    pub effort: f64,
    pub velocity: f64,
}
#[derive(Debug)]
pub struct Sensor {
    pub name: String,
    pub kind: String,
    pub pose: Pose,
    pub update_rate: f64,
    pub always_on: bool,
    pub topic: String,
    /// Unmodified sensor-specific XML; consumers must validate its own settings.
    pub configuration_xml: String,
    pub source: Source,
}
#[derive(Debug)]
pub struct Plugin {
    pub name: String,
    pub filename: String,
    /// Original XML including namespace-qualified extension attributes/content.
    pub xml: String,
}
