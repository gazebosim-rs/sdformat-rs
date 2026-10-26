// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::Camera;
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
    JointAxisExpressedInInvalid,
    JointAxisMimicInvalid,
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
            for joint in &model.joints {
                let frame = format!("{}::{}", model.scoped_name, joint.name);
                for sensor in &joint.sensors {
                    sdf.resolve_pose(&sensor.pose, &model.scoped_name, &frame, "::")?;
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
    /// `default_frame` is its owning semantic frame, and `target` is fully scoped.
    pub fn resolve_pose(
        &self,
        pose: &Pose,
        scope: &str,
        default_frame: &str,
        target: &str,
    ) -> Result<Transform, Diagnostic> {
        self.graphs.resolve_pose(pose, scope, default_frame, target)
    }
    /// Resolve an SDF reference, including a nested model's terminal `::__model__`.
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
    /// Resolve an attachment to a link, world, or static model's implicit frame.
    /// Static sinks retain `<model scope>::__model__`; they are not collapsed to
    /// world. This graph identity does not select simulator static-body behavior.
    pub fn attached_body(&self, name: &str) -> Result<&str, Diagnostic> {
        self.graphs.attached_body(name)
    }
}

#[derive(Debug)]
pub struct World {
    pub name: String,
    pub gravity: [f64; 3],
    /// Atmosphere metadata, including native defaults when the element is absent.
    pub atmosphere: Atmosphere,
    /// Physics profiles in declaration order, or the implicit native default.
    pub physics: Vec<Physics>,
    pub models: Vec<Model>,
    pub frames: Vec<Frame>,
    pub plugins: Vec<Plugin>,
    pub source: Source,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtmosphereType {
    Adiabatic,
}

/// Sea-level atmosphere metadata; no pressure or temperature sampling is implied.
#[derive(Debug, Clone)]
pub struct Atmosphere {
    pub atmosphere_type: AtmosphereType,
    pub temperature_kelvin: f64,
    /// Temperature gradient in kelvins per metre of increasing altitude.
    pub temperature_gradient: f64,
    /// Sea-level pressure in pascals.
    pub pressure: f64,
    pub source: Option<Source>,
    pub xml: Option<String>,
}
impl Default for Atmosphere {
    fn default() -> Self {
        Self {
            atmosphere_type: AtmosphereType::Adiabatic,
            temperature_kelvin: 288.15,
            temperature_gradient: -0.0065,
            pressure: 101325.,
            source: None,
            xml: None,
        }
    }
}
impl World {
    /// First explicitly default profile, otherwise the first profile.
    pub fn physics_default(&self) -> Option<&Physics> {
        self.physics
            .iter()
            .find(|profile| profile.is_default)
            .or_else(|| self.physics.first())
    }
}

/// Owned physics configuration; parsing does not select or configure an engine.
#[derive(Debug, Clone, PartialEq)]
pub struct Physics {
    pub name: String,
    pub is_default: bool,
    pub engine_type: String,
    pub max_step_size: f64,
    pub real_time_factor: f64,
    pub real_time_update_rate: f64,
    pub max_contacts: i32,
    /// Absent for a constructed value or the world's implicit default profile.
    pub source: Option<Source>,
    pub xml: Option<String>,
}
impl Default for Physics {
    fn default() -> Self {
        Self {
            name: String::new(),
            is_default: false,
            engine_type: "ode".into(),
            max_step_size: 0.001,
            real_time_factor: 1.,
            real_time_update_rate: 1000.,
            max_contacts: 20,
            source: None,
            xml: None,
        }
    }
}
#[derive(Debug, Clone)]
pub struct Model {
    pub name: String,
    pub scoped_name: String,
    /// Owned namespace metadata. XML namespace loading is not supported yet.
    pub raw_namespace: Option<String>,
    pub pose: Pose,
    pub is_static: bool,
    pub canonical_link: String,
    pub placement_frame: String,
    /// Original include metadata; absent for a model declared directly in XML.
    pub include: Option<Box<Include>>,
    /// Includes flattened into this model; their generated frames remain in frames.
    pub merged_includes: Vec<MergedInclude>,
    pub links: Vec<Link>,
    pub joints: Vec<Joint>,
    pub models: Vec<Model>,
    pub frames: Vec<Frame>,
    pub plugins: Vec<Plugin>,
    pub source: Source,
}
impl Model {
    /// Resolve literal `{name}` placeholders against the current model name.
    /// This does not change frame scopes or include/resource lookup.
    pub fn namespace(&self) -> Option<String> {
        self.raw_namespace
            .as_ref()
            .map(|namespace| namespace.replace("{name}", &self.name))
    }
    /// Insert a link with a unique direct name; an existing frame graph is unchanged.
    pub fn add_link_unique(&mut self, link: Link) -> bool {
        if self.links.iter().any(|existing| existing.name == link.name) {
            return false;
        }
        self.links.push(link);
        true
    }
    /// Insert a joint with a unique direct name.
    /// This edits owned DOM data; an already built frame graph is unchanged.
    pub fn add_joint_unique(&mut self, joint: Joint) -> bool {
        if self
            .joints
            .iter()
            .any(|existing| existing.name == joint.name)
        {
            return false;
        }
        self.joints.push(joint);
        true
    }
    pub fn clear_joints(&mut self) {
        self.joints.clear();
    }
    /// Insert a nested model with a unique direct name.
    /// This edits owned DOM data; an already built frame graph is unchanged.
    pub fn add_model_unique(&mut self, model: Model) -> bool {
        if self
            .models
            .iter()
            .any(|existing| existing.name == model.name)
        {
            return false;
        }
        self.models.push(model);
        true
    }
    pub fn clear_models(&mut self) {
        self.models.clear();
    }
    /// Insert a frame with a unique direct name; an existing frame graph is unchanged.
    pub fn add_frame_unique(&mut self, frame: Frame) -> bool {
        if self
            .frames
            .iter()
            .any(|existing| existing.name == frame.name)
        {
            return false;
        }
        self.frames.push(frame);
        true
    }
    pub fn clear_frames(&mut self) {
        self.frames.clear();
    }
    /// Borrow a nested model by an explicitly scoped relative name.
    /// A descendant's unqualified name does not search through other models.
    pub fn model_by_name(&self, name: &str) -> Option<&Model> {
        let (first, remaining) = name.split_once("::").unwrap_or((name, ""));
        let model = self.models.iter().find(|model| model.name == first)?;
        if name.contains("::") {
            model.model_by_name(remaining)
        } else {
            Some(model)
        }
    }
    /// Native DOM canonical-link selection, independently of static attachments.
    /// The name is relative to this model. An explicit frame alias can produce
    /// a nonempty name with no link. Only the first nested model is considered.
    pub fn canonical_link_and_relative_name(&self) -> (Option<&Link>, String) {
        if !self.canonical_link.is_empty() {
            return (
                self.link_by_name(&self.canonical_link),
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
    /// Borrow a link using a name relative to this model's DOM scope.
    /// Explicit nested scopes are required; no semantic frame aliases are followed.
    pub fn link_by_name(&self, name: &str) -> Option<&Link> {
        let (model, name) = self.member_scope(name);
        model.links.iter().find(|link| link.name == name)
    }
    /// Borrow a joint using explicit relative nested model scopes.
    pub fn joint_by_name(&self, name: &str) -> Option<&Joint> {
        let (model, name) = self.member_scope(name);
        model.joints.iter().find(|joint| joint.name == name)
    }
    /// Borrow an explicitly declared frame; implicit model/link frames are absent.
    pub fn frame_by_name(&self, name: &str) -> Option<&Frame> {
        let (model, name) = self.member_scope(name);
        model.frames.iter().find(|frame| frame.name == name)
    }
    fn member_scope<'n>(&self, name: &'n str) -> (&Self, &'n str) {
        if let Some((scope, local)) = name.rsplit_once("::")
            && let Some(model) = self.model_by_name(scope)
        {
            return (model, local);
        }
        // Native DOM lookup falls back to an exact direct member name when the
        // model prefix is absent, including literal-colon names set in memory.
        (self, name)
    }
}
#[derive(Debug, Clone)]
pub struct Frame {
    pub name: String,
    pub attached_to: String,
    pub pose: Pose,
    pub source: Source,
}
impl Default for Frame {
    fn default() -> Self {
        Self {
            name: String::new(),
            attached_to: String::new(),
            pose: Pose::default(),
            source: Source {
                uri: String::new(),
                line: 0,
                column: 0,
            },
        }
    }
}
impl Frame {
    /// A frame pose without an explicit relative_to uses its attachment.
    pub fn semantic_relative_to(&self) -> &str {
        if self.pose.relative_to.is_empty() {
            &self.attached_to
        } else {
            &self.pose.relative_to
        }
    }
}
/// Owned light values; XML parsing and rendering remain pending.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LightType {
    #[default]
    Point,
    Directional,
    Spot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Light {
    pub name: String,
    pub light_type: LightType,
    pub pose: Pose,
    pub light_on: bool,
    pub visualize: bool,
    pub cast_shadows: bool,
    /// Native color channels are single-precision values.
    pub diffuse: [f32; 4],
    pub specular: [f32; 4],
    pub attenuation_range: f64,
    pub spot_falloff: f64,
    /// Radians.
    pub spot_inner_angle: f64,
    /// Radians.
    pub spot_outer_angle: f64,
    pub linear_attenuation_factor: f64,
    pub constant_attenuation_factor: f64,
    pub quadratic_attenuation_factor: f64,
    pub direction: [f64; 3],
    pub intensity: f64,
}
impl Default for Light {
    fn default() -> Self {
        Self {
            name: String::new(),
            light_type: LightType::Point,
            pose: Pose::default(),
            light_on: true,
            visualize: true,
            cast_shadows: false,
            diffuse: [0., 0., 0., 1.],
            specular: [0., 0., 0., 1.],
            attenuation_range: 10.,
            spot_falloff: 0.,
            spot_inner_angle: 0.,
            spot_outer_angle: 0.,
            linear_attenuation_factor: 1.,
            constant_attenuation_factor: 1.,
            quadratic_attenuation_factor: 0.,
            direction: [0., 0., -1.],
            intensity: 1.,
        }
    }
}
impl Light {
    pub fn set_spot_falloff(&mut self, value: f64) {
        self.spot_falloff = value.max(0.);
    }

    pub fn set_spot_inner_angle(&mut self, radians: f64) {
        self.spot_inner_angle = radians.max(0.);
    }

    pub fn set_spot_outer_angle(&mut self, radians: f64) {
        self.spot_outer_angle = radians.max(0.);
    }

    pub fn set_linear_attenuation_factor(&mut self, value: f64) {
        self.linear_attenuation_factor = value.clamp(0., 1.);
    }

    pub fn set_constant_attenuation_factor(&mut self, value: f64) {
        self.constant_attenuation_factor = value.clamp(0., 1.);
    }

    pub fn set_quadratic_attenuation_factor(&mut self, value: f64) {
        self.quadratic_attenuation_factor = value.max(0.);
    }
}
/// Owned particle emitter identity; configuration and XML parsing remain pending.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParticleEmitter {
    pub name: String,
}
/// Owned projector identity; configuration and XML parsing remain pending.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Projector {
    pub name: String,
}
#[derive(Debug, Clone)]
pub struct Link {
    pub name: String,
    pub scoped_name: String,
    pub pose: Pose,
    pub inertial: Inertial,
    pub collisions: Vec<Collision>,
    pub visuals: Vec<Visual>,
    pub lights: Vec<Light>,
    pub particle_emitters: Vec<ParticleEmitter>,
    pub projectors: Vec<Projector>,
    pub sensors: Vec<Sensor>,
    pub properties: LinkProperties,
    pub plugins: Vec<Plugin>,
    pub source: Source,
}
impl Link {
    /// Insert a particle emitter with a unique direct name.
    pub fn add_particle_emitter_unique(&mut self, emitter: ParticleEmitter) -> bool {
        if self
            .particle_emitters
            .iter()
            .any(|existing| existing.name == emitter.name)
        {
            return false;
        }
        self.particle_emitters.push(emitter);
        true
    }

    /// Insert a projector with a unique direct name.
    pub fn add_projector_unique(&mut self, projector: Projector) -> bool {
        if self
            .projectors
            .iter()
            .any(|existing| existing.name == projector.name)
        {
            return false;
        }
        self.projectors.push(projector);
        true
    }

    /// Insert a light with a unique direct name.
    pub fn add_light_unique(&mut self, light: Light) -> bool {
        if self
            .lights
            .iter()
            .any(|existing| existing.name == light.name)
        {
            return false;
        }
        self.lights.push(light);
        true
    }

    pub fn clear_lights(&mut self) {
        self.lights.clear();
    }

    /// Insert a sensor with a unique direct name.
    pub fn add_sensor_unique(&mut self, sensor: Sensor) -> bool {
        if self
            .sensors
            .iter()
            .any(|existing| existing.name == sensor.name)
        {
            return false;
        }
        self.sensors.push(sensor);
        true
    }

    pub fn clear_sensors(&mut self) {
        self.sensors.clear();
    }
}
/// Link flags retained as data; consumers implement or reject their physical use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkProperties {
    pub enable_wind: bool,
    pub enable_gravity: bool,
    pub kinematic: bool,
    /// Effective XML parameter; the native Link DOM has no corresponding getter.
    pub self_collide: bool,
}
impl Default for LinkProperties {
    fn default() -> Self {
        Self {
            enable_wind: false,
            enable_gravity: true,
            kinematic: false,
            self_collide: false,
        }
    }
}
impl Link {
    /// Insert a collision with a unique direct name.
    pub fn add_collision_unique(&mut self, collision: Collision) -> bool {
        if self
            .collisions
            .iter()
            .any(|existing| existing.name == collision.name)
        {
            return false;
        }
        self.collisions.push(collision);
        true
    }

    pub fn clear_collisions(&mut self) {
        self.collisions.clear();
    }

    /// Insert a visual with a unique direct name.
    pub fn add_visual_unique(&mut self, visual: Visual) -> bool {
        if self
            .visuals
            .iter()
            .any(|existing| existing.name == visual.name)
        {
            return false;
        }
        self.visuals.push(visual);
        true
    }

    pub fn clear_visuals(&mut self) {
        self.visuals.clear();
    }
}
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct Visual {
    pub name: String,
    pub pose: Pose,
    pub geometry: Option<Geometry>,
    pub properties: VisualProperties,
    pub material: Option<Box<Material>>,
    pub plugins: Vec<Plugin>,
    pub source: Source,
}
/// Owned rendering/lidar metadata; parsing does not implement those effects.
/// Owned rendering metadata; no image/script loading or renderer effects.
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub ambient: [f32; 4],
    pub diffuse: [f32; 4],
    pub specular: [f32; 4],
    pub emissive: [f32; 4],
    pub shininess: f64,
    /// Native raw float metadata may be nonfinite; consumers choose validation.
    pub render_order: f32,
    pub lighting: bool,
    pub double_sided: bool,
    pub shader: ShaderType,
    pub normal_map: String,
    pub script: Option<MaterialScript>,
    pub source: Source,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShaderType {
    #[default]
    Pixel,
    Vertex,
    NormalMapObjectSpace,
    NormalMapTangentSpace,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialScript {
    /// Declaration order; the native DOM ScriptUri getter uses the first URI.
    pub uris: Vec<String>,
    pub name: String,
    pub source: Source,
}

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
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JointKind {
    Fixed,
    Ball,
    Gearbox,
    Screw,
    Revolute,
    Prismatic,
    Continuous,
    Universal,
    Revolute2,
}
#[derive(Debug, Clone)]
pub struct Joint {
    pub name: String,
    pub kind: JointKind,
    pub parent: String,
    pub child: String,
    pub pose: Pose,
    pub axis: Option<Axis>,
    pub axis2: Option<Axis>,
    /// Meters per revolution. Legacy `<thread_pitch>` is converted at load.
    pub screw_thread_pitch: f64,
    /// Known joint physics payload retained verbatim; engine interpretation is separate.
    pub physics_xml: Option<String>,
    /// Owned sensor envelopes; their default pose frame is this joint.
    pub sensors: Vec<Sensor>,
    pub source: Source,
}
impl Joint {
    /// Zero borrows the first axis; every positive index aliases the second.
    /// This follows native Joint::Axis clamping without narrowing Rust's usize.
    pub fn axis(&self, index: usize) -> Option<&Axis> {
        if index == 0 {
            self.axis.as_ref()
        } else {
            self.axis2.as_ref()
        }
    }
}
#[derive(Debug, Clone)]
pub struct Axis {
    pub xyz: [f64; 3],
    pub expressed_in: String,
    pub lower: f64,
    pub upper: f64,
    pub effort: f64,
    pub velocity: f64,
    pub dynamics: AxisDynamics,
    /// Joint-stop stiffness, independently of the spring stiffness in dynamics.
    pub stiffness: f64,
    pub dissipation: f64,
    /// Optional leader joint/axis and linear position relationship.
    pub mimic: Option<AxisMimic>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AxisMimic {
    pub joint: String,
    pub axis: String,
    pub multiplier: f64,
    pub offset: f64,
    pub reference: f64,
}
impl Axis {
    /// Normalize a finite direction, preserving the prior value on failure.
    pub fn try_set_xyz(&mut self, xyz: [f64; 3]) -> Result<(), AxisDirectionError> {
        let vector = DVec3::from_array(xyz);
        let length = vector.length();
        if !length.is_finite() {
            return Err(AxisDirectionError::NonFinite);
        }
        // The native SetXyz zero check uses sdf::equal's default float epsilon.
        if length <= f64::from(1e-6_f32) {
            return Err(AxisDirectionError::ZeroLength);
        }
        self.xyz = (vector / length).to_array();
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisDirectionError {
    ZeroLength,
    NonFinite,
}

impl std::fmt::Display for AxisDirectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroLength => formatter.write_str("The norm of the xyz vector cannot be zero"),
            Self::NonFinite => formatter.write_str("The xyz vector must be finite"),
        }
    }
}

impl std::error::Error for AxisDirectionError {}
/// Native axis dynamics metadata; no damping, friction or spring is simulated here.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct AxisDynamics {
    pub damping: f64,
    pub friction: f64,
    pub spring_reference: f64,
    pub spring_stiffness: f64,
    /// Explicit dynamics declaration, if present; absent declarations use zero defaults.
    pub source: Option<Source>,
    pub xml: Option<String>,
}
#[derive(Debug, Clone)]
pub struct Sensor {
    pub name: String,
    pub kind: String,
    pub pose: Pose,
    pub update_rate: f64,
    pub always_on: bool,
    pub enable_metrics: bool,
    pub config: SensorConfig,
    /// Plugins in declaration or insertion order.
    pub plugins: Vec<Plugin>,
    pub topic: String,
    /// Canonical SDF frame metadata; an empty value delegates fallback to the consumer.
    pub frame_id: String,
    /// Legacy literal gz_frame_id element, including an explicitly empty value.
    /// Consumers apply this only when frame_id is empty, before their name fallback.
    pub gz_frame_id: Option<String>,
    /// Exact selected configuration child XML; empty when that child was omitted.
    pub configuration_xml: String,
    pub source: Source,
}
impl Default for Sensor {
    fn default() -> Self {
        Self {
            name: String::new(),
            kind: "none".into(),
            pose: Pose::default(),
            update_rate: 0.0,
            always_on: false,
            enable_metrics: false,
            config: SensorConfig::Unconfigured,
            plugins: Vec::new(),
            topic: String::new(),
            frame_id: String::new(),
            gz_frame_id: None,
            configuration_xml: String::new(),
            source: Source {
                uri: String::new(),
                line: 0,
                column: 0,
            },
        }
    }
}
/// Native sensor classification, separate from the literal XML `type` value.
/// A parsed `gps` sensor, for example, has type [`SensorType::NavSat`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SensorType {
    #[default]
    None,
    Altimeter,
    Camera,
    Contact,
    DepthCamera,
    ForceTorque,
    Gps,
    GpuLidar,
    Imu,
    LogicalCamera,
    Magnetometer,
    Multicamera,
    Lidar,
    Rfid,
    RfidTag,
    Sonar,
    WirelessReceiver,
    WirelessTransmitter,
    AirPressure,
    RgbdCamera,
    ThermalCamera,
    NavSat,
    SegmentationCamera,
    BoundingBoxCamera,
    Custom,
    WideAngleCamera,
    AirSpeed,
}

