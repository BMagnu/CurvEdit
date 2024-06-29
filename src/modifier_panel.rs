use std::fs;
use std::mem::swap;
use std::str::FromStr;
use std::time::Instant;
use egui::{Align, Id, Layout, Ui};
use fso_tables_impl::curves::{BUILTIN_CURVES, Curve, CurveKeyframe, CurveSegment, CurveTable};
use native_dialog::{MessageDialog, MessageType};
use crate::{CurvEdit, TableData};
use crate::note_bar::{Note, NoteSeverity};

pub(crate) const MODIFIER_PANEL_WIDTH: f32 = 300f32;
pub(crate) const KEYFRAME_PANEL_HEIGHT: f32 = 300f32;

pub(crate) const CURVE_LABEL_HEIGHT: f32 = 22f32;

impl CurvEdit {
	pub(crate) fn curve_list<'a>(&mut self, ui: &mut Ui, ctx: &egui::Context) {
		let mut curves: Vec<(usize, usize)> = Vec::new();
		let mut remove_table: Option<usize> = None;
		let mut rename_curves: Vec<(usize, usize, String)> = Vec::new();
		let mut add_curve: Option<(usize, String)> = None;

		for (table_num, (table, file_data)) in self.tables.iter_mut().enumerate() {
			ui.horizontal(|ui| {
				ui.set_height(CURVE_LABEL_HEIGHT);
				if table_entry(ui, table, file_data, &mut self.notes) {
					remove_table = Some(table_num);
				}
			});

			let mut remove_curve: Option<usize> = None;
			let mut switch_curves: Option<(usize, usize)> = None;
			for (curve_num, curve) in table.curves.iter().enumerate() {
				let is_clicked = self.curves_to_show.contains(&(table_num, curve_num));
				ui.horizontal(|ui| {
					ui.set_height(CURVE_LABEL_HEIGHT);
					let (display, remove, up, down, new_name) = curve_entry(ui, curve, ctx, is_clicked, curve_num < table.curves.len() - 1, curve_num > 0);
					let mut curve_num_to_display = switch_curves.map_or(curve_num, |(switch, other)| if other == curve_num { switch } else { curve_num });

					if remove {
						file_data.dirty = true;
						remove_curve = Some(curve_num);
					}
					if up {
						file_data.dirty = true;
						switch_curves = Some((curve_num - 1, curve_num));
						curve_num_to_display -= 1;
						curves = curves.iter().map(|(table, curve)| (*table, if *table == table_num && *curve == curve_num - 1 { *curve + 1 } else { *curve })).collect();
					}
					if down {
						file_data.dirty = true;
						//Other party is handled by map above
						switch_curves = Some((curve_num, curve_num + 1));
						curve_num_to_display += 1;
					}
					if display {
						curves.push((table_num, curve_num_to_display));
					}
					if let Some(new_name) = new_name {
						rename_curves.push((table_num, curve_num, new_name));
					}
				});
			}

			if let Some(to_remove) = remove_curve {
				table.curves.remove(to_remove);
				curves = curves.iter().filter(|(table, curve)| *table != table_num || *curve != to_remove).map(|(table, curve)| (*table, if *table == table_num && *curve > to_remove { *curve - 1 } else { *curve })).collect();
				if let Some((table, curve, _)) = self.selected_keyframe {
					if table == table_num && curve == to_remove {
						self.selected_keyframe = None;
					}
				}
			}
			if let Some((first, second)) = switch_curves {
				let (front, back) = table.curves.split_at_mut(second);
				swap(&mut front[first], &mut back[0]);
			}

			ui.horizontal(|ui| {
				ui.add_space(20f32);
				
				ui.label("Add curve: ");
				
				let id = Id::new(format!("new_curve_{}", file_data.file.to_string_lossy()));
				let was_editing = ctx.memory(|mem| mem.data.get_temp::<String>(id));
				let was_typing = was_editing.is_some();
				let mut name = was_editing.unwrap_or("".to_string());

				if ui.text_edit_singleline(&mut name).lost_focus() {
					add_curve = Some((table_num, name));
					ctx.memory_mut(|mem| mem.data.remove_temp::<String>(id));
				} else if name != "" {
					ctx.memory_mut(|mem| mem.data.insert_temp::<String>(id, name));
				} else if was_typing {
					ctx.memory_mut(|mem| mem.data.remove_temp::<String>(id));
				}
			});
		}

