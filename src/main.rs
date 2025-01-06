use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}, input::mouse::MouseMotion, prelude::*, render::storage::ShaderStorageBuffer, window::{CursorGrabMode, WindowResized, WindowResolution}
};

use bevy_egui::EguiPlugin;
use ray_marching_material::{RMCamera, RMMaterial, RMRenderable, Transform4D};

mod screen_space_quad;
use crate::screen_space_quad::ScreenSpaceQuad;

mod ray_marching_material;
use crate::ray_marching_material::{RayMarchingMaterial, RayMarchingMaterialPlugin};

mod ui;
use crate::ui::UIPlugin;

mod geometries;

pub const WIDTH: f32 = 720.0;
pub const HEIGHT: f32 = 720.0;

/// System set to allow ordering of camera systems
#[derive(Debug, Clone, Copy, SystemSet, PartialEq, Eq, Hash)]
pub struct CamSystemSet;

fn main() {
    let mut app = App::new();

    app.insert_resource(ClearColor(Color::srgb(0.3, 0.3, 0.3)))
        // .insert_resource(Msaa::Sample8)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(WIDTH, HEIGHT),
                title: "Ray Marching".to_string(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RayMarchingMaterialPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins((EguiPlugin, UIPlugin))
        //Create the aspect ratio as a resource. Only one instance of this data is needed so a global resource was chosen
        .add_systems(Startup, setup)
        .add_systems(Update, resize_event)
        .add_systems(Update, process_camera_translation.in_set(CamSystemSet))
        .add_systems(Update, process_camera_rotation.in_set(CamSystemSet))
        .add_systems(Update, cursor_grab_system.in_set(CamSystemSet))
        .add_systems(Update, log_pos_system);

    app.init_resource::<EguiWantsFocus>()
        .add_systems(PostUpdate, check_egui_wants_focus)
        .configure_sets(
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
    buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    commands.spawn((
        Camera2d,
        Msaa::Sample8,
    ));
    commands.spawn((
        // SyncToRenderWorld,
        Mesh2d(meshes.add(Mesh::from(ScreenSpaceQuad::default()))),
        MeshMaterial2d(materials.add(RayMarchingMaterial::from_buffers(buffers))),
    ));
    commands.spawn(
        RMRenderable::sphere(1.0, RMMaterial::Flat(LinearRgba::BLUE))
    );
}

//Handle a window resize event to set the AspectRatio so it can be updated in the uniform that is sent to our shader
fn resize_event(
    mut resize_reader: EventReader<WindowResized>,
    mut rm_camera: ResMut<RMCamera>,
) {
    for event in resize_reader.read() {
        rm_camera.settings.aspect_ratio = event.width / event.height;
    }
}

fn process_camera_translation(
    keys: Res<ButtonInput<KeyCode>>,
    mut rm_camera: ResMut<RMCamera>,
    time: Res<Time>,
) {
    // Constants for speed and default directions.
    const SPEED: f32 = 0.5;
    let forward: Vec4 = rm_camera.transform.forward;
    let right: Vec4 = rm_camera.transform.right;
    let up: Vec4 = rm_camera.transform.up;

    // This will accumulate the total movement for this frame.
    let mut movement = Vec4::ZERO;

    // Check for key presses and adjust the movement vector accordingly.
    if keys.pressed(KeyCode::KeyW) {
        movement += forward; // Note: moving "forward" typically means reducing the Z coordinate in many engines.
    }
    if keys.pressed(KeyCode::KeyS) {
        movement -= forward;
    }
    if keys.pressed(KeyCode::KeyA) {
        movement -= right;
    }
    if keys.pressed(KeyCode::KeyD) {
        movement += right;
    }
    if keys.pressed(KeyCode::KeyR) {
        movement += up;
    }
    if keys.pressed(KeyCode::KeyF) {
        movement -= up;
    }

    // If there's any movement, normalize the vector to ensure consistent movement speed in all directions.
    if movement != Vec4::ZERO {
        movement = movement.normalize();
    }

    // Scale the movement by the speed and delta time, then apply it to the camera's translation.
    let translation_change = movement * SPEED * time.delta_secs();
    
    rm_camera.transform.translation += translation_change;
}

fn process_camera_rotation(
    mut _motion_event: EventReader<MouseMotion>,
    windows: Query<&mut Window>,
    mut _rm_camera: ResMut<RMCamera>,
    _time: Res<Time>,
) {
    let _window = windows.single();

    // for event in motion_event.read() {
    //     const ROTATION_SPEED: f32 = 0.1;
    //     if window.cursor_options.grab_mode == CursorGrabMode::Locked {
    //         rm_camera.transform.rotate_local_x(-event.delta.y * ROTATION_SPEED * time.delta_secs());
    //         rm_camera.transform.rotate_local_y(-event.delta.x * ROTATION_SPEED * time.delta_secs());
    //     }
    // }
}

fn log_pos_system(
    transforms: Query<&Transform4D>,
    key: Res<ButtonInput<KeyCode>>,
    cam: Res<RMCamera>,
) {
    if key.just_pressed(KeyCode::KeyL) {
        println!("cam: {:?}", cam.transform);
        for t in transforms.iter() {
            println!("{t:?}")
        }
    }
}

// This system grabs the mouse when the left mouse button is pressed
// and releases it when the escape key is pressed
fn cursor_grab_system(
    mut windows: Query<&mut Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    let mut window = windows.single_mut();

    if mouse.just_pressed(MouseButton::Left) {
        window.cursor_options.visible = false;
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor_options.visible = true;
        window.cursor_options.grab_mode = CursorGrabMode::None;
    }
}
