#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod plot;

use std::error::Error;
use std::path::Path;
use fso_curves::{CurveTable};
use fso_tables::{FSOTable, FSOTableFileParser};
use eframe::egui;
use eframe::emath::Align;
use egui::{Id, Layout, Vec2};
use egui::CursorIcon::Grabbing;
use crate::plot::plot_curve;

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

const CURVE_RENDER_ACCURACY: usize = 500;

#[derive(Default)]
struct CurvEdit {
	table: Option<CurveTable>

}
struct CurvEditInput {
	pointer_down: bool
}
impl eframe::App for CurvEdit {
	
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		if let Some(table) = &mut self.table{
			egui::CentralPanel::default().show(ctx, |ui| {
				let input = ui.input(|i| { CurvEditInput { pointer_down: i.pointer.primary_down() } });

				let cursor_group = Id::new("CursorGroup");
				let mut is_dragging = false;

				ui.label("This example shows how to use raw input events to implement different plot controls than the ones egui provides by default, e.g., default to zooming instead of panning when the Ctrl key is not pressed, or controlling much it zooms with each mouse wheel step.");

				let total_h = ui.available_height();

				ui.allocate_ui_with_layout(Vec2::new(ui.available_width(), total_h / 2f32), Layout::top_down(Align::Center), |ui|
					egui_plot::Plot::new("plot1")
						.allow_zoom(false)
						.allow_drag(false)
						.allow_scroll(false)
						.allow_boxed_zoom(false)
						.link_cursor(cursor_group, true, false)
						.show(ui, |plot_ui| {
							plot_curve(plot_ui, ctx, &input, table, 0, &mut is_dragging);
						})
				);
				ui.allocate_ui_with_layout(Vec2::new(ui.available_width(), total_h / 2f32), Layout::top_down(Align::Center), |ui|
				egui_plot::Plot::new("plot2")
					.allow_zoom(false)
					.allow_drag(false)
					.allow_scroll(false)
					.allow_boxed_zoom(false)
					.link_cursor(cursor_group, true, false)
					.show(ui, |plot_ui| {
						plot_curve(plot_ui, ctx, &input, table, 1, &mut is_dragging);
					})
				);

				if is_dragging {
					ctx.output_mut(|o| o.cursor_icon = Grabbing);
				}
			});
		}
		else{
			let table = FSOTableFileParser::new(Path::new("/home/birk/Downloads/curves.tbl")).and_then(|parser| CurveTable::parse(&parser));
			if let Ok(table) = table{
				self.table = Some(table);
			}
			else{
				//Be sad :(
			}
		}
	}
}


