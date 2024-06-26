use egui::Ui;
use crate::CurvEdit;

pub(crate) const MODIFIER_PANEL_WIDTH: f32 = 300f32;
pub(crate) const KEYFRAME_PANEL_HEIGHT: f32 = 300f32;

impl CurvEdit {
	pub(crate) fn curve_list<'a>(&mut self, _ui: &mut Ui) {
		//TODO

		let mut curves: Vec<(usize, usize)> = Vec::new();
		
		for table_num in 0 .. self.tables.len() {
			for curve_num in 0 .. self.tables[table_num].0.curves.len() {
				curves.push((table_num, curve_num));
			}
		}
		
		self.curves_to_show = curves;
	}
	
	pub(crate) fn current_keyframe(&mut self, _ui: &mut Ui) {
		//TODO
	}
}