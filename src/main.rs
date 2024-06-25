#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod plot_panel;
mod context_bar;
mod modifier_panel;
mod curves_panel;

use fso_tables_impl::curves::CurveTable;
use std::error::Error;
use eframe::egui;
use egui::Frame;
use crate::modifier_panel::{KEYFRAME_PANEL_HEIGHT, MODIFIER_PANEL_WIDTH};

const CURVEDIT_VERSION: &str = env!("CARGO_PKG_VERSION");
fn main() -> Result<(), Box<dyn Error>> {
	let options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default()
			.with_inner_size([1080.0, 720.0]),
		..Default::default()
	};
	
	Ok(eframe::run_native(
		format!("CurvEdit {}", CURVEDIT_VERSION).as_str(),
		options,
		Box::new(|_| Box::new(CurvEdit::default())),
	)?)
}

#[derive(Default)]
struct CurvEdit {
	tables: Vec<CurveTable>,
	curves_to_show: Vec<(usize, usize)>
}
struct CurvEditInput {
	pointer_down: bool
}
impl eframe::App for CurvEdit {
	
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		egui::TopBottomPanel::top("context_bar").show(ctx, |ui| self.context_bar(ui));
		egui::CentralPanel::default().frame(Frame::default().inner_margin(0f32)).show(ctx, |ui| {
			egui::SidePanel::right("modifier_panel").frame(Frame::default().inner_margin(0f32)).resizable(false).exact_width(MODIFIER_PANEL_WIDTH).show_inside(ui, |ui| {
				egui::TopBottomPanel::bottom("keyframe_panel").min_height(KEYFRAME_PANEL_HEIGHT).show_inside(ui, |ui| self.current_keyframe(ui));
				egui::CentralPanel::default().show_inside(ui, |ui| self.curve_list(ui)).inner
			});
			egui::CentralPanel::default().frame(Frame::default().inner_margin(0f32)).show_inside(ui, |ui| {
				egui::TopBottomPanel::top("mode_panel").show_inside(ui, |ui| self.mode_panel(ui));
				egui::CentralPanel::default().show_inside(ui, |ui| self.curve_panel(ui, ctx));
			});
		});
	}
}