impl SensorType {
    /// The string returned by native `Sensor::TypeStr` for this classification.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Altimeter => "altimeter",
            Self::Camera => "camera",
            Self::Contact => "contact",
            Self::DepthCamera => "depth_camera",
            Self::ForceTorque => "force_torque",
            Self::Gps => "gps",
            Self::GpuLidar => "gpu_lidar",
            Self::Imu => "imu",
            Self::LogicalCamera => "logical_camera",
            Self::Magnetometer => "magnetometer",
            Self::Multicamera => "multicamera",
            Self::Lidar => "lidar",
            Self::Rfid => "rfid",
            Self::RfidTag => "rfidtag",
            Self::Sonar => "sonar",
            Self::WirelessReceiver => "wireless_receiver",
            Self::WirelessTransmitter => "wireless_transmitter",
            Self::AirPressure => "air_pressure",
            Self::RgbdCamera => "rgbd_camera",
            Self::ThermalCamera => "thermal_camera",
            Self::NavSat => "navsat",
            Self::SegmentationCamera => "segmentation_camera",
            Self::BoundingBoxCamera => "boundingbox_camera",
            Self::Custom => "custom",
            Self::WideAngleCamera => "wide_angle_camera",
            Self::AirSpeed => "air_speed",
        }
    }

    /// Parse a canonical native `TypeStr` name. This does not enable XML loading.
    pub fn from_type_str(value: &str) -> Option<Self> {
        Some(match value {
            "none" => Self::None,
            "altimeter" => Self::Altimeter,
            "camera" => Self::Camera,
            "contact" => Self::Contact,
            "depth_camera" => Self::DepthCamera,
            "force_torque" => Self::ForceTorque,
            "gps" => Self::Gps,
            "gpu_lidar" => Self::GpuLidar,
            "imu" => Self::Imu,
            "logical_camera" => Self::LogicalCamera,
            "magnetometer" => Self::Magnetometer,
            "multicamera" => Self::Multicamera,
            "lidar" => Self::Lidar,
            "rfid" => Self::Rfid,
            "rfidtag" => Self::RfidTag,
            "sonar" => Self::Sonar,
            "wireless_receiver" => Self::WirelessReceiver,
            "wireless_transmitter" => Self::WirelessTransmitter,
            "air_pressure" => Self::AirPressure,
            "rgbd_camera" => Self::RgbdCamera,
            "thermal_camera" => Self::ThermalCamera,
            "navsat" => Self::NavSat,
            "segmentation_camera" => Self::SegmentationCamera,
            "boundingbox_camera" => Self::BoundingBoxCamera,
            "custom" => Self::Custom,
            "wide_angle_camera" => Self::WideAngleCamera,
            "air_speed" => Self::AirSpeed,
            _ => return None,
        })
    }

    /// Replace this native type value only for a canonical type string.
    /// An unknown string leaves the previous value intact.
    pub fn set_from_type_str(&mut self, value: &str) -> bool {
        if let Some(next) = Self::from_type_str(value) {
            *self = next;
            true
        } else {
            false
        }
    }

    /// Classify an XML type using native `Sensor::Load` alias rules. Parsing
    /// support remains limited to the configurations accepted by the loader.
    pub fn from_xml_kind(value: &str) -> Option<Self> {
        if value == "none" {
            return None;
        }
        Some(match value {
            "gps" | "navsat" => Self::NavSat,
            "gpu_ray" | "gpu_lidar" => Self::GpuLidar,
            "ray" | "lidar" => Self::Lidar,
            "depth" | "depth_camera" => Self::DepthCamera,
            "rgbd" | "rgbd_camera" => Self::RgbdCamera,
            "thermal" | "thermal_camera" => Self::ThermalCamera,
            "segmentation" | "segmentation_camera" => Self::SegmentationCamera,
            "boundingbox" | "boundingbox_camera" => Self::BoundingBoxCamera,
            "wideanglecamera" | "wide_angle_camera" => Self::WideAngleCamera,
            _ => Self::from_type_str(value)?,
        })
    }
}

