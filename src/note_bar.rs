use std::cmp::PartialEq;
use std::time::{Duration, Instant};
use egui::{Align, Layout, Ui};
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

		if let Some((idx, (note, timestamp))) = to_show {
			//TODO properly show severity

			let timestamp: &_ = timestamp.get_or_insert(Instant::now());

			ui.horizontal(|ui| {
				ui.label(&note.text);
				ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
					ui.horizontal(|ui| {
						ui.label(format!("E: {num_err}, W: {num_warn}, I: {num_info}"));
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
			ui.label("...");
		}
	}
}