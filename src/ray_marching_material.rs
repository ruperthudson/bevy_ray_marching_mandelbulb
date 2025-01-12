use std::f32::consts::{PI, TAU};

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
        let mut cam = RMCamera::default();
        cam.transform.translate(Vec3::new(0.0, 1.0, 0.0), 0.5);
        println!("{:?}", cam );
        app.add_plugins(Material2dPlugin::<RayMarchingMaterial>::default())
            .add_systems(PostUpdate, update_material)
            .insert_resource(cam);
    }
}

#[derive(Component)]
#[require(HypTransform)]
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
            max_dist: 100.0,
            min_dist: 0.0001,
            tan_fov: (7.0/18.0*PI).tan(),
        }
    }
}

// #[derive(Debug, Clone, Component)]
// pub struct Transform4D {
//     pub translation: Vec4,
//     pub up: Vec4,
//     pub right: Vec4,
//     pub forward: Vec4,
// }

// impl Default for Transform4D {
//     fn default() -> Self {
//         Self {
//             translation: Vec4::ZERO.with_w(1.0),
//             up: Vec4::ZERO.with_y(1.0),
//             right: Vec4::ZERO.with_x(1.0),
//             forward: Vec4::ZERO.with_z(-1.0),
//         }
//     }
// }

#[derive(Debug, Clone, Default)]
pub struct LocalOrient {
    yaw: f32,
    pitch: f32,
}

impl LocalOrient {
    pub fn mat3(&self) -> Mat3 {
        Mat3::from_rotation_y(self.yaw).mul_mat3(&Mat3::from_rotation_x(-self.pitch))
    }

    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    pub fn set_yaw(&mut self, yaw: f32) -> &mut Self {
        self.yaw = yaw % TAU;
        self
    }

    pub fn set_pitch(&mut self, pitch: f32) -> &mut Self {
        self.pitch = pitch.clamp(-PI/2.0, PI/2.0);
        self
    }

    pub fn add_mut_yaw(&mut self, delta_yaw: f32) -> &mut Self {
        self.set_yaw(self.yaw() + delta_yaw);
        self
    }

    pub fn add_mut_pitch(&mut self, delta_pitch: f32) -> &mut Self {
        self.set_pitch(self.pitch() + delta_pitch);
        self
    }

    pub fn into_global_orient(&self, transform: &HypTransform) -> [Vec4; 3] {
        into_global_orient(self.mat3(), transform)
    }
}

pub fn into_global_orient(mat3: Mat3, transform: &HypTransform) -> [Vec4; 3] {
    let mat4 = Mat4::from_cols(transform.right, transform.up, transform.forward, Vec4::ZERO);
    let res = mat4.mul_mat4(&Mat4::from_mat3(mat3));

    [res.x_axis, res.y_axis, res.z_axis]
}

#[derive(Resource, Debug, Clone, Default)]
pub struct RMCamera {
    pub transform: HypTransform,
    pub settings: RMCameraSettings,
    pub orient: LocalOrient,
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
        let orient = self.orient.into_global_orient(&self.transform);
        PreparedRMCamera {
            position: self.transform.translation,
            forward: orient[2],
            right: orient[0],
            up: orient[1],
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
    renderables: Query<(&HypTransform, &RMRenderable)>,
    time: Res<Time>
) {
    let mut spheres = Vec::new();

    let tf = rm_camera.transform
        .clone()
        .translate_forward(1.0)
        .clone();

    let tr = rm_camera.transform
        .clone()
        .translate_right(1.0)
        .clone();

    let tu = rm_camera.transform
        .clone()
        .translate_up(1.0)
        .clone();

    {
        let trans = HypTransform::default()
            .translate(Vec3::new(2.0, -1.0, 0.5), 2.0)
            .clone();

        let mat = Mat3::from_rotation_x(time.elapsed_secs() * 0.5)
            .mul_mat3(&Mat3::from_rotation_y(time.elapsed_secs() * 0.1))
            .mul_mat3(&Mat3::from_rotation_z(time.elapsed_secs() * 0.05));

        let flow = |v: Vec3| {
            trans.clone()
                .translate(v, 0.5)
                .clone()
                .translation
        };

        let c1 = flow(mat.x_axis + mat.y_axis + mat.z_axis);
        let c2 = flow(mat.x_axis + mat.y_axis - mat.z_axis);
        let c3 = flow(mat.x_axis - mat.y_axis + mat.z_axis);
        let c4 = flow(mat.x_axis - mat.y_axis - mat.z_axis);
        let c5 = flow(- mat.x_axis + mat.y_axis + mat.z_axis);
        let c6 = flow(- mat.x_axis + mat.y_axis - mat.z_axis);
        let c7 = flow(- mat.x_axis - mat.y_axis + mat.z_axis);
        let c8 = flow(- mat.x_axis - mat.y_axis - mat.z_axis);

        for c in [c1, c2, c3, c4, c5, c6, c7, c8] {
            spheres.push(PreparedRMSphere {
                centre: c,
                radius: 0.075,
                material_id: 3,
            })
        }
    }

    spheres.push(PreparedRMSphere {
        centre: tf.translation,
        radius: 0.05,
        material_id: 4,
    });

    spheres.push(PreparedRMSphere {
        centre: tr.translation,
        radius: 0.05,
        material_id: 5,
    });

    spheres.push(PreparedRMSphere {
        centre: tu.translation,
        radius: 0.05,
        material_id: 6,
    });

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
