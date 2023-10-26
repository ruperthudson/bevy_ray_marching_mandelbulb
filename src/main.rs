use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::mouse::MouseMotion,
    prelude::*,
    sprite::MaterialMesh2dBundle,
    window::WindowResized,
    window::WindowResolution,
};

use bevy_egui::EguiPlugin;

mod screen_space_quad;
use crate::screen_space_quad::ScreenSpaceQuad;

mod ray_marching_material;
use crate::ray_marching_material::{RayMarchingMaterial, RayMarchingMaterialPlugin};

mod ui;
use crate::ui::UIPlugin;

pub const WIDTH: f32 = 720.0;
pub const HEIGHT: f32 = 720.0;

/// System set to allow ordering of `PanCamPlugin`
#[derive(Debug, Clone, Copy, SystemSet, PartialEq, Eq, Hash)]
pub struct CamSystemSet;

fn main() {
    let mut app = App::new();

    app.insert_resource(ClearColor(Color::rgb(0.3, 0.3, 0.3)))
        .insert_resource(Msaa::Sample8)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(WIDTH, HEIGHT),
                title: "Ray Marching Scene".to_string(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        //.add_plugins(WorldInspectorPlugin::new())
        .add_plugins(RayMarchingMaterialPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins((EguiPlugin, UIPlugin))
        //Create the aspect ratio as a resource. Only one instance of this data is needed so a global resource was chosen
        .init_resource::<AspectRatio>()
        .init_resource::<MandelbulbUniforms>()
        .add_systems(Startup, setup)
        .add_systems(Update, resize_event)
        .add_systems(Update, process_camera_translation.in_set(CamSystemSet))
        .add_systems(Update, process_camera_rotation.in_set(CamSystemSet))
        .add_systems(Update, cursor_grab_system.in_set(CamSystemSet));

    app.init_resource::<EguiWantsFocus>()
        .add_systems(PostUpdate, check_egui_wants_focus)
        .configure_set(
            Update,
            CamSystemSet.run_if(resource_equals(EguiWantsFocus(false))),
        );

    app.run();
}

#[derive(Resource, Deref, DerefMut, PartialEq, Eq, Default)]
struct EguiWantsFocus(bool);

// todo: make run condition when Bevy supports mutable resources in them
fn check_egui_wants_focus(
    mut contexts: Query<&mut bevy_egui::EguiContext>,
    mut wants_focus: ResMut<EguiWantsFocus>,
) {
    let ctx = contexts.iter_mut().next();
    let new_wants_focus = if let Some(ctx) = ctx {
        let ctx = ctx.into_inner().get_mut();
        ctx.wants_pointer_input() || ctx.wants_keyboard_input()
    } else {
        false
    };
    wants_focus.set_if_neq(EguiWantsFocus(new_wants_focus));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<RayMarchingMaterial>>,
) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 3.0),
        ..default()
    });
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(ScreenSpaceQuad::default())).into(),
        material: materials.add(RayMarchingMaterial::new()),
        ..default()
    });
}

//Struct which becomes the Global Resource for the aspect ratio
#[derive(Default, Resource)]
pub struct AspectRatio {
    aspect_ratio: f32,
}
//Struct which becomes the Global Resource for the aspect ratio
#[derive(Resource)]
pub struct MandelbulbUniforms {
    power: f32,
    max_iterations: u32,
    bailout: f32,
    num_steps: u32,
    min_dist: f32,
    max_dist: f32,
}
impl Default for MandelbulbUniforms {
    fn default() -> Self {
        Self {
            power: 8.0,
            max_iterations: 8,
            bailout: 3.0,
            num_steps: 64,
            min_dist: 0.002,
            max_dist: 1000.0,
        }
    }
}

//Handle a window resize event to set the AspectRatio so it can be updated in the uniform that is sent to our shader
fn resize_event(
    mut resize_reader: EventReader<WindowResized>,
    mut aspect_ratio_resource: ResMut<AspectRatio>,
) {
    for event in resize_reader.iter() {
        aspect_ratio_resource.aspect_ratio = event.width / event.height;
    }
}

fn process_camera_translation(
    keys: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
) {
    // Constants for speed and default directions.
    const SPEED: f32 = 0.5;
    for mut transform in camera_query.iter_mut() {
        let forward: Vec3 = transform.forward(); // In Bevy, negative Z is typically "forward."
        let right: Vec3 = transform.right();
        let up: Vec3 = transform.up();

        // This will accumulate the total movement for this frame.
        let mut movement = Vec3::ZERO;

        // Check for key presses and adjust the movement vector accordingly.
        if keys.pressed(KeyCode::W) {
            movement += forward; // Note: moving "forward" typically means reducing the Z coordinate in many engines.
        }
        if keys.pressed(KeyCode::S) {
            movement -= forward;
        }
        if keys.pressed(KeyCode::A) {
            movement -= right;
        }
        if keys.pressed(KeyCode::D) {
            movement += right;
        }
        if keys.pressed(KeyCode::R) {
            movement += up;
        }
        if keys.pressed(KeyCode::F) {
            movement -= up;
        }

        // If there's any movement, normalize the vector to ensure consistent movement speed in all directions.
        if movement != Vec3::ZERO {
            movement = movement.normalize();
        }

        // Scale the movement by the speed and delta time, then apply it to the camera's translation.
        let translation_change = movement * SPEED * time.delta_seconds();
        transform.translation += translation_change;
    }
}

fn process_camera_rotation(
    mut motion_event: EventReader<MouseMotion>,
    windows: Query<&mut Window>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
) {
    let window = windows.single();

    for event in motion_event.iter() {
        const ROTATION_SPEED: f32 = 0.1;
        //if mouse_buttons.pressed(MouseButton::Right) {
        if window.cursor.grab_mode == CursorGrabMode::Locked {
            for mut transform in camera_query.iter_mut() {
                transform.rotate_local_x(-event.delta.y * ROTATION_SPEED * time.delta_seconds());
                transform.rotate_local_y(-event.delta.x * ROTATION_SPEED * time.delta_seconds());
            }
        }
        //}
    }
}

use bevy::window::CursorGrabMode;

// This system grabs the mouse when the left mouse button is pressed
// and releases it when the escape key is pressed
fn cursor_grab_system(
    mut windows: Query<&mut Window>,
    mouse: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let mut window = windows.single_mut();

    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }
}
