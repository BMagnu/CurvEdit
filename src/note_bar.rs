use std::cmp::PartialEq;
use std::time::{Duration, Instant};
use egui::{Align, Color32, Layout, Rounding, Stroke, Ui};
use crate::CurvEdit;

pub(crate) struct Note {
	pub(crate) text: String,
	pub(crate) severity: NoteSeverity,
	pub(crate) timeout: f32
}

#[derive(PartialEq)]
pub(crate) enum NoteSeverity {
	Info,
	Warning,
	Error
}

impl CurvEdit {
	pub(crate) fn note_bar(&mut self, ui: &mut Ui, ctx: &egui::Context) {
		let num_err = self.notes.iter().filter(|(note, _)| note.severity == NoteSeverity::Error).count();
		let num_warn = self.notes.iter().filter(|(note, _)| note.severity == NoteSeverity::Warning).count();
		let num_info = self.notes.iter().filter(|(note, _)| note.severity == NoteSeverity::Info).count();

		let to_show =
		if num_err > 0 {
			self.notes.iter_mut().enumerate().find(|(_, (note, _))| note.severity == NoteSeverity::Error)
		}
		else if num_warn > 0 {
			self.notes.iter_mut().enumerate().find(|(_, (note, _))| note.severity == NoteSeverity::Warning)
		}
		else if num_info > 0 {
			self.notes.iter_mut().enumerate().find(|(_, (note, _))| note.severity == NoteSeverity::Info)
		}
		else {
			None
		};

		ui.add_space(3f32);
		if let Some((idx, (note, timestamp))) = to_show {
			let timestamp: &_ = timestamp.get_or_insert(Instant::now());
			egui::Frame::none()
				.fill(
					if num_err > 0 { Color32::from_rgb(43, 27, 26) }
					else if num_warn > 0 { Color32::from_rgb(43, 40, 26) }
					else if num_info > 0 { Color32::from_rgb(42, 52, 71) }
					else { Color32::from_black_alpha(0) }
				)
				.stroke(
					Stroke::new(1f32,
						if num_err > 0 { Color32::from_rgb(255, 89, 64) }
						else if num_warn > 0 { Color32::from_rgb(255, 191, 64) }
						else if num_info > 0 { Color32::from_rgb(64, 140, 255) }
						else { Color32::from_black_alpha(0) }
					)
				)
				.rounding(Rounding::same(3f32))
				.show(ui, |ui| {
					ui.horizontal(|ui| {
						ui.add_space(4f32);
						ui.label(&note.text);
						ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
							ui.horizontal(|ui| {
								ui.add_space(4f32);
								//TODO Symbols instead of letters here
								ui.label(format!("E: {num_err}, W: {num_warn}, I: {num_info}"));
							});
						});
					});
				});

			if timestamp.elapsed().as_secs_f32() > note.timeout {
				self.notes.remove(idx);
				ctx.request_repaint();
			}
			else {
				ctx.request_repaint_after(Duration::from_secs_f32(note.timeout - timestamp.elapsed().as_secs_f32()));
			}
		}
		else {
			ui.add_space(2f32);
			ui.label("...");
			ui.add_space(2f32);
		}
		ui.add_space(3f32);
	}
}