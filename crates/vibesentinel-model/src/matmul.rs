
pub fn linear_forward<const IN: usize, const OUT: usize>(
    x:   &[f32; IN],
    w:   &[[f32; IN]; OUT],
    b:   &[f32; OUT],
    out: &mut [f32; OUT],
) {
    for i in 0..OUT {
        let dot: f32 = w[i].iter()
            .zip(x.iter())
            .map(|(&wi, &xi)| wi * xi)
            .sum();
        out[i] = dot + b[i];
    }
}