/// Owned configuration selected by the parsed sensor type; unsupported types error.
#[derive(Debug, Clone, PartialEq)]
pub enum SensorConfig {
    /// A native-known sensor family without an owned configuration payload.
    /// Its literal kind and effective type remain on the Sensor envelope.
    Unconfigured,
    Custom(CustomSensor),
    Imu(Box<Imu>),
    Contact(ContactSensor),
    ForceTorque(Box<ForceTorque>),
    Magnetometer(Box<Magnetometer>),
    Altimeter(Box<Altimeter>),
    AirPressure(Box<AirPressure>),
    NavSat(Box<NavSat>),
    AirSpeed(Box<AirSpeed>),
    Lidar(Box<Lidar>),
    Camera(Box<Camera>),
}
/// Direct text values from one vendor extension child of a custom sensor.
/// The original subtree remains in `Sensor::configuration_xml`.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CustomSensor {
    pub element_name: String,
    pub parameters: Vec<(String, String)>,
}
impl CustomSensor {
    pub fn parameter_text(&self, name: &str) -> Option<&str> {
        self.parameters
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}
/// Contact-system element metadata; collision references remain unresolved.
#[derive(Debug, Clone, PartialEq)]
pub struct ContactSensor {
    pub collisions: Vec<String>,
    /// Native contact/topic value. The consumer applies __default_topic__ fallback.
    pub topic: String,
    /// None for an omitted contact group; Some for an explicit declaration.
    pub source: Option<Source>,
}
impl Default for ContactSensor {
    fn default() -> Self {
        Self {
            collisions: vec!["__default__".into()],
            topic: "__default_topic__".into(),
            source: None,
        }
    }
}
/// Force/torque measurement orientation; its point remains the joint origin.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ForceTorqueFrame {
    Parent,
    #[default]
    Child,
    Sensor,
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ForceTorqueMeasureDirection {
    ParentToChild,
    #[default]
    ChildToParent,
}
/// Force/torque configuration only; no wrench is measured or noise sampled here.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ForceTorque {
    pub frame: ForceTorqueFrame,
    pub measure_direction: ForceTorqueMeasureDirection,
    /// Three per-axis force noise records, in newtons.
    pub force: [Noise; 3],
    /// Three per-axis torque noise records, in newton metres.
    pub torque: [Noise; 3],
    /// Source of the explicit declaration; omitted groups use effective defaults.
    pub source: Option<Source>,
}
/// Altimeter configuration only; height and velocity calculation belong to consumers.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Altimeter {
    pub vertical_position: Noise,
    pub vertical_velocity: Noise,
    /// Source of the explicit declaration; omitted groups use effective defaults.
    pub source: Option<Source>,
}
/// Air-pressure configuration only; pressure measurement belongs to consumers.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct AirPressure {
    pub pressure: Noise,
    pub reference_altitude: f64,
    /// Source of the explicit declaration; omitted groups use effective defaults.
    pub source: Option<Source>,
}
/// Air-speed configuration only; differential pressure sampling belongs to consumers.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct AirSpeed {
    pub pressure: Noise,
    /// Source of the explicit declaration; omitted groups use effective defaults.
    pub source: Option<Source>,
}
/// NavSat configuration only; position and velocity estimates belong to consumers.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct NavSat {
    pub horizontal_position: Noise,
    pub vertical_position: Noise,
    pub horizontal_velocity: Noise,
    pub vertical_velocity: Noise,
    /// Source of the explicit declaration; omitted groups use effective defaults.
    pub source: Option<Source>,
}
/// Angular scan metadata in radians; a ray generator belongs to consumers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LidarScan {
    pub samples: u32,
    pub resolution: f64,
    pub min_angle: f64,
    pub max_angle: f64,
}
/// Lidar configuration only; no rays are simulated here.
#[derive(Debug, Clone, PartialEq)]
pub struct Lidar {
    pub horizontal: LidarScan,
    pub vertical: LidarScan,
    pub range_min: f64,
    pub range_max: f64,
    pub range_resolution: f64,
    pub noise: Noise,
    pub visibility_mask: u32,
    /// Source of the explicit declaration; omitted groups use effective defaults.
    pub source: Option<Source>,
}
impl Default for Lidar {
    fn default() -> Self {
        Self {
            horizontal: LidarScan {
                samples: 640,
                resolution: 1.,
                min_angle: 0.,
                max_angle: 0.,
            },
            vertical: LidarScan {
                samples: 1,
                resolution: 1.,
                min_angle: 0.,
                max_angle: 0.,
            },
            range_min: 0.,
            range_max: 0.,
            range_resolution: 0.,
            noise: Noise::default(),
            visibility_mask: u32::MAX,
            source: None,
        }
    }
}
/// Magnetometer configuration only; field sampling belongs to consumers.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Magnetometer {
    /// X, Y and Z noise records in declaration order.
    pub noise: [Noise; 3],
    /// Source of the explicit declaration; omitted groups use effective defaults.
    pub source: Option<Source>,
}
impl Sensor {
    /// Load a standalone `<sensor>` element into this value without a frame graph.
    /// A native-invalid `type="none"` updates envelope values before returning
    /// an error, including `enable_metrics`; full document validation is separate.
    pub fn load_element_str(
        &mut self,
        xml: &str,
        source: &str,
        resolver: &dyn crate::ResourceResolver,
    ) -> Result<(), Diagnostic> {
        crate::parse::load_sensor_element_into(self, xml, source, resolver)
    }

