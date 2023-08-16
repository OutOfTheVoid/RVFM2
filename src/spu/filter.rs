pub struct Filter {
    a: [f32; 2],
    b: [f32; 3],
}

pub enum FilterType {
    LowPass6,
    LowPass12,
    LowPass24,

    HighPass6,
    HighPass12,
    HighPass24,

    BandPass6,
    BandPass12,
    BandPass24,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            a: [0.0; 2],
            b: [0.0; 3],
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn compute(&mut self, x: f32, dt: f32, f: f32, params: &[i16]) -> f32 {
        // todo
        0.0
    }
}
