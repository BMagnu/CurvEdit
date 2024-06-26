use std::path::PathBuf;
use egui::{menu, Ui};
use fso_tables_impl::curves::CurveTable;
use fso_tables_impl::FSOTableFileParser;
use native_dialog::FileDialog;
use crate::CurvEdit;
use crate::note_bar::{Note, NoteSeverity};
use crate::plot_panel::get_available_curves;

impl CurvEdit {
	pub(crate) fn context_bar(&mut self, ui: &mut Ui) {
		menu::bar(ui, |ui| {
			ui.menu_button("File", |ui| {
				if ui.button("Open Table").clicked() {
					let path = FileDialog::new()
						.set_location(self.tables.last().map_or(&PathBuf::default(), |table| &table.1))
						.add_filter("FSO Table", &["tbl", "tbm"])
						.show_open_single_file();
					if let Ok(Some(path)) = path {
						if path.file_name().is_some_and(|path| {
							let path = path.to_string_lossy().to_ascii_lowercase();
							path == "curves.tbl" || path.ends_with("-crv.tbm")
						}) {
							let table_parse = FSOTableFileParser::new(&path).and_then(|parser| CurveTable::parse(parser));
							match table_parse {
								Ok (table) => {
									let available_curves = get_available_curves(&self.tables);
									let name_collision = table.curves.iter().find(|curve| available_curves.iter().find(|other| other.name == curve.name).is_some());
									if let Some(curve) = name_collision {
										self.notes.push((Note {
											text: format!("Cannot add table {}, a curve with the name {} already exists!", path.file_name().unwrap_or("".as_ref()).to_string_lossy(), curve.name),
											severity: NoteSeverity::Error,
											timeout: 5f32
										}, None));
									}
									else {
										self.tables.push((table, path));
									}
								}
								Err (error) => {
									self.notes.push((Note {
										text: format!("Failed to parse {} at line {}: {}!", path.file_name().unwrap_or("".as_ref()).to_string_lossy(), error.line, error.reason),
										severity: NoteSeverity::Error,
										timeout: 5f32
									}, None));
								}
							}
						}
						else {
							self.notes.push((Note {
								text: format!("{} is not a curves table!", path.file_name().unwrap_or("".as_ref()).to_string_lossy()),
								severity: NoteSeverity::Error,
								timeout: 5f32
							}, None));
						}
					}
				}
				/*if ui.button("Open Tables Directory").clicked() {
					//TODO
				}*/
			});
		});
	}
}