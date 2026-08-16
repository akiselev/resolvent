//! Tensor-valued property frame transforms used by R15 material semantics.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SymmetricTensor2 {
    pub xx: f64,
    pub yy: f64,
    pub xy: f64,
}

impl SymmetricTensor2 {
    pub const fn isotropic(value: f64) -> Self {
        Self {
            xx: value,
            yy: value,
            xy: 0.0,
        }
    }

    pub fn matrix(self) -> [[f64; 2]; 2] {
        [[self.xx, self.xy], [self.xy, self.yy]]
    }

    /// Rotate a material-frame symmetric tensor into a target frame using `R A R^T`.
    pub fn rotate(self, angle_radians: f64) -> Self {
        let (s, c) = angle_radians.sin_cos();
        let r = [[c, -s], [s, c]];
        let a = self.matrix();
        let mut out = [[0.0; 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    for l in 0..2 {
                        out[i][j] += r[i][k] * a[k][l] * r[j][l];
                    }
                }
            }
        }
        Self {
            xx: out[0][0],
            yy: out[1][1],
            xy: 0.5 * (out[0][1] + out[1][0]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isotropic_tensor_is_rotation_invariant() {
        let a = SymmetricTensor2::isotropic(7.0);
        for angle in [0.0, 0.31, 1.2, 2.4] {
            let b = a.rotate(angle);
            assert!((b.xx - 7.0).abs() < 1e-12);
            assert!((b.yy - 7.0).abs() < 1e-12);
            assert!(b.xy.abs() < 1e-12);
        }
    }

    #[test]
    fn orthotropic_tensor_rotates_axes_and_creates_cross_term() {
        let a = SymmetricTensor2 {
            xx: 10.0,
            yy: 2.0,
            xy: 0.0,
        };
        let quarter = a.rotate(std::f64::consts::FRAC_PI_2);
        assert!((quarter.xx - 2.0).abs() < 1e-12);
        assert!((quarter.yy - 10.0).abs() < 1e-12);
        assert!(quarter.xy.abs() < 1e-12);

        let eighth = a.rotate(std::f64::consts::FRAC_PI_4);
        assert!((eighth.xx - 6.0).abs() < 1e-12);
        assert!((eighth.yy - 6.0).abs() < 1e-12);
        assert!((eighth.xy - 4.0).abs() < 1e-12);
    }
}