    /// Native effective type of the literal XML kind. Parsed sensors always
    /// return `Some`; manually changed or unknown kinds return `None`.
    pub fn effective_type(&self) -> Option<SensorType> {
        SensorType::from_xml_kind(&self.kind)
    }

    pub fn imu(&self) -> Option<&Imu> {
        match &self.config {
            SensorConfig::Imu(value) => Some(value),
            SensorConfig::Contact(_)
            | SensorConfig::ForceTorque(_)
            | SensorConfig::Magnetometer(_)
            | SensorConfig::Altimeter(_)
            | SensorConfig::AirPressure(_)
            | SensorConfig::NavSat(_)
            | SensorConfig::AirSpeed(_)
            | SensorConfig::Lidar(_)
            | SensorConfig::Camera(_)
            | SensorConfig::Custom(_)
            | SensorConfig::Unconfigured => None,
        }
    }
    pub fn contact(&self) -> Option<&ContactSensor> {
        match &self.config {
            SensorConfig::Contact(value) => Some(value),
            SensorConfig::Imu(_)
            | SensorConfig::ForceTorque(_)
            | SensorConfig::Magnetometer(_)
            | SensorConfig::Altimeter(_)
            | SensorConfig::AirPressure(_)
            | SensorConfig::NavSat(_)
            | SensorConfig::AirSpeed(_)
            | SensorConfig::Lidar(_)
            | SensorConfig::Camera(_)
            | SensorConfig::Custom(_)
            | SensorConfig::Unconfigured => None,
        }
    }
    pub fn force_torque(&self) -> Option<&ForceTorque> {
        match &self.config {
            SensorConfig::ForceTorque(value) => Some(value),
            SensorConfig::Imu(_)
            | SensorConfig::Contact(_)
            | SensorConfig::Magnetometer(_)
            | SensorConfig::Altimeter(_)
            | SensorConfig::AirPressure(_)
            | SensorConfig::NavSat(_)
            | SensorConfig::AirSpeed(_)
            | SensorConfig::Lidar(_)
            | SensorConfig::Camera(_)
            | SensorConfig::Custom(_)
            | SensorConfig::Unconfigured => None,
        }
    }
    pub fn magnetometer(&self) -> Option<&Magnetometer> {
        match &self.config {
            SensorConfig::Magnetometer(value) => Some(value),
            SensorConfig::Imu(_)
            | SensorConfig::Contact(_)
            | SensorConfig::ForceTorque(_)
            | SensorConfig::Altimeter(_)
            | SensorConfig::AirPressure(_)
            | SensorConfig::NavSat(_)
            | SensorConfig::AirSpeed(_)
            | SensorConfig::Lidar(_)
            | SensorConfig::Camera(_)
            | SensorConfig::Custom(_)
            | SensorConfig::Unconfigured => None,
        }
    }
    pub fn altimeter(&self) -> Option<&Altimeter> {
        match &self.config {
            SensorConfig::Altimeter(value) => Some(value),
            SensorConfig::Imu(_)
            | SensorConfig::Contact(_)
            | SensorConfig::ForceTorque(_)
            | SensorConfig::Magnetometer(_)
            | SensorConfig::AirPressure(_)
            | SensorConfig::NavSat(_)
            | SensorConfig::AirSpeed(_)
            | SensorConfig::Lidar(_)
            | SensorConfig::Camera(_)
            | SensorConfig::Custom(_)
            | SensorConfig::Unconfigured => None,
        }
    }
    pub fn air_pressure(&self) -> Option<&AirPressure> {
        match &self.config {
            SensorConfig::AirPressure(value) => Some(value),
            SensorConfig::Imu(_)
            | SensorConfig::Contact(_)
            | SensorConfig::ForceTorque(_)
            | SensorConfig::Magnetometer(_)
            | SensorConfig::Altimeter(_)
            | SensorConfig::NavSat(_)
            | SensorConfig::AirSpeed(_)
            | SensorConfig::Lidar(_)
            | SensorConfig::Camera(_)
            | SensorConfig::Custom(_)
            | SensorConfig::Unconfigured => None,
        }
    }
    pub fn navsat(&self) -> Option<&NavSat> {
        match &self.config {
            SensorConfig::NavSat(value) => Some(value),
            SensorConfig::Imu(_)
            | SensorConfig::Contact(_)
            | SensorConfig::ForceTorque(_)
            | SensorConfig::Magnetometer(_)
            | SensorConfig::Altimeter(_)
            | SensorConfig::AirPressure(_)
            | SensorConfig::AirSpeed(_)
            | SensorConfig::Lidar(_)
            | SensorConfig::Camera(_)
            | SensorConfig::Custom(_)
            | SensorConfig::Unconfigured => None,
        }
    }
    pub fn air_speed(&self) -> Option<&AirSpeed> {
        match &self.config {
            SensorConfig::AirSpeed(value) => Some(value),
            SensorConfig::Imu(_)
            | SensorConfig::Contact(_)
            | SensorConfig::ForceTorque(_)
            | SensorConfig::Magnetometer(_)
            | SensorConfig::Altimeter(_)
            | SensorConfig::AirPressure(_)
            | SensorConfig::NavSat(_)
            | SensorConfig::Lidar(_)
            | SensorConfig::Camera(_)
            | SensorConfig::Custom(_)
            | SensorConfig::Unconfigured => None,
        }
    }
    pub fn lidar(&self) -> Option<&Lidar> {
        match &self.config {
            SensorConfig::Lidar(value) => Some(value),
            SensorConfig::Imu(_)
            | SensorConfig::Contact(_)
            | SensorConfig::ForceTorque(_)
            | SensorConfig::Magnetometer(_)
            | SensorConfig::Altimeter(_)
            | SensorConfig::AirPressure(_)
            | SensorConfig::NavSat(_)
            | SensorConfig::AirSpeed(_)
            | SensorConfig::Camera(_)
            | SensorConfig::Custom(_)
            | SensorConfig::Unconfigured => None,
        }
    }
    pub fn camera(&self) -> Option<&Camera> {
        match &self.config {
            SensorConfig::Camera(value) => Some(value),
            _ => None,
        }
    }
    pub fn custom(&self) -> Option<&CustomSensor> {
        match &self.config {
            SensorConfig::Custom(value) => Some(value),
            _ => None,
        }
    }
}

/// Provenance of a model-scope merge after its model node has been flattened.
#[derive(Debug, Clone)]
pub struct MergedInclude {
    pub include: Box<Include>,
    pub model_source: Source,
    pub proxy_frame: String,
}

/// Original include values, independently of the resolved model/resource state.
#[derive(Debug, Clone)]
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
