pub use self::curve::*;
mod curve;

#[cfg(test)]
mod tests {
	use super::curve::*;

	#[test]
	fn builtin() {
		let curve = BUILTIN_CURVES.iter().find(|curve| curve.name == "EaseInOutQuad");
		assert!(curve.is_some());
		
		let curve = curve.unwrap();
		
		assert!((curve.calculate(0f32) - 0f32).abs() < 0.001);
		assert!((curve.calculate(0.25f32) - 0.125f32).abs() < 0.001);
		assert!((curve.calculate(0.5f32) - 0.5f32).abs() < 0.001);
		assert!((curve.calculate(0.75f32) - 0.875f32).abs() < 0.001);
		assert!((curve.calculate(1f32) - 1f32).abs() < 0.001);
	}

	#[test]
	fn subcurve() {
		let ease_in_quad = BUILTIN_CURVES.iter().find(|curve| curve.name == "EaseInQuad");
		assert!(ease_in_quad.is_some());

		let ease_in_quad = ease_in_quad.unwrap();
		let curve = Curve {
			name: "".to_string(),
			keyframes: vec![
				CurveKeyframe{ x: 0f32, y: 0f32, segment: CurveSegment::Subcurve { curve: ease_in_quad } },
				CurveKeyframe{ x: 0.5f32, y: 0.5f32, segment: CurveSegment::Polynomial { ease_in: true, degree: 2f32 } },
				CurveKeyframe{ x: 1f32, y: 1f32, segment: CurveSegment::Constant }
			]
		};

		assert!((curve.calculate(0f32) - 0f32).abs() < 0.001);
		assert!((curve.calculate(0.25f32) - 0.125f32).abs() < 0.001);
		assert!((curve.calculate(0.5f32) - 0.5f32).abs() < 0.001);
		assert!((curve.calculate(0.75f32) - 0.625f32).abs() < 0.001);
		assert!((curve.calculate(1f32) - 1f32).abs() < 0.001);
	}
}
