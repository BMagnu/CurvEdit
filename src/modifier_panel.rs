use std::fs;
use std::mem::swap;
use std::time::Instant;
use egui::{Align, Layout, Ui, Vec2};
use fso_tables_impl::curves::{Curve, CurveTable};
use native_dialog::{MessageDialog, MessageType};
use crate::{CurvEdit, TableData};
use crate::note_bar::{Note, NoteSeverity};

pub(crate) const MODIFIER_PANEL_WIDTH: f32 = 300f32;
pub(crate) const KEYFRAME_PANEL_HEIGHT: f32 = 300f32;

impl CurvEdit {
	pub(crate) fn curve_list<'a>(&mut self, ui: &mut Ui) {
		//TODO

		let mut curves: Vec<(usize, usize)> = Vec::new();
		let mut remove_table: Option<usize> = None;

		for (table_num, (table, file_data)) in self.tables.iter_mut().enumerate() {
			ui.horizontal(|ui| {
				if table_entry(ui, table, file_data, &mut self.notes) {
					remove_table = Some(table_num);
				}
			});

			let mut remove_curve: Option<usize> = None;
			let mut switch_curves: Option<(usize, usize)> = None;
			for (curve_num, curve) in table.curves.iter().enumerate() {
				let is_clicked = self.curves_to_show.contains(&(table_num, curve_num));
				ui.horizontal(|ui| {
				let (display, remove, up, down) = curve_entry(ui, curve, is_clicked, curve_num < table.curves.len() - 1, curve_num > 0);
					let mut curve_num_to_display = switch_curves.map_or(curve_num, |(switch, other)| if other == curve_num { switch } else { curve_num });

					if remove {
						remove_curve = Some(curve_num);
					}
					if up {
						switch_curves = Some((curve_num - 1, curve_num));
						curve_num_to_display -= 1;
						curves = curves.iter().map(|(table, curve)| (*table, if *table == table_num && *curve == curve_num - 1 { *curve + 1 } else { *curve })).collect();
					}
					if down {
						//Other party is handled by map above
						switch_curves = Some((curve_num, curve_num + 1));
						curve_num_to_display += 1;
					}
					if display {

						curves.push((table_num, curve_num_to_display));
					}
				});
			}

			if let Some(to_remove) = remove_curve {
				table.curves.remove(to_remove);
				curves = curves.iter().filter(|(table, curve)| *table != table_num || *curve != to_remove).map(|(table, curve)| (*table, if *table == table_num && *curve > to_remove { *curve - 1 } else { *curve })).collect();
			}
			if let Some((first, second)) = switch_curves {
				let (front, back) = table.curves.split_at_mut(second);
				swap(&mut front[first], &mut back[0]);
			}
		}

		if let Some(to_remove) = remove_table {
			self.tables.remove(to_remove);
			curves = curves.iter().filter(|(table, _)| *table != to_remove).map(|(table, curve)| (if *table > to_remove { *table - 1 } else { *table }, *curve)).collect();
		}

		self.curves_to_show = curves;
	}

	pub(crate) fn current_keyframe(&mut self, _ui: &mut Ui) {
	}
}

fn table_entry(ui: &mut Ui, table: &CurveTable, file_data: &mut TableData, notes: &mut Vec<(Note, Option<Instant>)>) -> bool {
	let filename = file_data.file.file_name().map_or("".to_string(), |filename| filename.to_string_lossy().to_string());
	ui.label(&filename);

	ui.allocate_ui_with_layout(Vec2::new(ui.available_width(), 0f32), Layout::right_to_left(Align::Center), |ui| -> bool {
		let close = if ui.button("X").clicked() {
			if file_data.dirty {
				MessageDialog::new()
					.set_title("Close table?")
					.set_type(MessageType::Warning)
					.set_text(&format!("The table {} has unsaved changes. Are you sure you want to close the table and discard the changes?", filename))
					.show_confirm()
					.unwrap_or(false)
			}
			else {
				true
			}
		} else {
			false
		};
		if ui.button("S").clicked() {
			let table_content = table.spew();

			match fs::write(&file_data.file, table_content) {
				Ok(_) => {
					file_data.dirty = false;
				}
				Err(error) => {
					notes.push((Note {
						text: format!("Cannot save table {}: {}!", filename, error),
						severity: NoteSeverity::Error,
						timeout: 5f32
					}, None));
				}
			}
		}
		close
	}).inner
}

fn curve_entry(ui: &mut Ui, curve: &Curve, mut is_clicked: bool, can_go_down: bool, can_go_up: bool) -> (bool, bool, bool, bool) {
	//(display, remove, up, down)
	ui.add_space(20f32);
	ui.label(&curve.name);

	ui.allocate_ui_with_layout(Vec2::new(ui.available_width(), 0f32), Layout::right_to_left(Align::Center), |ui| -> (bool, bool, bool, bool) {
		let remove = if ui.button("X").clicked() {
			MessageDialog::new()
				.set_title("Delete curve?")
				.set_type(MessageType::Warning)
				.set_text(&format!("Are you sure you want to delete the curve {}?", curve.name))
				.show_confirm()
				.unwrap_or(false)
		} else {
			false
		};
		let up = ui.add_enabled(can_go_up, egui::Button::new("U")).clicked();
		let down = ui.add_enabled(can_go_down, egui::Button::new("D")).clicked();
		ui.toggle_value(&mut is_clicked, "S");
		(is_clicked, remove, up, down)
	}).inner
}
