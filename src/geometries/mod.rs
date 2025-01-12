use bevy::{math::{Vec3, Vec4, Vec4Swizzles}, prelude::Component};

#[derive(Debug, Clone, Component)]
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
    pub fn translate_forward(&mut self, t: f32) -> &mut Self {
        let p = self.translation;
        let v = self.forward;

        self.translation = hyp_geodesic(p, v, t);

        self.forward = hyp_geodesic(v, p, t);

        self
    }

    pub fn translate_right(&mut self, t: f32) -> &mut Self {
        let p = self.translation;
        let v = self.right;

        self.translation = hyp_geodesic(p, v, t);

        self.right = hyp_geodesic(v, p, t);

        self
    }

    pub fn translate_up(&mut self, t: f32) -> &mut Self {
        let p = self.translation;
        let v = self.up;

        self.translation = hyp_geodesic(p, v, t);

        self.up = hyp_geodesic(v, p, t);

        self
    }

    pub fn normal(&self) -> Vec4 {
        hyp_normalize(self.translation.with_w(-1.0 * self.translation.w))
    }

    pub fn translate(&mut self, v: Vec3, t: f32) -> &mut Self {
        let v = hyp_normalize(v.x * self.right + v.y * self.up + v.z * self.forward);
        let (cosh_t, sinh_t) = cosh_sinh(t);

        let p = self.translation;

        self.translation =  hyp_normalize(p * cosh_t + v * sinh_t);

        let parallel_transport = |vec: Vec4| -> Vec4 {
            vec + hyp_dot(vec, v) * (v * (cosh_t - 1.0) + p * sinh_t)
        };

        self.forward = hyp_normalize(parallel_transport(self.forward));
        self.up = hyp_normalize(parallel_transport(self.up));
        self.right = hyp_normalize(parallel_transport(self.right));

        self
    }

    pub fn rotate_local_x(&mut self, theta: f32) -> &mut Self {
        let z = self.forward;
        let y = self.up;

        let cos_t = theta.cos();
        let sin_t = (1.0 - cos_t*cos_t).sqrt();

        self.forward = cos_t * z + sin_t * y;
        self.up = cos_t * y - sin_t * z;

        self
    }

    pub fn rotate_local_y(&mut self, theta: f32) -> &mut Self {
        let z = self.forward;
        let x = self.right;

        let cos_t = theta.cos();
        let sin_t = (1.0 - cos_t*cos_t).sqrt();

        self.forward = cos_t * z - sin_t * x;
        self.right = cos_t * x + sin_t * z;

        self
    }

    pub fn set_up(&mut self, up: Vec4) -> &mut Self {
        self.up = up;
        self.forward = hyp_normalize(self.forward - hyp_dot(self.forward, up) * up);
        self.right = hyp_normalize(self.right - hyp_dot(self.right, up) * up);

        self
    }
}

fn cosh_sinh(t: f32) -> (f32, f32) {
    let exp_t = t.exp();
    let exp_inv_t = 1.0 / exp_t;
    let cosh_t = (exp_t + exp_inv_t) * 0.5;
    let sinh_t = (exp_t - exp_inv_t) * 0.5;

    (cosh_t, sinh_t)
}

pub fn hyp_dot(u: Vec4, v: Vec4) -> f32 {
    u.xyz().dot(v.xyz()) - u.w*v.w
}

fn hyp_geodesic(p: Vec4, v: Vec4, t: f32) -> Vec4 {
    let exp_t = t.exp();
    let exp_inv_t = 1.0 / exp_t;
    let cosh_t = (exp_t + exp_inv_t) * 0.5;
    let sinh_t = (exp_t - exp_inv_t) * 0.5;

    p * cosh_t + v * sinh_t
}

pub fn hyp_normalize(p: Vec4) -> Vec4 {
    let p2 = hyp_dot(p, p);
    p * 1.0 / p2.abs().sqrt()
}

#[cfg(test)]
mod tests {
    use bevy::math::NormedVectorSpace;

    use super::*;

    const THRESH: f32 = 1e-6;

    fn valid_position(p: Vec4) -> bool {
        (hyp_dot(p, p) + 1.0).abs() < THRESH
    }

    fn is_unit(v: Vec4) -> bool {
        (hyp_dot(v, v) - 1.0).abs() < THRESH
    }

    fn is_orthogonal(u: Vec4, v: Vec4) -> bool {
        hyp_dot(u, v).abs() < THRESH
    }

    fn is_unit_tangent(v: Vec4, p: Vec4) -> bool {
        is_orthogonal(v, p) && is_unit(v)
    }

    fn approximately_identical(t0: HypTransform, t1: HypTransform) -> bool {
        (t0.translation - t1.translation).norm() < THRESH
            && (t0.translation - t1.translation).norm() < THRESH
            && (t0.forward - t1.forward).norm() < THRESH
            && (t0.up - t1.up).norm() < THRESH
            && (t0.right - t1.right).norm() < THRESH
    }

    fn is_valid_transform(t: &HypTransform) -> bool {
        return valid_position(t.translation)
            && is_unit_tangent(t.forward, t.translation)
            && is_unit_tangent(t.up, t.translation)
            && is_unit_tangent(t.right, t.translation)
            && is_orthogonal(t.forward, t.up)
            && is_orthogonal(t.up, t.right)
            && is_orthogonal(t.right, t.forward)
    }

    #[test]
    fn test_hyp_dot() {
        let v1 = Vec4::new(1.0, 2.0, 3.0, 4.0);
        let v2 = Vec4::new(4.0, 3.0, 2.0, 1.0);
        let result = hyp_dot(v1, v2);
        // Minkowski inner product: -1*4 + 2*3 + 3*2 + 4*1 = -4 + 6 + 6 + 4 = 12
        assert!((result - 12.0).abs() < THRESH);
    }

    #[test]
    fn test_hyp_normalize_position() {
        let p = Vec4::new(2.0, 0.0, 1.0, 4.0);

        let normalized = hyp_normalize(p);

        assert!(valid_position(normalized))
    }

    #[test]
    fn test_hyp_normalize_velocity() {
        let v = Vec4::new(2.0, 3.0, 3.0, -4.0);

        let normalized = hyp_normalize(v);

        assert!(is_unit(normalized))
    }

    #[test]
    fn test_translation() {
        let mut t = HypTransform::default();
        let t0 = t.clone();

        t.translate(Vec3::new(1.0, -1.0, 0.0), 1.0);

        assert!(true && is_valid_transform(&t));

        t.translate(Vec3::new(1.0, -1.0, 0.0), -1.0);

        assert!(is_valid_transform(&t));
        assert!(approximately_identical(t0, t))
    }
}