		for (table_num, curve_num, mut new_name) in rename_curves {
			if self.tables[table_num].0.curves[curve_num].name == new_name {
				continue;
			}
			if self.tables.iter().find(|(table, _)| table.curves.iter().find(|curve| curve.name == new_name).is_some()).is_some() {
				self.notes.push((Note {
					text: format!("Cannot rename {} to {}: Curve with this name already exists!", self.tables[table_num].0.curves[curve_num].name, new_name),
					severity: NoteSeverity::Error,
					timeout: 5f32
				}, None));
			}
			else {
				swap(&mut self.tables[table_num].0.curves[curve_num].name, &mut new_name);
				let old_name = new_name;
				let new_name = self.tables[table_num].0.curves[curve_num].name.clone();
				//We also need to find all references to this in subcurves and update them.
				for (table, file_data) in self.tables.iter_mut() {
					for curve in table.curves.iter_mut() {
						for keyframe in curve.keyframes.iter_mut() {
							match &mut keyframe.segment {
								CurveSegment::Subcurve { curve: ref mut name } if *name == old_name => {
									*name = new_name.clone();
									file_data.dirty = true;
								}
								_ => {}
							}
						}
					}
				}
				
				self.tables[table_num].1.dirty = true;
			}
		}
		
		if let Some(to_remove) = remove_table {
			self.tables.remove(to_remove);
			curves = curves.iter().filter(|(table, _)| *table != to_remove).map(|(table, curve)| (if *table > to_remove { *table - 1 } else { *table }, *curve)).collect();
			if let Some((table, _, _)) = self.selected_keyframe {
				if table == to_remove {
					self.selected_keyframe = None;
				}
			}
		}
		
		if let Some((table, name)) = add_curve {
			if self.tables.iter().find(|(table, _)| table.curves.iter().find(|curve| curve.name == name).is_some()).is_some() {
				self.notes.push((Note {
					text: format!("Cannot add {}: Curve with this name already exists!", name),
					severity: NoteSeverity::Error,
					timeout: 5f32
				}, None));
			}
			else {
				let keyframes = vec![
					CurveKeyframe::new((0f32, 0f32), CurveSegment::Linear),
					CurveKeyframe::new((1f32, 1f32), CurveSegment::Constant)
				];
				let (table, file_data) = &mut self.tables[table];
				file_data.dirty = true;
				table.curves.push(Curve::new(name, keyframes));
			}
		}

