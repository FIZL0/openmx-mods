//! A camera pose: where the original's camera was when its shot was taken, and the same pose as
//! openmx's `--shot-pose` takes it (openmx MD10: `x,y,z,yaw,pitch,fov`).
//!
//! Angles are degrees. Yaw is the heading of the camera's forward vector about +y, 0 looking along +z
//! and 90 along +x (`atan2(forward.x, forward.z)`); pitch is its elevation, positive looking up
//! (`asin(forward.y)`). The fov is the viewport's own number (85 on MX vs. ATV's ride camera); which
//! axis it spans is in `notes/accuracy.md`. Roll is not carried: the follow camera keeps its up vector
//! in the plane of forward and +y.

use crate::ram::quad;
use crate::source::Viewport;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub eye: [f32; 3],
    pub forward: [f32; 3],
    pub up: [f32; 3],
    pub fov: f32,
}

impl Pose {
    /// The pose in a viewport at `at` in `bytes`, which is RAM from address `base` on (a whole image with
    /// `base` 0, or a block read over PINE).
    pub fn from_viewport(bytes: &[u8], base: u32, at: u32, vp: &Viewport) -> Result<Pose, String> {
        let off = at.checked_sub(base).ok_or("the viewport is before the block")?;
        let q = |o: u32| quad(bytes, off + o).ok_or_else(|| format!("the block ends before the viewport's +{o:#x}"));
        let (e, f, u) = (q(vp.eye)?, q(vp.forward)?, q(vp.up)?);
        let fov = quad(bytes, off + vp.fov).ok_or("the block ends before the fov")?[0];
        let p = Pose { eye: [e[0], e[1], e[2]], forward: [f[0], f[1], f[2]], up: [u[0], u[1], u[2]], fov };
        if e[3] != 1.0 || !p.is_finite() {
            return Err(format!("not a viewport at {at:#x}: {p:?}"));
        }
        Ok(p)
    }

    /// A pose from openmx's numbers: eye, yaw and pitch in degrees, fov. Up is square to forward with no
    /// roll.
    pub fn from_angles(eye: [f32; 3], yaw: f32, pitch: f32, fov: f32) -> Pose {
        let (sy, cy) = yaw.to_radians().sin_cos();
        let (sp, cp) = pitch.to_radians().sin_cos();
        Pose { eye, forward: [sy * cp, sp, cy * cp], up: [-sp * sy, cp, -sp * cy], fov }
    }

    /// `x,y,z,yaw,pitch,fov`, as [`Pose::md10`] writes it.
    pub fn parse_md10(text: &str) -> Result<Pose, String> {
        let v: Vec<f32> = text.split(',').map(|w| w.trim().parse::<f32>().map_err(|e| format!("{text}: {e}"))).collect::<Result<_, _>>()?;
        match v.as_slice() {
            [x, y, z, yaw, pitch, fov] => Ok(Pose::from_angles([*x, *y, *z], *yaw, *pitch, *fov)),
            _ => Err(format!("{text}: not x,y,z,yaw,pitch,fov")),
        }
    }

    /// Whether two poses are the same to `tol` in every number.
    pub fn close(&self, o: &Pose, tol: f32) -> bool {
        let a = self.eye.iter().chain(&self.forward).chain(&self.up).chain(std::iter::once(&self.fov));
        let b = o.eye.iter().chain(&o.forward).chain(&o.up).chain(std::iter::once(&o.fov));
        a.zip(b).all(|(x, y)| (x - y).abs() <= tol)
    }

    fn is_finite(&self) -> bool {
        self.eye.iter().chain(&self.forward).chain(&self.up).all(|c| c.is_finite()) && self.fov.is_finite()
    }

    pub fn yaw(&self) -> f32 {
        self.forward[0].atan2(self.forward[2]).to_degrees()
    }

    pub fn pitch(&self) -> f32 {
        self.forward[1].clamp(-1.0, 1.0).asin().to_degrees()
    }

    /// openmx's `--shot-pose` argument.
    pub fn md10(&self) -> String {
        format!("{},{},{},{},{},{}", self.eye[0], self.eye[1], self.eye[2], self.yaw(), self.pitch(), self.fov)
    }

    /// How far apart two poses are: the eyes' distance, and the angle between the forward vectors in
    /// degrees.
    pub fn drift(&self, o: &Pose) -> (f32, f32) {
        let d = (0..3).map(|k| (self.eye[k] - o.eye[k]).powi(2)).sum::<f32>().sqrt();
        let dot = (0..3).map(|k| self.forward[k] * o.forward[k]).sum::<f32>().clamp(-1.0, 1.0);
        (d, dot.acos().to_degrees())
    }

    /// One line of `poses.txt`: the name, then eye, forward, up and fov, every float in full.
    pub fn line(&self, name: &str) -> String {
        let v: Vec<String> = self.eye.iter().chain(&self.forward).chain(&self.up).map(|c| format!("{c:?}")).collect();
        format!("{name} {} {:?}", v.join(" "), self.fov)
    }

    pub fn parse_line(line: &str) -> Result<(String, Pose), String> {
        let mut it = line.split_whitespace();
        let name = it.next().ok_or("an empty pose line")?.to_owned();
        let v: Vec<f32> = it.map(|w| w.parse::<f32>().map_err(|e| format!("{line}: {e}"))).collect::<Result<_, _>>()?;
        if v.len() != 10 {
            return Err(format!("{line}: {} numbers, not 10", v.len()));
        }
        Ok((name, Pose { eye: [v[0], v[1], v[2]], forward: [v[3], v[4], v[5]], up: [v[6], v[7], v[8]], fov: v[9] }))
    }
}
