use egui_plot::PlotPoints;
use fso_curves::Curve;

pub fn from_curve (
	curve: &Curve,
	available_curves: &Vec<&Curve>,
	points: usize,
) -> PlotPoints {
	let (bounds, _) = curve.get_bounds();
	
	let increment = (bounds.end - bounds.start) / (points as f32);
	
	(0..points)
		.map(|i| {
			let x = bounds.start + i as f32 * increment;
			(x as f64, curve.calculate(x, available_curves) as f64).into()
		})
		.collect()
}