		self.curves_to_show = curves;
	}

	pub(crate) fn current_keyframe(&mut self, ui: &mut Ui, ctx: &egui::Context) {
		let id_x = Id::new("kf_data_x");
		let id_y = Id::new("kf_data_y");
		let id_deg = Id::new("kf_data_degree");
		let was_editing_x = ctx.memory(|mem| mem.data.get_temp::<String>(id_x));
		let was_editing_y = ctx.memory(|mem| mem.data.get_temp::<String>(id_y));
		let was_editing_deg = ctx.memory(|mem| mem.data.get_temp::<String>(id_deg));
		
		if let Some((table, curve, keyframe)) = self.selected_keyframe {
			ui.add_space(6f32);
			let list_of_curves = self.tables.iter().flat_map(|(table, _)| table.curves.iter().map(|curve| curve.name.clone()))
				.chain(BUILTIN_CURVES.iter().map(|curve| curve.name.clone())).collect::<Vec<String>>();
			
			let (table, file_data) = &mut self.tables[table];
			let curve = &mut table.curves[curve];
			let keyframe = &mut curve.keyframes[keyframe];
			ui.horizontal(|ui| {
				ui.label("X: ");

				let was_typing = was_editing_x.is_some();
				let pos_orig = format!("{}", keyframe.pos.0);
				let mut pos = was_editing_x.unwrap_or(pos_orig.clone());

				if ui.text_edit_singleline(&mut pos).lost_focus() {
					if let Ok(pos) = f32::from_str(pos.as_str()) {
						file_data.dirty = true;
						keyframe.pos.0 = pos;
					}
					ctx.memory_mut(|mem| mem.data.remove_temp::<String>(id_x));
				} else if pos != pos_orig {
					ctx.memory_mut(|mem| mem.data.insert_temp::<String>(id_x, pos));
				} else if was_typing {
					ctx.memory_mut(|mem| mem.data.remove_temp::<String>(id_x));
				}
			});
			ui.horizontal(|ui| {
				ui.label("Y: ");

				let was_typing = was_editing_y.is_some();
				let pos_orig = format!("{}", keyframe.pos.1);
				let mut pos = was_editing_y.unwrap_or(pos_orig.clone());

				if ui.text_edit_singleline(&mut pos).lost_focus() {
					if let Ok(pos) = f32::from_str(pos.as_str()) {
						file_data.dirty = true;
						keyframe.pos.1 = pos;
					}
					ctx.memory_mut(|mem| mem.data.remove_temp::<String>(id_y));
				} else if pos != pos_orig {
					ctx.memory_mut(|mem| mem.data.insert_temp::<String>(id_y, pos));
				} else if was_typing {
					ctx.memory_mut(|mem| mem.data.remove_temp::<String>(id_y));
				}
			});
			ui.horizontal(|ui| {
				ui.label("Interpolation type: ");
				if egui::ComboBox::from_id_source("interptype")
					.selected_text(
						match &keyframe.segment {
							CurveSegment::Constant => { "Constant" }
							CurveSegment::Linear => { "Linear" }
							CurveSegment::Polynomial { .. } => { "Polynomial" }
							CurveSegment::Circular { .. } => { "Circular" }
							CurveSegment::Subcurve { .. } => { "Subcurve" }
						})
					.show_ui(ui, |ui| {
						ui.selectable_value(&mut keyframe.segment, CurveSegment::Constant, "Constant");
						ui.selectable_value(&mut keyframe.segment, CurveSegment::Linear, "Linear");
						ui.selectable_value(&mut keyframe.segment, CurveSegment::Polynomial { degree: 2f32, ease_in: None }, "Polynomial");
						ui.selectable_value(&mut keyframe.segment, CurveSegment::Circular { ease_in: None }, "Circular");
						if let Some(first) = list_of_curves.iter().find(|string| **string != curve.name) {
							ui.selectable_value(&mut keyframe.segment, CurveSegment::Subcurve { curve: first.clone() }, "Subcurve");
						} 
					}).response.clicked() {
						file_data.dirty = true;
				}
			});

			match &mut keyframe.segment {
				CurveSegment::Polynomial { degree, ease_in } => {
					ui.horizontal(|ui| {
						ui.label("Degree: ");

						let was_typing = was_editing_deg.is_some();
						let deg_orig = format!("{}", degree);
						let mut deg = was_editing_deg.unwrap_or(deg_orig.clone());

						if ui.text_edit_singleline(&mut deg).lost_focus() {
							if let Ok(deg) = f32::from_str(deg.as_str()) {
								if deg > 0f32 {
									file_data.dirty = true;
									*degree = deg;
								}
							}
							ctx.memory_mut(|mem| mem.data.remove_temp::<String>(id_deg));
						} else if deg != deg_orig {
							ctx.memory_mut(|mem| mem.data.insert_temp::<String>(id_deg, deg));
						} else if was_typing {
							ctx.memory_mut(|mem| mem.data.remove_temp::<String>(id_deg));
						}
					});
					
					if ui.checkbox(ease_in.get_or_insert(true), "Ease In: ").changed() {
						file_data.dirty = true;
					}
				}
				CurveSegment::Circular { ease_in } => {
					if ui.checkbox(ease_in.get_or_insert(true), "Ease In: ").changed() {
						file_data.dirty = true;
					}
					ctx.memory_mut(|mem| mem.data.remove_temp::<String>(id_deg));
				}
				CurveSegment::Subcurve { curve: subcurve } => {
					ui.horizontal(|ui| {
						ui.label("Source curve: ");
						if egui::ComboBox::from_id_source("interptype_subcurve_curve")
							.selected_text(subcurve.as_str())
							.show_ui(ui, |ui| {
								for other_curve in list_of_curves.iter().filter(|string| **string != curve.name) {
									ui.selectable_value(subcurve, other_curve.clone(), other_curve);
								}
							}).response.clicked() {
							file_data.dirty = true;
						}
					});
					ctx.memory_mut(|mem| mem.data.remove_temp::<String>(id_deg));
				}
				_ => {
					ctx.memory_mut(|mem| mem.data.remove_temp::<String>(id_deg));
				}
			}
		}
		else {
			ctx.memory_mut(|mem| mem.data.remove_temp::<String>(id_x));
			ctx.memory_mut(|mem| mem.data.remove_temp::<String>(id_y));
			ctx.memory_mut(|mem| mem.data.remove_temp::<String>(id_deg));
		}
	}
}

fn table_entry(ui: &mut Ui, table: &CurveTable, file_data: &mut TableData, notes: &mut Vec<(Note, Option<Instant>)>) -> bool {
	let filename = file_data.file.file_name().map_or("".to_string(), |filename| filename.to_string_lossy().to_string());
	ui.label(&filename);

	ui.with_layout(Layout::right_to_left(Align::Center), |ui| -> bool {
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

fn curve_entry(ui: &mut Ui, curve: &Curve, ctx: &egui::Context, mut is_clicked: bool, can_go_down: bool, can_go_up: bool) -> (bool, bool, bool, bool, Option<String>) {
	//(display, remove, up, down)
	ui.add_space(20f32);

	ui.with_layout(Layout::right_to_left(Align::Center), |ui| -> (bool, bool, bool, bool, Option<String>) {
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

		let id = Id::new(format!("name_{}", curve.name));
		let was_editing = ctx.memory(|mem| mem.data.get_temp::<String>(id));
		let was_typing = was_editing.is_some();
		let mut new_name_return: Option<String> = None;

		let mut new_name = was_editing.unwrap_or(curve.name.clone());
		if ui.text_edit_singleline(&mut new_name).lost_focus() {
			new_name_return = Some(new_name);
			ctx.memory_mut(|mem| mem.data.remove_temp::<String>(id));
		}
		else if new_name != curve.name {
			ctx.memory_mut(|mem| mem.data.insert_temp::<String>(id, new_name));
		}
		else if was_typing {
			ctx.memory_mut(|mem| mem.data.remove_temp::<String>(id));
		}
		
		(is_clicked, remove, up, down, new_name_return)
	}).inner
}
