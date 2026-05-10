use libm::expf;

#[inline(always)]
pub fn relu(x: f32) -> f32 { if x > 0.0 { x } else { 0.0 } }

#[inline(always)]
pub fn sigmoid(x: f32) -> f32 { 1.0 / (1.0 + expf(-x)) }

#[inline]
pub fn apply_relu<const N: usize>(arr: &mut [f32; N]) {
    for x in arr.iter_mut() { *x = relu(*x); }
}

#[inline]
pub fn apply_sigmoid<const N: usize>(arr: &mut [f32; N]) {
    for x in arr.iter_mut() { *x = sigmoid(*x); }
}
