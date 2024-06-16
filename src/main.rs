#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod plot;

use std::error::Error;
use std::path::Path;
use fso_curves::{BUILTIN_CURVES, Curve, CurveTable};
use fso_tables::{FSOTable, FSOTableFileParser};
use eframe::egui;
use egui::Id;
use egui_plot::{Legend, Line};

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
impl eframe::App for CurvEdit {
	
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		if let Some(table) = &mut self.table{
			let available_curves = BUILTIN_CURVES.iter().chain(table.curves.iter()).map(|c| c).collect::<Vec<&Curve>>();
			
			let test_curve = &table.curves[1];

			egui::CentralPanel::default().show(ctx, |ui| {
				let cursor_group = Id::new("CursorGroup");
				
				ui.label("This example shows how to use raw input events to implement different plot controls than the ones egui provides by default, e.g., default to zooming instead of panning when the Ctrl key is not pressed, or controlling much it zooms with each mouse wheel step.");
				egui_plot::Plot::new("plot")
					.allow_zoom(false)
					.allow_drag(false)
					.allow_scroll(false)
					.allow_boxed_zoom(false)
					.link_cursor(cursor_group, true, false)
					.legend(Legend::default())
					.show(ui, |plot_ui| {
						let curve_points = plot::from_curve( test_curve, &available_curves, CURVE_RENDER_ACCURACY);
						plot_ui.line(Line::new(curve_points).name(&test_curve.name));
					});
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


