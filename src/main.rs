#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod plot_panel;
mod context_bar;
mod modifier_panel;
mod curves_panel;
mod note_bar;

use fso_tables_impl::curves::CurveTable;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use eframe::egui;
use eframe::epaint::FontFamily;
use egui::{FontData, FontDefinitions, FontId, FontTweak, Frame, Margin, Style, TextStyle};
use homedir::get_my_home;
use include_dir::{Dir, include_dir};
use crate::curves_panel::SnapMode;
use crate::modifier_panel::{KEYFRAME_PANEL_HEIGHT, MODIFIER_PANEL_WIDTH};
use crate::note_bar::Note;

const CURVEDIT_VERSION: &str = env!("CARGO_PKG_VERSION");
static ASSET_DIR: Dir = include_dir!("src/assets");

fn main() -> Result<(), Box<dyn Error>> {
	let mut args = std::env::args();
	args.next();
	let path = 
		if let Some(path) = args.next().map(|arg| PathBuf::from(arg.as_str())) { path }
		else if let Ok(Some(path)) = get_my_home() { path }
		else { PathBuf::from("") };
	
	let options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default()
			.with_inner_size([1080.0, 720.0]),
		..Default::default()
	};
	
	Ok(eframe::run_native(
		format!("CurvEdit {}", CURVEDIT_VERSION).as_str(),
		options,
		Box::new(move |ctx| Box::new({
			let mut curvedit = CurvEdit::default();
			curvedit.default_path = path;

			let mut fonts = FontDefinitions::default();
			fonts.font_data.insert(
				"NotoSymbols2".to_string(),
				FontData::from_static(ASSET_DIR.get_file("NotoSansSymbols2-Regular.ttf").unwrap().contents())
					.tweak(FontTweak {
						scale: 1.3,
						y_offset_factor: 0.05,
						y_offset: 0.0,
						baseline_offset_factor: 0.0,
					}),
			);
			fonts.families.insert(FontFamily::Name("NotoSymbols2".into()), vec!["NotoSymbols2".to_string()]);
			ctx.egui_ctx.set_fonts(fonts);
			
			let mut style = (*ctx.egui_ctx.style()).clone();
			style.text_styles.insert(TextStyle::Button, FontId::new(11f32, FontFamily::Name("NotoSymbols2".into())));
			curvedit.noto_symbols_buttons = Arc::new(style);
			
			curvedit
		})),
	)?)
}

#[derive(Default)]
struct TableData {
	file: PathBuf,
	dirty: bool
}

#[derive(Default)]
struct CurvEdit {
	tables: Vec<(CurveTable, TableData)>,
	curves_to_show: Vec<(usize, usize)>,
	notes: Vec<(Note, Option<Instant>)>,
	selected_keyframe: Option<(usize, usize, usize)>,
	snap_mode: SnapMode,
	default_path: PathBuf,
	noto_symbols_buttons: Arc<Style>
}
struct CurvEditInput {
	pointer_down: bool,
	right_clicked: bool,
	ctrl_held: bool,
	escape_pressed: bool
}

impl eframe::App for CurvEdit {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		egui::TopBottomPanel::top("context_bar").show(ctx, |ui| self.context_bar(ui));
		egui::TopBottomPanel::bottom("note_bar").show(ctx, |ui| self.note_bar(ui, ctx));
		egui::CentralPanel::default().frame(Frame::default().inner_margin(Margin {
			left: 0.0,
			right: 0.0,
			top: 0.0,
			bottom: 4.0,
		})).show(ctx, |ui| {
			egui::SidePanel::right("modifier_panel").frame(Frame::default().inner_margin(0f32)).resizable(false).exact_width(MODIFIER_PANEL_WIDTH).show_inside(ui, |ui| {
				egui::TopBottomPanel::bottom("keyframe_panel").exact_height(KEYFRAME_PANEL_HEIGHT).show_inside(ui, |ui| self.current_keyframe(ui, ctx));
				egui::CentralPanel::default().show_inside(ui, |ui| {
					egui::ScrollArea::vertical().show(ui, |ui| {
						self.curve_list(ui, ctx);
					});
				});
			});
			egui::CentralPanel::default().frame(Frame::default().inner_margin(0f32)).show_inside(ui, |ui| {
				egui::TopBottomPanel::top("mode_panel").show_inside(ui, |ui| {
					ui.vertical(|ui| {
						ui.add_space(2f32);
						ui.horizontal(|ui| self.mode_panel(ui));
						ui.add_space(2f32);
					});
				});
				egui::CentralPanel::default().show_inside(ui, |ui| self.curve_panel(ui, ctx));
			});
		});
	}
}


