use std::f32::consts::PI;

// use crate::MandelbulbUniforms;
use bevy::{
    prelude::*,
    reflect::TypePath,
    render::{render_resource::{AsBindGroup, ShaderRef, ShaderType}, storage::ShaderStorageBuffer},
    sprite::{Material2d, Material2dPlugin},
};

use crate::geometries::HypTransform;

pub struct RayMarchingMaterialPlugin;

impl Plugin for RayMarchingMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<RayMarchingMaterial>::default())
            .add_systems(PostUpdate, update_material)
            .insert_resource(RMCamera::default());
    }
}

#[derive(Component)]
#[require(Transform4D)]
pub struct RMRenderable {
    pub visible: bool,
    pub material: RMMaterial,
    pub shape: RMShape,
}

impl RMRenderable {
    pub fn sphere(radius: f32, material: RMMaterial) -> Self {
        Self {
            visible: true,
            material,
            shape: RMShape::Sphere { radius },
        }
    }

    pub fn hide(&mut self) -> &mut Self {
        self.visible = false;
        self
    }

    pub fn show(&mut self) -> &mut Self {
        self.visible = true;
        self
    }

    pub fn toggle_visibility(&mut self) -> &mut Self {
        self.visible = !self.visible;
        self
    }

    pub fn set_visibility(&mut self, visible: bool) -> &mut Self {
        self.visible = visible;
        self
    }
}

#[derive(Debug, Clone)]
pub enum RMShape {
    Sphere {
        radius: f32,
    },
}

#[derive(Debug, Clone)]
pub enum RMMaterial {
    Flat(LinearRgba),
}

#[derive(Debug, Clone)]
pub struct RMCameraSettings {
    pub aspect_ratio: f32,
    pub max_iterations: u32,
    pub max_dist: f32,
    pub min_dist: f32,
    pub tan_fov: f32,
}

impl Default for RMCameraSettings {
    fn default() -> Self {
        Self {
            aspect_ratio: 1.0,
            max_iterations: 300,
            max_dist: 1000.0,
            min_dist: 0.0001,
            tan_fov: (7.0/18.0*PI).tan(),
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct Transform4D {
    pub translation: Vec4,
    pub up: Vec4,
    pub right: Vec4,
    pub forward: Vec4,
}

impl Default for Transform4D {
    fn default() -> Self {
        Self {
            translation: Vec4::ZERO.with_w(1.0),
            up: Vec4::ZERO.with_y(1.0),
            right: Vec4::ZERO.with_x(1.0),
            forward: Vec4::ZERO.with_z(-1.0),
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct RMCamera {
    pub transform: HypTransform,
    pub settings: RMCameraSettings
}

#[derive(ShaderType, Clone, Debug)]
struct PreparedRMCamera {
    pub position: Vec4,
    pub forward: Vec4,
    pub right: Vec4,
    pub up: Vec4,
    pub aspect_ratio: f32,
    pub max_iterations: u32,
    pub min_dist: f32,
    pub max_dist: f32,
    pub tan_fov: f32,
}

impl Into<PreparedRMCamera> for RMCamera {
    fn into(self) -> PreparedRMCamera {
        (&self).into()
    }
}

impl Into<PreparedRMCamera> for &RMCamera {
    fn into(self) -> PreparedRMCamera {
        PreparedRMCamera {
            position: self.transform.translation,
            up: self.transform.up,
            right: self.transform.right,
            forward: self.transform.forward,
            aspect_ratio: self.settings.aspect_ratio,
            max_iterations: self.settings.max_iterations,
            max_dist: self.settings.max_dist,
            min_dist: self.settings.min_dist,
            tan_fov: self.settings.tan_fov,
        }
    }
}

#[derive(Debug, Clone, ShaderType)]
struct PreparedRMSphere {
    centre: Vec4,
    radius: f32,
    material_id: u32,
}

#[derive(Clone, Debug, Default, ShaderType)]
pub struct PreparedRMSpheres {
    #[size(runtime)]
    spheres: Vec<PreparedRMSphere>,
}


fn update_material(
    mut rm_mats: ResMut<Assets<RayMarchingMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    rm_camera: Res<RMCamera>,
    renderables: Query<(&Transform4D, &RMRenderable)>,
) {
    let mut spheres = Vec::new();
    for (transform, renderable) in renderables.iter() {
        if !renderable.visible {
            continue;
        }
        match renderable.shape {
            RMShape::Sphere { radius } => spheres.push(PreparedRMSphere {
                centre: transform.translation,
                radius,
                material_id: 1,
            }),
        }
    }
    for (_, rm_mat) in rm_mats.iter_mut() {
        rm_mat.camera = (&*rm_camera).into();
        buffers.get_mut(&rm_mat.spheres)
            .expect("buffer must exist")
            .set_data(PreparedRMSpheres {
                spheres: spheres.clone(),
            });
    }
}

//New material created to setup custom shader
#[derive(AsBindGroup, Debug, Clone, TypePath, Asset)]
pub struct RayMarchingMaterial {
    //Set the uniform at binding 0 to have the following information - connects to Camera struct in ray_marching_material.wgsl
    #[uniform(0)]
    camera: PreparedRMCamera,
    #[storage(1, read_only)]
    spheres: Handle<ShaderStorageBuffer>,
}

impl RayMarchingMaterial {
    pub fn from_buffers(mut buffers: ResMut<Assets<ShaderStorageBuffer>>) -> Self {
        let spheres = buffers.add(ShaderStorageBuffer::from(PreparedRMSpheres::default()));

        RayMarchingMaterial {
            camera: RMCamera::default().into(),
            spheres,
        }
    }
}

//Setup the RayMarchingMaterial to use the custom shader file for the vertex and fragment shader
//Note: one of these can be removed to use the default material 2D bevy shaders for the vertex/fragment shader
impl Material2d for RayMarchingMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/ray_marching_material.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/ray_marching_material.wgsl".into()
    }
}

//Uniform data struct to move data from the "Game World" to the "Render World" with the ShaderType derived
#[derive(ShaderType, Clone)]
struct RayMarchingMaterialUniformData {
    camera_position: Vec3,
    camera_forward: Vec3,
    camera_horizontal: Vec3,
    camera_vertical: Vec3,
    apsect_ratio: f32,
    power: f32,
    max_iterations: u32,
    bailout: f32,
    num_steps: u32,
    min_dist: f32,
    max_dist: f32,
    zoom: f32,
}
