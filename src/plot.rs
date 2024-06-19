use eframe::emath::Vec2;
use eframe::epaint::Color32;
use egui::Id;
use egui_plot::{Line, MarkerShape, PlotPoints, PlotUi, Points};
use fso_tables_impl::curves::{BUILTIN_CURVES, Curve, CurveTable};
use crate::{CURVE_RENDER_ACCURACY, CurvEditInput};

pub fn from_curve (
	curve: &Curve,
	available_curves: &Vec<&Curve>,
	points: usize,
) -> PlotPoints {
	let (bounds, _) = curve.get_bounds();
	
	let increment = (bounds.end - bounds.start) / (points as f32);
	
	(0..points + 1)
		.map(|i| {
			let x = bounds.start + i as f32 * increment;
			(x as f64, curve.calculate(x, available_curves) as f64).into()
		})
		.collect()
}

pub fn plot_curve (plot_ui: &mut PlotUi, ctx: &egui::Context, input: &CurvEditInput, table: &mut CurveTable, curve_number: usize, is_dragging: &mut bool) {
	let available_curves = BUILTIN_CURVES.iter().chain(table.curves.iter()).map(|c| c).collect::<Vec<&Curve>>();
	let test_curve = &table.curves[curve_number];
	let curve_points = from_curve( test_curve, &available_curves, CURVE_RENDER_ACCURACY);

	drop(available_curves);

	plot_ui.line(Line::new(curve_points).name(&test_curve.name));

	let point_size = Vec2::from(plot_ui.transform().dpos_dvalue().map(|v| (15f32 / v as f32).abs()));
	let mut point_bounds: Vec<(Vec2, Vec2)> = Vec::new();
	for (i, keyframe) in test_curve.keyframes.iter().enumerate() {
		let kf_point = Points::new(PlotPoints::new(vec![[keyframe.pos.0 as f64, keyframe.pos.1 as f64]]));
		point_bounds.push((Vec2::from(keyframe.pos) - point_size, Vec2::from(keyframe.pos) + point_size));
		plot_ui.points(kf_point.name(format!("Keyframe {}", i + 1))
			.filled(true)
			.radius(5f32)
			.shape(MarkerShape::Square)
			.color(Color32::from_rgb(102, 153, 255)));
	}

	type DraggingPntTuple = (usize, Vec2);
	let id_dragging = Id::new(format!("Dragging{}", test_curve.name));
	let was_dragging = ctx.memory(|mem| mem.data.get_temp::<DraggingPntTuple>(id_dragging));

	let test_curve = &mut table.curves[curve_number];

	if let Some(mouse_coords) = plot_ui.pointer_coordinate() {
		let mouse_coords: Vec2 = mouse_coords.to_vec2();
		if plot_ui.response().hovered() && input.pointer_down {
			let pointer_translate = plot_ui.pointer_coordinate_drag_delta();

			let new_drag: Option<DraggingPntTuple> = if let Some((pnt, dragged)) = was_dragging {
				Some((pnt, dragged + pointer_translate))
			}
			else {
				point_bounds.iter().enumerate().find(|(_, (bound_lower, bound_upper))| {
					bound_lower.x < mouse_coords.x && bound_lower.y < mouse_coords.y && bound_upper.x > mouse_coords.x && bound_upper.y > mouse_coords.y
				}).map( |(i, _)| {
					(i, pointer_translate)
				})
			};

			if let Some(new_drag) = new_drag {
				*is_dragging = true;
				ctx.memory_mut(|mem| mem.data.insert_temp::<DraggingPntTuple>(id_dragging, new_drag));
			}
		}
		else if let Some((pnt, dragged)) = was_dragging {
			let kf = &mut test_curve.keyframes[pnt];
			kf.pos.0 += dragged.x;
			kf.pos.1 += dragged.y;

			ctx.memory_mut(|mem| mem.data.remove_temp::<DraggingPntTuple>(id_dragging));
		}
	}
}