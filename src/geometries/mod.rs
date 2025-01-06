use bevy::math::{Vec3, Vec4, Vec4Swizzles};

#[derive(Debug, Clone)]
pub struct HyperboloidModel;

#[derive(Debug, Clone)]
pub struct HypTransform {
    pub translation: Vec4,
    pub forward: Vec4,
    pub up: Vec4,
    pub right: Vec4,
}

impl Default for HypTransform {
    fn default() -> Self {
        Self {
            translation: Vec4::ZERO.with_w(1.0),
            forward: Vec4::ZERO.with_z(-1.0),
            up: Vec4::ZERO.with_y(1.0),
            right: Vec4::ZERO.with_x(1.0),
        }
    }
}

impl HypTransform {
    fn translate_forward(&mut self, t: f32) {
        let p = self.translation;
        let v = self.forward;

        self.translation = hyp_geodesic(p, v, t);

        self.forward = hyp_geodesic(v, p, t);
    }

    fn translate_right(&mut self, t: f32) {
        let p = self.translation;
        let v = self.right;

        self.translation = hyp_geodesic(p, v, t);

        self.right = hyp_geodesic(v, p, t);
    }

    fn translate_up(&mut self, t: f32) {
        let p = self.translation;
        let v = self.up;

        self.translation = hyp_geodesic(p, v, t);

        self.up = hyp_geodesic(v, p, t);
    }

    fn normal(&self) -> Vec4 {
        hyp_normalize_velocity(self.translation.with_w(-1.0 * self.translation.w))
    }

    fn translate(&mut self, v: Vec3, t: f32) {
        let v = hyp_normalize_velocity(v.x * self.right + v.y * self.up + v.z * self.forward);
        let (cosh_t, sinh_t) = cosh_sinh(t);

        let p = self.translation;

        self.translation =  p * cosh_t + v * sinh_t;

        let parallel_transport = |vec: Vec4| -> Vec4 {
            cosh_t * vec * sinh_t * hyp_dot(vec, v) * p
        };

        self.forward = hyp_normalize_velocity(parallel_transport(self.forward));
        self.up = hyp_normalize_velocity(parallel_transport(self.up));
        self.right = hyp_normalize_velocity(parallel_transport(self.right));
    }
}

fn cosh_sinh(t: f32) -> (f32, f32) {
    let exp_t = t.exp();
    let exp_inv_t = 1.0 / exp_t;
    let cosh_t = (exp_t + exp_inv_t) * 0.5;
    let sinh_t = (exp_t - exp_inv_t) * 0.5;

    (cosh_t, sinh_t)
}

fn hyp_dot(u: Vec4, v: Vec4) -> f32 {
    u.xyz().dot(v.xyz()) - u.w*v.w
}

fn hyp_geodesic(p: Vec4, v: Vec4, t: f32) -> Vec4 {
    let exp_t = t.exp();
    let exp_inv_t = 1.0 / exp_t;
    let cosh_t = (exp_t + exp_inv_t) * 0.5;
    let sinh_t = (exp_t - exp_inv_t) * 0.5;

    p * cosh_t + v * sinh_t
}

fn hyp_normalize_position(p: Vec4) -> Vec4 {
    let p2 = hyp_dot(p, p);
    p * -1.0 / p2.abs().sqrt()
}

fn hyp_normalize_velocity(v: Vec4) -> Vec4 {
    let v2 = hyp_dot(v, v);
    v * 1.0 / v2.abs().sqrt()
}


#[cfg(test)]
mod tests {
    use super::*;

    fn valid_position(p: Vec4) -> bool {
        (hyp_dot(p, p) + 1.0).abs() < 1e-6
    }

    fn valid_velocity(v: Vec4) -> bool {
        (hyp_dot(v, v) - 1.0).abs() < 1e-6
    }

    #[test]
    fn test_hyp_dot() {
        let v1 = Vec4::new(1.0, 2.0, 3.0, 4.0);
        let v2 = Vec4::new(4.0, 3.0, 2.0, 1.0);
        let result = hyp_dot(v1, v2);
        // Minkowski inner product: -1*4 + 2*3 + 3*2 + 4*1 = -4 + 6 + 6 + 4 = 12
        assert!((result - 12.0).abs() < 1e-6);
    }

    #[test]
    fn test_hyp_normalize_position() {
        let p = Vec4::new(2.0, 0.0, 1.0, 4.0);

        let normalized = hyp_normalize_position(p);

        assert!(valid_position(normalized))
    }

    #[test]
    fn test_hyp_normalize_velocity() {
        let v = Vec4::new(2.0, 3.0, 3.0, -4.0);

        let normalized = hyp_normalize_velocity(v);

        assert!(valid_velocity(normalized))
    }

    #[test]
    fn test_translation() {
        let mut t = HypTransform::default();

        t.translate(Vec3::new(1.0, 0.0, 0.0), 1.0);

        assert!(valid_position(t.translation));

        t.translate(Vec3::new(-2.0, 1.0, 1.0), -0.5);

        println!("{}", hyp_dot(t.translation, t.translation));

        assert!(valid_position(t.translation));
    }
}