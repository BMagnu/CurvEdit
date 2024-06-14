use fso_curves::{BUILTIN_CURVES, Curve};

fn main() {
	for builtin in BUILTIN_CURVES.iter() {
		println!("{}", builtin.name)
	}

	let curve = BUILTIN_CURVES.iter().find(|curve| curve.name == "EaseInOutQuad");
	let available_curves = BUILTIN_CURVES.iter().map(|c| c).collect::<Vec<&Curve>>();
	
	if let Some(curve) = curve {
		println!("0: {}, 0.25: {}, 0.5: {}, 0.75: {}, 1: {}",
				 curve.calculate(0f32, &available_curves),
				 curve.calculate(0.25f32, &available_curves),
				 curve.calculate(0.5f32, &available_curves),
				 curve.calculate(0.75f32, &available_curves),
				 curve.calculate(1f32, &available_curves));
	}

}
