use egui::Ui;
use crate::CurvEdit;

pub(crate) struct Note {
	pub(crate) text: String,
	pub(crate) severity: NoteSeverity,
	pub(crate) timeout: f32
}

pub(crate) enum NoteSeverity {
	Info,
	Warning,
	Error
}

impl CurvEdit {
	pub(crate) fn note_bar(&mut self, ui: &mut Ui) {
		ui.label("Notes");
	}
}