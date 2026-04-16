use std::{array, usize};
use nalgebra_glm as glm;

const STRETCH_CONSTANT2: f32 = -0.211324865405187; // (1/sqrt(2+1)-1)/2
const SQUISH_CONSTANT2: f32 = 0.366025403784439;  // (sqrt(2+1)-1)/2
const NORM_CONSTANT2: f32 = 47.0;
const GRADIENTS2: [i8; 16] = [
    5, 2, 2, 5, -5, 2, -2, 5, 5, -2, 2, -5, -5, -2, -2, -5,
];

pub struct OpenSimplex {
    perm: [u8; 256],
}

pub struct FractalSample {
    pub frequency: f32,
    pub amplitude: f32,
}

impl FractalSample {
    pub fn new_harmonic<const LEN: usize>(
        start_amplitude: f32, start_frequency: f32,
        persistence: f32, lacunarity: f32,
    ) -> [FractalSample; LEN] {
        std::array::from_fn(|i|
            FractalSample {
                amplitude: start_amplitude * persistence.powi(i as i32),
                frequency: start_frequency * lacunarity.powi(i as i32),
            }
        )
    }
}

fn replace_zero(v: usize) -> usize {
    if v == 0 {
        1
    } else {
        v
    }
}

impl OpenSimplex {
    pub fn new(mut seed: u64) -> Self {
        let mut perm = array::from_fn(|i| i as u8);
        let mut source: [u8; 256] = array::from_fn(|i| i as u8);

        let next_it_seed = |x: u64| x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);

        for i in (0..256).rev() {
            seed = next_it_seed(seed);

            let r = (seed + 31) % (i + 1);

            perm[i as usize] = source[r as usize];
            source[r as usize] = source[i as usize];
        }

        return Self {
            perm,
        };
    }

    
    fn extrapolate2(&self, xsb: i32, ysb: i32, dx: f32, dy: f32) -> f32 {
        let index_x = (xsb & 0xFF) as usize;
        let index_y = (ysb & 0xFF) as u8;

        let hash = self.perm[index_x].wrapping_add(index_y) as usize;
        let index = (self.perm[hash] & 0x0E) as usize;

        let g1 = GRADIENTS2[index] as f32;
        let g2 = GRADIENTS2[index + 1] as f32;

        return g1 * dx + g2 * dy;
    }

    pub fn noise2(&self, position: glm::Vec2) -> f32 {
        let x = position.x;
        let y = position.y;

        let stretch_offset = (x + y) * STRETCH_CONSTANT2;
        let xs = x + stretch_offset;
        let ys = y + stretch_offset;

        let xsb_i = xs.floor() as i32;
        let ysb_i = ys.floor() as i32;

        let squish_offset = (xsb_i + ysb_i) as f32 * SQUISH_CONSTANT2;
        let xb = xsb_i as f32 + squish_offset;
        let yb = ysb_i as f32 + squish_offset;

        let xins = xs - xsb_i as f32;
        let yins = ys - ysb_i as f32;
        let in_sum = xins + yins;

        let dx0 = x - xb;
        let dy0 = y - yb;

        let mut value = 0.0;

        let dx1 = dx0 - 1.0 - SQUISH_CONSTANT2;
        let dy1 = dy0 - 0.0 - SQUISH_CONSTANT2;
        let mut attn1 = 2.0 - dx1 * dx1 - dy1 * dy1;
        if attn1 > 0.0 {
            attn1 *= attn1;
            value += attn1 * attn1 * self.extrapolate2(xsb_i + 1, ysb_i, dx1, dy1);
        }

        let dx2 = dx0 - 0.0 - SQUISH_CONSTANT2;
        let dy2 = dy0 - 1.0 - SQUISH_CONSTANT2;
        let mut attn2 = 2.0 - dx2 * dx2 - dy2 * dy2;
        if attn2 > 0.0 {
            attn2 *= attn2;
            value += attn2 * attn2 * self.extrapolate2(xsb_i, ysb_i + 1, dx2, dy2);
        }

        let (xsv_ext, ysv_ext, dx_ext, dy_ext);

        if in_sum <= 1.0 {
            let zins = 1.0 - in_sum;
            if zins > xins || zins > yins {
                if xins > yins {
                    xsv_ext = xsb_i + 1; ysv_ext = ysb_i - 1;
                    dx_ext = dx0 - 1.0; dy_ext = dy0 + 1.0;
                } else {
                    xsv_ext = xsb_i - 1; ysv_ext = ysb_i + 1;
                    dx_ext = dx0 + 1.0; dy_ext = dy0 - 1.0;
                }
            } else {
                xsv_ext = xsb_i + 1; ysv_ext = ysb_i + 1;
                dx_ext = dx0 - 1.0 - 2.0 * SQUISH_CONSTANT2;
                dy_ext = dy0 - 1.0 - 2.0 * SQUISH_CONSTANT2;
            }
        } else {
            let zins = 2.0 - in_sum;
            if zins < xins || zins < yins {
                if xins > yins {
                    xsv_ext = xsb_i + 2; ysv_ext = ysb_i;
                    dx_ext = dx0 - 2.0 - 2.0 * SQUISH_CONSTANT2;
                    dy_ext = dy0 + 0.0 - 2.0 * SQUISH_CONSTANT2;
                } else {
                    xsv_ext = xsb_i; ysv_ext = ysb_i + 2;
                    dx_ext = dx0 + 0.0 - 2.0 * SQUISH_CONSTANT2;
                    dy_ext = dy0 - 2.0 - 2.0 * SQUISH_CONSTANT2;
                }
            } else {
                dx_ext = dx0; dy_ext = dy0;
                xsv_ext = xsb_i; ysv_ext = ysb_i;
            }
            
            let dx0_final = dx0 - 1.0 - 2.0 * SQUISH_CONSTANT2;
            let dy0_final = dy0 - 1.0 - 2.0 * SQUISH_CONSTANT2;
            let mut attn0 = 2.0 - dx0_final * dx0_final - dy0_final * dy0_final;
            if attn0 > 0.0 {
                attn0 *= attn0;
                value += attn0 * attn0 * self.extrapolate2(xsb_i + 1, ysb_i + 1, dx0_final, dy0_final);
            }
            
            let mut attn_ext = 2.0 - dx_ext * dx_ext - dy_ext * dy_ext;
            if attn_ext > 0.0 {
                attn_ext *= attn_ext;
                value += attn_ext * attn_ext * self.extrapolate2(xsv_ext, ysv_ext, dx_ext, dy_ext);
            }

            return value / NORM_CONSTANT2;
        }

        let mut attn0 = 2.0 - dx0 * dx0 - dy0 * dy0;
        if attn0 > 0.0 {
            attn0 *= attn0;
            value += attn0 * attn0 * self.extrapolate2(xsb_i, ysb_i, dx0, dy0);
        }

        let mut attn_ext = 2.0 - dx_ext * dx_ext - dy_ext * dy_ext;
        if attn_ext > 0.0 {
            attn_ext *= attn_ext;
            value += attn_ext * attn_ext * self.extrapolate2(xsv_ext, ysv_ext, dx_ext, dy_ext);
        }

        return value / NORM_CONSTANT2;
    }

    pub fn fractal_sample(&self, position: glm::Vec2, sample_config: &[FractalSample]) -> f32 {
        sample_config.iter().fold(0.0, |prev, config|
            prev + self.noise2(position * config.frequency) * config.amplitude) / (replace_zero(sample_config.len()) as f32)
    }
}
