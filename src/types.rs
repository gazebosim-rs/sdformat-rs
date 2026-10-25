// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0

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
    /// Eagerly resolve non-frame poses against the loaded frame graph. Native
    /// collision/visual/sensor DOM loading defers these errors until resolve.
    /// Inertial references are checked as an explicit consumer policy; native
    /// Link::ResolveInertial instead ignores the retained inertial reference.
    /// Scene builders can call this before creating any runtime objects. This
    /// does not rebuild the graph after caller edits to the public DOM fields.
    pub fn validate_non_frame_poses(&self) -> Result<(), Diagnostic> {
        fn validate(sdf: &Sdf, model: &Model) -> Result<(), Diagnostic> {
            for link in &model.links {
                for pose in std::iter::once(&link.inertial.pose)
                    .chain(link.collisions.iter().map(|c| &c.pose))
                    .chain(link.sensors.iter().map(|s| &s.pose))
                    .chain(link.visuals.iter().map(|v| &v.pose))
                {
                    sdf.resolve_pose(pose, &model.scoped_name, &link.scoped_name, "::")?;
                }
            }
            for nested in &model.models {
                validate(sdf, nested)?;
            }
            Ok(())
        }
        for model in self
            .models
            .iter()
            .chain(self.worlds.iter().flat_map(|w| &w.models))
        {
            validate(self, model)?;
        }
        Ok(())
    }
    /// Resolve a semantic frame into another frame. Names are fully scoped,
    /// e.g. `world::robot::link`; the global root is `::`.
    pub fn pose_of(&self, name: &str, target: &str) -> Result<Transform, Diagnostic> {
        self.graphs.pose_of(name, target)
    }
    /// Resolve a collision, visual, sensor or inertial pose. `scope` is its owning model,
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
            self.graphs.axis_frame(scope, &axis.expressed_in)?
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
    pub placement_frame: String,
    /// Original include metadata; absent for a model declared directly in XML.
    pub include: Option<Box<Include>>,
    pub links: Vec<Link>,
    pub joints: Vec<Joint>,
    pub models: Vec<Model>,
    pub frames: Vec<Frame>,
    pub plugins: Vec<Plugin>,
    pub source: Source,
}
impl Model {
    /// Native DOM canonical-link selection, independently of static attachments.
    /// The name is relative to this model. An explicit frame alias can produce
    /// a nonempty name with no link. Only the first nested model is considered.
    pub fn canonical_link_and_relative_name(&self) -> (Option<&Link>, String) {
        if !self.canonical_link.is_empty() {
            return (
                self.link_by_relative_name(&self.canonical_link),
                self.canonical_link.clone(),
            );
        }
        if let Some(link) = self.links.first() {
            return (Some(link), link.name.clone());
        }
        if let Some(model) = self.models.first() {
            let (link, name) = model.canonical_link_and_relative_name();
            return (
                link,
                if name.is_empty() {
                    name
                } else {
                    format!("{}::{name}", model.name)
                },
            );
        }
        (None, String::new())
    }
    fn link_by_relative_name(&self, name: &str) -> Option<&Link> {
        if let Some((model, remaining)) = name.split_once("::") {
            self.models
                .iter()
                .find(|candidate| candidate.name == model)?
                .link_by_relative_name(remaining)
        } else {
            self.links.iter().find(|link| link.name == name)
        }
    }
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
    pub visuals: Vec<Visual>,
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
    /// Symmetric [x,y,z,p,q,r] fluid added mass about the link origin.
    /// Metadata only; None differs from an explicit zero matrix.
    pub fluid_added_mass: Option<Box<[[f64; 6]; 6]>>,
}
impl Default for Inertial {
    fn default() -> Self {
        Self {
            mass: 1.,
            inertia: [1., 1., 1., 0., 0., 0.],
            pose: Pose::default(),
            fluid_added_mass: None,
        }
    }
}
#[derive(Debug)]
pub struct Collision {
    pub name: String,
    pub pose: Pose,
    pub geometry: Geometry,
    pub surface: Surface,
    pub source: Source,
}
/// Contact and friction metadata, independent of any physics engine.
/// Source is None for an absent group and Some for explicitly loaded XML.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Surface {
    pub contact: Contact,
    pub friction: Friction,
    pub source: Option<Source>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Contact {
    pub collide_bitmask: u16,
    /// None means use collide_bitmask as the category.
    pub category_bitmask: Option<u16>,
    pub source: Option<Source>,
}
impl Default for Contact {
    fn default() -> Self {
        Self {
            collide_bitmask: 0xff,
            category_bitmask: None,
            source: None,
        }
    }
}
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Friction {
    pub ode: OdeFriction,
    pub bullet: Option<BulletFriction>,
    pub torsional: Option<TorsionalFriction>,
    pub source: Option<Source>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct OdeFriction {
    pub mu: f64,
    pub mu2: f64,
    pub fdir1: [f64; 3],
    pub slip1: f64,
    pub slip2: f64,
    pub source: Option<Source>,
}
impl Default for OdeFriction {
    fn default() -> Self {
        Self {
            mu: 1.,
            mu2: 1.,
            fdir1: [0.; 3],
            slip1: 0.,
            slip2: 0.,
            source: None,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct BulletFriction {
    pub friction: f64,
    pub friction2: f64,
    pub fdir1: [f64; 3],
    pub rolling_friction: f64,
    pub source: Option<Source>,
}
impl Default for BulletFriction {
    fn default() -> Self {
        Self {
            friction: 1.,
            friction2: 1.,
            fdir1: [0.; 3],
            rolling_friction: 1.,
            source: None,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct TorsionalFriction {
    pub coefficient: f64,
    pub use_patch_radius: bool,
    pub patch_radius: f64,
    pub surface_radius: f64,
    pub ode_slip: f64,
    pub source: Option<Source>,
}
impl Default for TorsionalFriction {
    fn default() -> Self {
        Self {
            coefficient: 1.,
            use_patch_radius: true,
            patch_radius: 0.,
            surface_radius: 0.,
            ode_slip: 0.,
            source: None,
        }
    }
}
#[derive(Debug)]
pub struct Visual {
    pub name: String,
    pub pose: Pose,
    pub geometry: Option<Geometry>,
    pub properties: VisualProperties,
    pub plugins: Vec<Plugin>,
    pub source: Source,
}
/// Owned rendering/lidar metadata; parsing does not implement those effects.
#[derive(Debug, Clone, PartialEq)]
pub struct VisualProperties {
    pub cast_shadows: bool,
    /// Native getter stores float32 after converting its XML double value.
    pub transparency: f32,
    pub visibility_flags: u32,
    /// Presence matches native HasLaserRetro, including Some(0.0).
    pub laser_retro: Option<f64>,
}
impl Default for VisualProperties {
    fn default() -> Self {
        Self {
            cast_shadows: true,
            transparency: 0.,
            visibility_flags: u32::MAX,
            laser_retro: None,
        }
    }
}
#[derive(Debug, PartialEq)]
pub enum Geometry {
    Box { size: [f64; 3] },
    Sphere { radius: f64 },
    Cylinder { radius: f64, length: f64 },
    Capsule { radius: f64, length: f64 },
    Cone { radius: f64, length: f64 },
    Ellipsoid { radii: [f64; 3] },
    Mesh(Mesh),
    Heightmap(Box<Heightmap>),
    Polyline(Vec<Polyline>),
    Plane { normal: [f64; 3], size: [f64; 2] },
}
/// Heightmap metadata; URIs are retained without resource or image IO.
#[derive(Debug, Clone, PartialEq)]
pub struct Heightmap {
    pub uri: String,
    pub size: [f64; 3],
    pub position: [f64; 3],
    pub use_terrain_paging: bool,
    pub sampling: u32,
    pub textures: Vec<HeightmapTexture>,
    pub blends: Vec<HeightmapBlend>,
    pub source: Source,
}
#[derive(Debug, Clone, PartialEq)]
pub struct HeightmapTexture {
    pub size: f64,
    pub diffuse: String,
    pub normal: String,
}
#[derive(Debug, Clone, PartialEq)]
pub struct HeightmapBlend {
    pub min_height: f64,
    pub fade_distance: f64,
}
/// Ordered polyline metadata; loading does not validate a physical polygon.
#[derive(Debug, Clone, PartialEq)]
pub struct Polyline {
    pub height: f64,
    pub points: Vec<[f64; 2]>,
    pub source: Source,
}
/// Mesh metadata; the URI is retained without performing filesystem or network IO.
#[derive(Debug, PartialEq)]
pub struct Mesh {
    pub uri: String,
    pub scale: [f64; 3],
    pub submesh: String,
    pub center_submesh: bool,
    pub optimization: MeshOptimization,
    pub convex_decomposition: Option<ConvexDecomposition>,
    /// Original resource and mesh element position, including imported resources.
    pub source: Source,
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum MeshOptimization {
    #[default]
    None,
    ConvexHull,
    ConvexDecomposition,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConvexDecomposition {
    pub max_convex_hulls: u32,
    pub voxel_resolution: u32,
}
#[derive(Debug, PartialEq, Eq)]
pub enum JointKind {
    Fixed,
    Revolute,
    Prismatic,
    Continuous,
    Universal,
    Revolute2,
}
#[derive(Debug)]
pub struct Joint {
    pub name: String,
    pub kind: JointKind,
    pub parent: String,
    pub child: String,
    pub pose: Pose,
    pub axis: Option<Axis>,
    pub axis2: Option<Axis>,
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
    pub enable_metrics: bool,
    pub imu: Imu,
    pub topic: String,
    /// Canonical SDF frame metadata; an empty value delegates fallback to the consumer.
    pub frame_id: String,
    /// Legacy literal gz_frame_id element, including an explicitly empty value.
    /// Consumers apply this only when frame_id is empty, before their name fallback.
    pub gz_frame_id: Option<String>,
    /// Original IMU-specific XML, retained alongside the typed configuration.
    pub configuration_xml: String,
    pub source: Source,
}
/// Original include values, independently of the resolved model/resource state.
#[derive(Debug)]
pub struct Include {
    /// Native string value; resolver requests preserve CDATA whitespace separately.
    pub uri: String,
    pub name: Option<String>,
    pub pose: Option<Pose>,
    pub is_static: Option<bool>,
    pub placement_frame: Option<String>,
    /// Only the plugins supplied by this include; model.plugins also contains them.
    pub plugins: Vec<Plugin>,
    pub source: Source,
    /// Exact source span, not a converted/expanded serialization.
    pub xml: String,
}
#[derive(Debug, Clone)]
pub struct Plugin {
    pub name: String,
    pub filename: String,
    /// Original XML including namespace-qualified extension attributes/content.
    pub xml: String,
}

/// SDF noise parameters. Sampling and random-generator ownership belong to consumers.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Noise {
    pub kind: NoiseKind,
    pub mean: f64,
    pub stddev: f64,
    pub bias_mean: f64,
    pub bias_stddev: f64,
    pub dynamic_bias_stddev: f64,
    pub dynamic_bias_correlation_time: f64,
    pub precision: f64,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NoiseKind {
    #[default]
    None,
    Gaussian,
    GaussianQuantized,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Imu {
    pub angular_velocity: [Noise; 3],
    pub linear_acceleration: [Noise; 3],
    pub enable_orientation: bool,
    pub localization: String,
    pub custom_rpy: [f64; 3],
    pub custom_rpy_parent_frame: String,
    pub gravity_dir_x: [f64; 3],
    pub gravity_dir_x_parent_frame: String,
}
impl Default for Imu {
    fn default() -> Self {
        Self {
            angular_velocity: [Noise::default(); 3],
            linear_acceleration: [Noise::default(); 3],
            enable_orientation: true,
            localization: "CUSTOM".into(),
            custom_rpy: [0.; 3],
            custom_rpy_parent_frame: String::new(),
            gravity_dir_x: [1., 0., 0.],
            gravity_dir_x_parent_frame: String::new(),
        }
    }
}
