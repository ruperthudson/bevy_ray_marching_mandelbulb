use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}, input::mouse::MouseMotion, prelude::*, render::storage::ShaderStorageBuffer, window::{CursorGrabMode, WindowResized, WindowResolution}
};

use bevy_egui::EguiPlugin;
use geometries::{hyp_dot, hyp_normalize, HypTransform};
use ray_marching_material::{RMCamera, RMMaterial, RMRenderable};

mod screen_space_quad;
use crate::screen_space_quad::ScreenSpaceQuad;

mod ray_marching_material;
use crate::ray_marching_material::{RayMarchingMaterial, RayMarchingMaterialPlugin};

mod ui;
use crate::ui::UIPlugin;

mod geometries;

pub const INIT_WIDTH: f32 = 720.0;
pub const INIT_HEIGHT: f32 = 720.0;
pub const WINDOW_NAME: &str = "Awesome game dude.";

/// System set to allow ordering of camera systems
#[derive(Debug, Clone, Copy, SystemSet, PartialEq, Eq, Hash)]
pub struct CamSystemSet;

fn main() {
    let mut app = App::new();

    app.insert_resource(ClearColor(Color::srgb(0.3, 0.3, 0.3)))
        // .insert_resource(Msaa::Sample8)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(INIT_WIDTH, INIT_HEIGHT),
                title: WINDOW_NAME.to_string(),
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

    app.insert_resource(Player { vertical_velocity: 0.0, grounded: true });

    app.run();
}

#[derive(Resource, Clone, Debug)]
struct Player {
    vertical_velocity: f32,
    grounded: bool,
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

    commands.spawn((
        RMRenderable::sphere(0.2, RMMaterial::Flat(LinearRgba::BLUE)),
        HypTransform::default()
            .translate(Vec3::new(0.0, 1.0, 1.0), 0.5)
            .clone(),
    ));
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
    mut player: ResMut<Player>,
) {
    // Constants for speed and default directions.
    const SPEED: f32 = 0.1;
    let yaw = rm_camera.orient.yaw();
    let forward = Vec3::new(yaw.sin(), 0.0, yaw.cos());
    let right = Vec3::new(yaw.cos(), 0.0, -1.0 * yaw.sin());
    let up = Vec3::ZERO.with_y(1.0);

    let height = {
        let t = rm_camera.transform.translation;
        (t.w + t.y).ln() - 0.1
    };

    if keys.just_pressed(KeyCode::Space) && player.grounded {
        player.vertical_velocity += 0.15
    }

    if height <= 0.0 {
        // rm_camera.transform.translate(up, height);
        // println!("{:?}", rm_camera.transform);
        player.vertical_velocity = player.vertical_velocity.max(0.0);
        rm_camera.transform.translate(up, player.vertical_velocity * time.delta_secs());
        player.grounded = true;
    } else {
        player.vertical_velocity -= 0.1 * time.delta_secs();
        rm_camera.transform.translate(up, player.vertical_velocity * time.delta_secs());
        player.vertical_velocity -= 0.1 * time.delta_secs();
        player.grounded = false;
    }

    // This will accumulate the total movement for this frame.
    let mut movement = Vec3::ZERO;

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
    if movement == Vec3::ZERO {
        return;
    }
    
    movement = movement.normalize();

    let n = Vec4::new(0.0, -1.0, 0.0, 1.0);
    
    rm_camera.transform
        .translate(movement, SPEED * time.delta_secs());

    let p = rm_camera.transform.translation;

    let v = n + hyp_dot(p, n) * p;

    rm_camera.transform
        .set_up(-1.0 * hyp_normalize(v));
}

fn process_camera_rotation(
    mut motion_event: EventReader<MouseMotion>,
    windows: Query<&mut Window>,
    mut rm_camera: ResMut<RMCamera>,
    time: Res<Time>,
) {
    let window = windows.single();

    for event in motion_event.read() {
        const ROTATION_SPEED: f32 = 0.1;
        if window.cursor_options.grab_mode == CursorGrabMode::Locked {
            rm_camera.orient
                .add_mut_yaw(event.delta.x * ROTATION_SPEED * time.delta_secs())
                .add_mut_pitch(-event.delta.y * ROTATION_SPEED * time.delta_secs());
        }
    }
}

fn log_pos_system(
    transforms: Query<&HypTransform>,
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
