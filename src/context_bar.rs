use egui::{menu, Ui};
use crate::CurvEdit;

impl CurvEdit {
	pub(crate) fn context_bar(&mut self, ui: &mut Ui) {
		menu::bar(ui, |ui| {
			ui.menu_button("File", |ui| {
				if ui.button("Open").clicked() {}
			});
		});
	}
}