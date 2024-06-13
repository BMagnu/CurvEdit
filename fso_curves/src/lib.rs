struct Curve {
	name: String,
	keyframes: Vec<CurveKeyframe>
}

impl Curve {
	fn calculate(&self, x: f32) -> f32 {
		assert!(self.keyframes.len() >= 2);

		if self.keyframes[0].x >= x {
			return self.keyframes[0].y;
		}
		else if self.keyframes[self.keyframes.len() - 1].x < x {
			return self.keyframes[self.keyframes.len() - 1].y;
		}

		let result = self.keyframes[1..].iter().enumerate().find(|(_, kf)| kf.x < x).map(|(prev_index, kf)| {
			let prev_kf = &self.keyframes[prev_index];
			kf.segment.calculate(x, kf, prev_kf)
		});

		if let Some(result) = result {
			result
		}
		else {
			//At this point, no keyframe was matched. Should be impossible
			unreachable!("Keyframe not found");
		}
	}
}

struct CurveKeyframe {
	x: f32,
	y: f32,
	segment: Box<dyn CurveSegment>
}

trait CurveSegment {
	fn calculate(&self, x: f32, current: &CurveKeyframe, next: &CurveKeyframe) -> f32 {
		self.calculate_from_delta((x - current.x) / (next.x - current.x)) * (next.y - current.y) + current.y
	}
	fn calculate_from_delta(&self, t: f32) -> f32;
}

pub struct CurveSegmentConstant { }
impl CurveSegment for CurveSegmentConstant {
	fn calculate_from_delta(&self, _: f32) -> f32 { 0f32 }
}

pub struct CurveSegmentLinear { }
impl CurveSegment for CurveSegmentLinear {
	fn calculate_from_delta(&self, t: f32) -> f32 {
		t
	}
}

pub struct CurveSegmentPolynomial {
	ease_in: bool,
	degree: f32
}
impl CurveSegment for CurveSegmentPolynomial {
	fn calculate_from_delta(&self, t: f32) -> f32 {
		if self.ease_in {
			t.powf(self.degree)
		}
		else {
			1f32 - (1f32 - t).powf(self.degree)
		}
	}
}

pub struct CurveSegmentCircular {
	ease_in: bool
}
impl CurveSegment for CurveSegmentCircular {
	fn calculate_from_delta(&self, t: f32) -> f32 {
		if self.ease_in {
			1f32 - (1f32 - t.powi(2)).sqrt()
		}
		else {
			(1f32 - (1f32 - t).powi(2)).sqrt()
		}
	}
}

pub struct CurveSegmentSubcurve<'a> {
	curve: &'a Curve
}
impl CurveSegment for CurveSegmentSubcurve<'_> {
	fn calculate_from_delta(&self, t: f32) -> f32 {
		self.curve.calculate(t)
	}
}


pub fn add(left: usize, right: usize) -> usize {
	left + right
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
		let result = add(2, 2);
		assert_eq!(result, 4);
	}
}
