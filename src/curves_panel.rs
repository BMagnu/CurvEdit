use eframe::emath::{Align, Vec2};
use egui::{Context, Id, Key, Layout, Ui, Widget};
use egui::CursorIcon::{Grabbing, PointingHand};
use crate::{CurvEdit, CurvEditInput};
use crate::plot_panel::plot_curve;

pub(crate) const CURVE_RENDER_ACCURACY: usize = 1500;

#[derive(Default, PartialEq)]
pub(crate) enum SnapMode {
	#[default]
	NoSnap,
	SnapX,
	SnapY,
	SnapCurve
}

impl CurvEdit {
	pub(crate) fn mode_panel(&mut self, ui: &mut Ui) {
		//TODO v1.1 Display Modes
		ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
			let response_curve = ui.scope(|ui| {
				ui.set_style(self.noto_symbols_buttons.clone());
				egui::Button::new("🗠").selected(self.snap_mode == SnapMode::SnapCurve).ui(ui)
			}).inner;
			let button_size = Vec2::new(15f32, response_curve.rect.height());
			
			if response_curve.on_hover_text("Snap to the curve as is displayed.").clicked() {
				self.snap_mode = SnapMode::SnapCurve;
			}
			
			//TODO v1.1 Z
			if egui::Button::new("Y").min_size(button_size.clone()).selected(self.snap_mode == SnapMode::SnapY).ui(ui).on_hover_text("Snap to the Y axis.").clicked() {
				self.snap_mode = SnapMode::SnapY;
			}
			if egui::Button::new("X").min_size(button_size.clone()).selected(self.snap_mode == SnapMode::SnapX).ui(ui).on_hover_text("Snap to the X axis.").clicked() {
				self.snap_mode = SnapMode::SnapX;
			}
			if egui::Button::new(" ").min_size(button_size).selected(self.snap_mode == SnapMode::NoSnap).ui(ui).on_hover_text("Don't snap.").clicked() {
				self.snap_mode = SnapMode::NoSnap;
			}
			ui.label("Snap to axis: ");
		});
	}
	
	pub(crate) fn curve_panel(&mut self, ui: &mut Ui, ctx: &Context) {
		let input = ui.input(|i| { CurvEditInput { 
			pointer_down: i.pointer.primary_down(),
			right_clicked: i.pointer.secondary_pressed(),
			ctrl_held: i.modifiers.ctrl,
			escape_pressed: i.key_pressed(Key::Escape)
		} });
		let cursor_group = Id::new("CursorGroup");
		let height = ui.available_height() / (self.curves_to_show.len() as f32) - 3f32;
		let mut is_dragging = false;

		//TODO v1.1 different plot modes
		for curve in &self.curves_to_show {
			ui.allocate_ui_with_layout(Vec2::new(ui.available_width(), height), Layout::top_down(Align::Center), |ui| {
				let name = self.tables[curve.0].0.curves[curve.1].name.as_str();
				ui.label(name);
				egui_plot::Plot::new(name)
					.allow_zoom(false)
					.allow_drag(false)
					.allow_scroll(false)
					.allow_boxed_zoom(false)
					.link_cursor(cursor_group, true, false)
					.show(ui, |plot_ui| plot_curve(plot_ui, ctx, &input, &mut self.tables, curve, &self.snap_mode, &mut is_dragging, &mut self.selected_keyframe))
			});
		}

		if is_dragging {
			ctx.output_mut(|o| o.cursor_icon = Grabbing);
		}
		else if input.ctrl_held {
			ctx.output_mut(|o| o.cursor_icon = PointingHand);
		}
	}
}