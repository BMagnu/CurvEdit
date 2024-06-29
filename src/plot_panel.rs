use eframe::emath::Vec2;
use eframe::epaint::Color32;
use egui::Id;
use egui_plot::{Line, MarkerShape, PlotPoints, PlotUi, Points};
use fso_tables_impl::curves::{BUILTIN_CURVES, Curve, CurveKeyframe, CurveSegment, CurveTable};
use crate::{CurvEditInput, TableData};
use crate::curves_panel::{CURVE_RENDER_ACCURACY, SnapMode};

pub(crate) const KEYFRAME_MIN_X_DISTANCE: f32 = 0.001;

pub(crate) fn from_curve (
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

pub(crate) fn get_available_curves(tables: &Vec<(CurveTable, TableData)>) -> Vec<&Curve> {
	let mut available_curves: Vec<&Curve> = BUILTIN_CURVES.iter().collect::<Vec<&Curve>>();
	for (table, _) in tables.iter() {
		available_curves.extend(table.curves.iter());
	}
	available_curves
}

pub(crate) fn plot_curve (plot_ui: &mut PlotUi, ctx: &egui::Context, input: &CurvEditInput, tables: &mut Vec<(CurveTable, TableData)>, curve_number: &(usize, usize), drag_mode: &SnapMode, is_dragging: &mut bool, selected_keyframe: &mut Option<(usize, usize, usize)>) {
	let available_curves = get_available_curves(tables);
	
	let curve = &tables[curve_number.0].0.curves[curve_number.1];
	let curve_points = from_curve( curve, &available_curves, CURVE_RENDER_ACCURACY);

	plot_ui.line(Line::new(curve_points).name(&curve.name));

	let point_size = Vec2::from(plot_ui.transform().dpos_dvalue().map(|v| (15f32 / v as f32).abs()));
	let mut point_bounds: Vec<(Vec2, Vec2)> = Vec::new();
	for (i, keyframe) in curve.keyframes.iter().enumerate() {
		let kf_point = Points::new(PlotPoints::new(vec![[keyframe.pos.0 as f64, keyframe.pos.1 as f64]]));
		point_bounds.push((Vec2::from(keyframe.pos) - point_size, Vec2::from(keyframe.pos) + point_size));
		plot_ui.points(kf_point.name(format!("Keyframe {}", i + 1))
			.filled(true)
			.radius(5f32)
			.shape(MarkerShape::Square)
			.color(Color32::from_rgb(102, 153, 255)));
	}

	type DraggingPntTuple = (usize, Vec2);
	let id_dragging = Id::new(format!("Dragging{}", curve.name));
	let was_dragging = ctx.memory(|mem| mem.data.get_temp::<DraggingPntTuple>(id_dragging));
	
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
				*selected_keyframe = Some((curve_number.0, curve_number.1, new_drag.0));

				*is_dragging = true;
				ctx.memory_mut(|mem| mem.data.insert_temp::<DraggingPntTuple>(id_dragging, new_drag));
			}
		}
		else if let Some((pnt, dragged)) = was_dragging {
			let lower_bound = if pnt <= 0 { -f32::INFINITY } else { curve.keyframes[pnt - 1].pos.0 + KEYFRAME_MIN_X_DISTANCE };
			let upper_bound = if pnt >= curve.keyframes.len() - 1 { f32::INFINITY } else { curve.keyframes[pnt + 1].pos.0 - KEYFRAME_MIN_X_DISTANCE };

			let kf = &curve.keyframes[pnt];
			
			let new_pos = match drag_mode {
				SnapMode::NoSnap => {
					(
						(kf.pos.0 + dragged.x).clamp(lower_bound, upper_bound),
						kf.pos.1 + dragged.y
					)
				}
				SnapMode::SnapX => {
					(
						(kf.pos.0 + dragged.x).clamp(lower_bound, upper_bound),
						kf.pos.1
					)
				}
				SnapMode::SnapY => {
					(
						kf.pos.0,
						kf.pos.1 + dragged.y
					)
				}
				SnapMode::SnapCurve => {
					let new_x = (kf.pos.0 + dragged.x).clamp(lower_bound, upper_bound);
					let new_y = curve.calculate(new_x, &available_curves);
					(
						new_x,
						new_y
					)
				}
			};
			
			ctx.memory_mut(|mem| mem.data.remove_temp::<DraggingPntTuple>(id_dragging));

			let table = &mut tables[curve_number.0];
			let curve = &mut table.0.curves[curve_number.1];
			
			table.1.dirty = true;
			curve.keyframes[pnt].pos = new_pos;
		}
		else if input.right_clicked {
			let point_clicked =  point_bounds.iter().enumerate().find(|(_, (bound_lower, bound_upper))| {
				bound_lower.x < mouse_coords.x && bound_lower.y < mouse_coords.y && bound_upper.x > mouse_coords.x && bound_upper.y > mouse_coords.y
			});

			if let Some((pnt, ..)) = point_clicked {
				if input.ctrl_held && curve.keyframes.len() > 2 {
					let table = &mut tables[curve_number.0];
					let curve = &mut table.0.curves[curve_number.1];
					
					curve.keyframes.remove(pnt);
					table.1.dirty = true;
				}
			}
			else {
				let point_upper =  curve.keyframes.iter().enumerate().find(|(_, kf)| {
					kf.pos.0 >= mouse_coords.x
				}).map(|(pnt, _)| pnt);
				
				let new_pos = match drag_mode {
					SnapMode::NoSnap | SnapMode::SnapY => {
						(
							mouse_coords.x,
							mouse_coords.y
						)
					}
					SnapMode::SnapX | SnapMode::SnapCurve => {
						let new_y = curve.calculate(mouse_coords.x, &available_curves);
						(
							mouse_coords.x,
							new_y
						)
					}
				};

				let table = &mut tables[curve_number.0];
				let curve = &mut table.0.curves[curve_number.1];

				table.1.dirty = true;
				
				if let Some(insert) = point_upper {
					curve.keyframes.insert(insert, CurveKeyframe::new(new_pos, CurveSegment::Constant));
				}
				else {
					curve.keyframes.push(CurveKeyframe::new(new_pos, CurveSegment::Constant));
				}
			}
		}
	}
}