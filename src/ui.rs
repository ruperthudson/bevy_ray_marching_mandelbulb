use crate::MandelbulbUniforms;
use bevy::prelude::*;

use bevy_egui::{egui, EguiContexts};

#[derive(Default)]
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, uniform_update_ui_system);
    }
}

fn uniform_update_ui_system(
    mut ctx: EguiContexts,
    mut mandelbulb_uniform_resource: ResMut<MandelbulbUniforms>,
) {
    let context = ctx.ctx_mut();
    egui::Window::new("Update Uniforms").show(context, |ui| {
        ui.horizontal(|ui| {
            ui.label("Mandelbulb Power:");
            ui.add(egui::Slider::new(
                &mut mandelbulb_uniform_resource.power,
                0.0..=100.0,
            ));
        });
        ui.horizontal(|ui| {
            ui.label("Mandelbulb Max Iterations:");
            ui.add(egui::Slider::new(
                &mut mandelbulb_uniform_resource.max_iterations,
                1..=128,
            ));
        });
        ui.horizontal(|ui| {
            ui.label("Mandelbulb Bailout:");
            ui.add(egui::Slider::new(
                &mut mandelbulb_uniform_resource.bailout,
                0.0..=3.0,
            ));
        });
        ui.horizontal(|ui| {
            ui.label("Mandelbulb Num Steps:");
            ui.add(egui::Slider::new(
                &mut mandelbulb_uniform_resource.num_steps,
                16..=500,
            ));
        });
        ui.horizontal(|ui| {
            ui.label("Mandelbulb Min Distance:");
            ui.add(egui::Slider::new(
                &mut mandelbulb_uniform_resource.min_dist,
                0.00000001..=0.1,
            ));
        });
        ui.horizontal(|ui| {
            ui.label("Mandelbulb Max Distance:");
            ui.add(egui::Slider::new(
                &mut mandelbulb_uniform_resource.max_dist,
                10.0..=10000.0,
            ));
        });
    });
}
