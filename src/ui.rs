use crate::ray_marching_material::RMCamera;
use bevy::prelude::*;

use bevy_egui::{egui, EguiContexts};

#[derive(Default)]
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_systems(Startup, init_ui)
            .add_systems(Update, uniform_update_ui_system);
    }
}

// fn init_ui(
//     mut commands: Commands,
// ) {
//     commands.spawn(
//         Node {

//         }
//     )
// }

fn uniform_update_ui_system(
    mut ctx: EguiContexts,
    mut rm_camera: ResMut<RMCamera>,
) {
    let context = ctx.ctx_mut();
    egui::Window::new("Update Uniforms").show(context, |ui| {
        ui.horizontal(|ui| {
            ui.label("Mandelbulb Max Iterations:");
            ui.add(egui::Slider::new(
                &mut rm_camera.settings.max_iterations,
                1..=128,
            ));
        });
        ui.horizontal(|ui| {
            ui.label("Mandelbulb Min Distance:");
            ui.add(egui::Slider::new(
                &mut rm_camera.settings.min_dist,
                0.00000001..=0.01,
            ));
        });
        ui.horizontal(|ui| {
            ui.label("Mandelbulb Max Distance:");
            ui.add(egui::Slider::new(
                &mut rm_camera.settings.max_dist,
                10.0..=10000.0,
            ));
        });
        ui.horizontal(|ui| {
            ui.label("Mandelbulb TanFov:");
            ui.add(egui::Slider::new(
                &mut rm_camera.settings.tan_fov,
                1.0..=100.0,
            ));
        });
    });
}
