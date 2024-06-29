use std::fs;
use std::path::PathBuf;
use egui::{menu, Ui};
use fso_tables_impl::curves::CurveTable;
use fso_tables_impl::FSOTableFileParser;
use native_dialog::FileDialog;
use crate::{CurvEdit, TableData};
use crate::note_bar::{Note, NoteSeverity};
use crate::plot_panel::get_available_curves;

impl CurvEdit {
	pub(crate) fn context_bar(&mut self, ui: &mut Ui) {
		ui.add_space(2f32);
		menu::bar(ui, |ui| {
			ui.menu_button("File", |ui| {
				if ui.button("Open Table").clicked() {
					let path = FileDialog::new()
						.set_location(self.tables.last().map_or(&self.default_path, |table| &table.1.file))
						.set_filename("curves.tbl")
						.add_filter("FSO Table", &["tbl", "tbm"])
						.show_open_single_file();
					if let Ok(Some(path)) = path {
						self.try_open_file(path);
					}
				}
				if ui.button("Open Tables Directory").clicked() {
					let path = FileDialog::new()
						.set_location(self.tables.last().and_then(|table| table.1.file.parent()).unwrap_or(&self.default_path))
						.show_open_single_dir();
					if let Ok(Some(path)) = path {
						if let Ok(files) = fs::read_dir(path){
							for file in files {
								if let Ok(file) = file {
									let filename = file.file_name().to_string_lossy().to_ascii_lowercase();
									if filename == "curves.tbl" || filename.ends_with("-crv.tbm") {
										self.try_open_file(file.path());
									}
								}							
							}
						}
					}
				}
				if ui.button("New Table").clicked() {
					let path = FileDialog::new()
						.set_location(self.tables.last().map_or(&self.default_path, |table| &table.1.file))
						.set_filename("-crv.tbm")
						.add_filter("FSO Table", &["tbl", "tbm"])
						.show_save_single_file();
					if let Ok(Some(mut path)) = path {
						let filename = path.file_name().unwrap_or("curves.tbl".as_ref()).to_string_lossy().to_ascii_lowercase();
						if filename != "curves.tbl" && !filename.ends_with("-crv.tbm") {
							path.set_file_name(path.file_name().map(|filename| {
								let mut fname = filename.to_os_string();
								fname.push("-crv.tbm");
								fname
							}).unwrap());
						}
						
						match fs::write(&path, "") {
							Ok(_) => {
								self.tables.push((CurveTable::new(vec![]), TableData { file: path, dirty: true }));
							}
							Err(error) => {
								self.notes.push((Note {
									text: format!("Cannot save table {}: {}!", path.to_string_lossy(), error),
									severity: NoteSeverity::Error,
									timeout: 5f32
								}, None));
							}
						}
					}
				}
			});
		});
		ui.add_space(1f32);
	}
	
	fn try_open_file(&mut self, path: PathBuf) {
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
						self.tables.push((table, TableData { file: path, dirty: false }));
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