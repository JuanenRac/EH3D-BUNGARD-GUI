//! The spindle of the 3D view as the original one is built: a body of revolution (a nose, a collar, a long chrome body and a rear cap) with the
//! cable connector at an angle and the tool in the collet, drawn at its real size from its data sheet. Every part is a faceted solid (a prism
//! or a cone of `FACETS` sides); the facets that face the eye are sorted far to near and shaded by a light that stays at the eye's side, so the
//! chrome has its bright stripe and the round parts look round. Everything that is geometry is plain numbers and is tested on its own.

use super::*;

/// Sides of every round part.
pub(super) const FACETS: usize = 18;

/// One part of revolution along the vertical axis: it runs from `z0` to `z1` (mm above the face of the nose) and goes from radius `r0` to `r1`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Ring {
    pub z0: f64,
    pub z1: f64,
    pub r0: f64,
    pub r1: f64,
    pub color: (u8, u8, u8),
    pub shiny: bool,
}

/// The sizes of the original spindle (a 174 mm long, 33 mm round chrome body with an O20 nose that has an O27 collar), from its drawing, as the parameters of
/// the drawing of the head. They can be overridden in the configuration of the program (`spindle.*` of the settings): see [`Dims`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Dims {
    pub total_mm: f64,
    pub nose_mm: f64,
    pub nose_d: f64,
    pub collar_d: f64,
    pub collar_mm: f64,
    pub body_mm: f64,
    pub body_d: f64,
    /// Height over the axis of the connector at the rear, in mm (the overall height of the drawing).
    pub connector_h: f64,
}

impl Default for Dims {
    fn default() -> Dims {
        Dims {
            total_mm: 174.0,
            nose_mm: 33.0,
            nose_d: 20.0,
            collar_d: 27.0,
            collar_mm: 14.0,
            body_mm: 88.0,
            body_d: 33.0,
            connector_h: 59.0,
        }
    }
}

/// A point in the space of the drawing (X, Y, Z in mm) and a colour.
type Point = (f64, f64, f64);
type Rgb8 = (u8, u8, u8);
/// A part of the connector: the two ends of its axis, the radius at each and its colour.
type Part = (Point, Point, f64, f64, Rgb8);
/// A facet on the screen: its corners and its colour.
type Painted = (Vec<(f32, f32)>, Color32);

const CHROME: (u8, u8, u8) = (200, 204, 210);
const STEEL: (u8, u8, u8) = (150, 156, 164);
const DARK: (u8, u8, u8) = (74, 76, 70);

impl Dims {
    /// The parts of revolution from the face of the nose upwards: the nose, the collar over it, the body and the cap behind it that ends the
    /// length. Height 0 is where the tool comes out of the nose.
    pub fn rings(&self) -> Vec<Ring> {
        let (rn, rc, rb) = (self.nose_d / 2.0, self.collar_d / 2.0, self.body_d / 2.0);
        let nose_top = self.nose_mm;
        let collar_from = (nose_top - self.collar_mm).max(0.0);
        let body_top = nose_top + self.body_mm;
        let rear_top = self.total_mm.max(body_top + 1.0);
        vec![
            Ring {
                z0: 0.0,
                z1: collar_from,
                r0: rn,
                r1: rn,
                color: STEEL,
                shiny: true,
            },
            Ring {
                z0: collar_from,
                z1: nose_top,
                r0: rc,
                r1: rc,
                color: STEEL,
                shiny: true,
            },
            Ring {
                z0: nose_top,
                z1: body_top,
                r0: rb,
                r1: rb,
                color: CHROME,
                shiny: true,
            },
            // the rear cap: a little narrower, a dark ring where the cable leaves
            Ring {
                z0: body_top,
                z1: rear_top,
                r0: rb - 0.5,
                r1: rb - 3.0,
                color: CHROME,
                shiny: true,
            },
        ]
    }

    /// The tool: a thin shaft of the diameter of the drill or the router that comes out of the nose (`out_mm` is how far).
    pub fn tool(&self, diameter_mm: f64, out_mm: f64) -> Ring {
        Ring {
            z0: -out_mm,
            z1: 0.0,
            r0: (diameter_mm / 2.0).max(0.25),
            r1: (diameter_mm / 2.0).max(0.25),
            color: (210, 190, 120),
            shiny: true,
        }
    }

    /// The cable connector: a dark body that leaves the rear of the spindle at an angle (from the axis at the rear to a point `connector_h` over it and a little
    /// to the side), as the first and the last point of its axis with the radius of its two ends, and the round plug that stands at the end of it.
    pub fn connector(&self) -> [Part; 2] {
        let z_base = self.total_mm - 30.0;
        let reach = self.connector_h - self.body_d / 2.0;
        let elbow = (self.body_d / 2.0 + reach * 0.35, 0.0, z_base + 22.0);
        let plug_end = (
            self.body_d / 2.0 + reach * 0.9,
            0.0,
            z_base + 22.0 + reach * 0.9,
        );
        [
            ((0.0, 0.0, z_base), elbow, 9.0, 8.0, DARK),
            (elbow, plug_end, 8.0, 6.0, (110, 112, 108)),
        ]
    }
}

/// A convex facet of the drawing: its corners in space, its colour and whether it is a shiny one.
#[derive(Debug, Clone)]
pub(super) struct Facet {
    pub pts: Vec<(f64, f64, f64)>,
    pub normal: (f64, f64, f64),
    pub color: (u8, u8, u8),
    pub shiny: bool,
}

/// The sides of a frustum along any axis from `a` to `b` with radii `ra` and `rb`, and its two end caps.
pub(super) fn frustum(
    a: (f64, f64, f64),
    b: (f64, f64, f64),
    ra: f64,
    rb: f64,
    color: (u8, u8, u8),
    shiny: bool,
) -> Vec<Facet> {
    let axis = norm(sub(b, a));
    // two directions across the axis
    let helper = if axis.2.abs() < 0.9 {
        (0.0, 0.0, 1.0)
    } else {
        (1.0, 0.0, 0.0)
    };
    let u = norm(cross(axis, helper));
    let v = cross(axis, u);
    let ring = |c: (f64, f64, f64), r: f64, k: usize| -> (f64, f64, f64) {
        let ang = k as f64 * std::f64::consts::TAU / FACETS as f64;
        let d = add(scale(u, ang.cos()), scale(v, ang.sin()));
        add(c, scale(d, r))
    };
    let mut out = Vec::with_capacity(FACETS + 2);
    for k in 0..FACETS {
        let mid = (k as f64 + 0.5) * std::f64::consts::TAU / FACETS as f64;
        let normal = add(scale(u, mid.cos()), scale(v, mid.sin()));
        out.push(Facet {
            pts: vec![
                ring(a, ra, k),
                ring(a, ra, k + 1),
                ring(b, rb, k + 1),
                ring(b, rb, k),
            ],
            normal,
            color,
            shiny,
        });
    }
    for (c, r, n) in [(a, ra, scale(axis, -1.0)), (b, rb, axis)] {
        if r > 0.0 {
            out.push(Facet {
                pts: (0..FACETS).map(|k| ring(c, r, k)).collect(),
                normal: n,
                color,
                shiny: false,
            });
        }
    }
    out
}

/// Every facet of the spindle with the tool in it, standing with the tip of the tool at `tip` (mm in the space of the drawing: X, Y of the table and Z up).
/// `tool_out_mm` is how far the tool comes out of the nose. The connector leaves towards +X.
pub(super) fn spindle_facets(
    dims: &Dims,
    tip: (f64, f64, f64),
    tool_d: f64,
    tool_out_mm: f64,
) -> Vec<Facet> {
    let face_z = tip.2 + tool_out_mm;
    let at = |z: f64| (tip.0, tip.1, face_z + z);
    let mut out = Vec::new();
    let t = dims.tool(tool_d, tool_out_mm);
    out.extend(frustum(at(t.z0), at(t.z1), t.r0, t.r1, t.color, t.shiny));
    for r in dims.rings() {
        out.extend(frustum(at(r.z0), at(r.z1), r.r0, r.r1, r.color, r.shiny));
    }
    for (a, b, ra, rb, color) in dims.connector() {
        let m = |p: (f64, f64, f64)| (tip.0 + p.0, tip.1 + p.1, face_z + p.2);
        out.extend(frustum(m(a), m(b), ra, rb, color, false));
    }
    out
}

fn add(a: (f64, f64, f64), b: (f64, f64, f64)) -> (f64, f64, f64) {
    (a.0 + b.0, a.1 + b.1, a.2 + b.2)
}
fn sub(a: (f64, f64, f64), b: (f64, f64, f64)) -> (f64, f64, f64) {
    (a.0 - b.0, a.1 - b.1, a.2 - b.2)
}
fn scale(a: (f64, f64, f64), k: f64) -> (f64, f64, f64) {
    (a.0 * k, a.1 * k, a.2 * k)
}
fn dot(a: (f64, f64, f64), b: (f64, f64, f64)) -> f64 {
    a.0 * b.0 + a.1 * b.1 + a.2 * b.2
}
fn cross(a: (f64, f64, f64), b: (f64, f64, f64)) -> (f64, f64, f64) {
    (
        a.1 * b.2 - a.2 * b.1,
        a.2 * b.0 - a.0 * b.2,
        a.0 * b.1 - a.1 * b.0,
    )
}
fn norm(a: (f64, f64, f64)) -> (f64, f64, f64) {
    let l = dot(a, a).sqrt();
    if l < 1e-12 {
        (0.0, 0.0, 1.0)
    } else {
        scale(a, 1.0 / l)
    }
}

/// Where the eye is, as a direction in the space of the drawing, and the direction of the right of the screen: from the yaw and the pitch of the view.
pub(super) fn eye_and_right(iso: &Iso) -> ((f64, f64, f64), (f64, f64, f64)) {
    let p = iso.pitch_deg.to_radians();
    let (sn, cs) = iso.yaw_deg.to_radians().sin_cos();
    // in the turned frame the eye is over the corner (+X, +Y); the turn is undone to have it in the frame of the table
    let back = |x: f64, y: f64| (x * cs + y * sn, -x * sn + y * cs);
    let h = std::f64::consts::FRAC_1_SQRT_2;
    let (ex, ey) = back(h * p.cos(), h * p.cos());
    let (rx, ry) = back(h, -h);
    ((ex, ey, p.sin()), (rx, ry, 0.0))
}

/// The colour of a facet under the light that comes from over the left shoulder of the eye; a shiny facet has a bright stripe where it mirrors the light.
pub(super) fn lit(f: &Facet, eye: (f64, f64, f64), right: (f64, f64, f64)) -> Color32 {
    let light = norm(add(add(eye, scale(right, -0.8)), (0.0, 0.0, 0.3)));
    let diffuse = dot(f.normal, light).max(0.0);
    let mut k = 0.42 + 0.58 * diffuse;
    if f.shiny {
        let half = norm(add(light, eye));
        let s = dot(f.normal, half).max(0.0).powf(24.0);
        k += 0.55 * s;
    }
    let c = |v: u8| (v as f64 * k).clamp(0.0, 255.0) as u8;
    Color32::from_rgb(c(f.color.0), c(f.color.1), c(f.color.2))
}

/// The facets that face the eye, far to near, with their corners on the screen and their colour: what the painter draws in this order.
pub(super) fn visible(
    facets: &[Facet],
    iso: &Iso,
    project: &dyn Fn(f64, f64, f64) -> (f32, f32),
) -> Vec<Painted> {
    let (eye, right) = eye_and_right(iso);
    let mut list: Vec<(f64, Painted)> = facets
        .iter()
        .filter(|f| dot(f.normal, eye) > -0.02)
        .map(|f| {
            let n = f.pts.len() as f64;
            let c = f.pts.iter().fold((0.0, 0.0, 0.0), |a, &p| add(a, p));
            let depth = dot(scale(c, 1.0 / n), eye);
            (
                depth,
                (
                    f.pts.iter().map(|&(x, y, z)| project(x, y, z)).collect(),
                    lit(f, eye, right),
                ),
            )
        })
        .collect();
    list.sort_by(|a, b| a.0.total_cmp(&b.0));
    list.into_iter().map(|(_, painted)| painted).collect()
}

impl App {
    /// The spindle with its tool at the tip `tip` of the 3D view, from its data sheet.
    pub(super) fn paint_spindle(
        &self,
        cv: &mut Cv,
        iso: &Iso,
        project: &dyn Fn(f64, f64, f64) -> (f32, f32),
        tip: (f64, f64, f64),
        tool_d: f64,
    ) {
        let facets = spindle_facets(&Dims::default(), tip, tool_d, 9.0);
        for (pts, color) in visible(&facets, iso, project) {
            cv.poly(&pts, color, Stroke::new(0.5 * cv.u, shade(color, 0.72)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_spindle_is_as_long_and_as_round_as_its_data_sheet() {
        let d = Dims::default();
        let rings = d.rings();
        let top = rings.iter().map(|r| r.z1).fold(0.0, f64::max);
        assert!((top - 174.0).abs() < 1e-9);
        // the nose is 20 mm round for 33 mm with a 27 mm collar, the body 33 mm for 88 mm
        assert!((rings[0].r0 * 2.0 - 20.0).abs() < 1e-9);
        assert!((rings[1].r0 * 2.0 - 27.0).abs() < 1e-9);
        assert!((rings[1].z1 - 33.0).abs() < 1e-9);
        assert!((rings[2].r0 * 2.0 - 33.0).abs() < 1e-9);
        assert!((rings[2].z1 - rings[2].z0 - 88.0).abs() < 1e-9);
        // the rings follow one another with no gap
        for w in rings.windows(2) {
            assert!((w[0].z1 - w[1].z0).abs() < 1e-9);
        }
    }

    #[test]
    fn the_connector_rises_to_its_overall_height() {
        let d = Dims::default();
        let c = d.connector();
        let reach = c[1].1 .0;
        assert!(reach > d.body_d / 2.0, "the connector goes out of the body");
        let high = c[1].1 .2 - (d.total_mm - 30.0);
        assert!(high > 20.0, "the connector rises {high}");
    }

    #[test]
    fn a_frustum_has_a_side_for_each_facet_and_two_caps_and_its_normals_point_out() {
        let f = frustum((0.0, 0.0, 0.0), (0.0, 0.0, 10.0), 5.0, 5.0, STEEL, false);
        assert_eq!(f.len(), FACETS + 2);
        for s in f.iter().take(FACETS) {
            // a side's normal is at a right angle to the axis and points away from it
            assert!(s.normal.2.abs() < 1e-9);
            let c = s.pts.iter().fold((0.0, 0.0), |a, p| (a.0 + p.0, a.1 + p.1));
            assert!(dot((c.0, c.1, 0.0), s.normal) > 0.0);
        }
        let angled = frustum((0.0, 0.0, 0.0), (10.0, 0.0, 10.0), 3.0, 2.0, DARK, false);
        assert_eq!(angled.len(), FACETS + 2);
    }

    #[test]
    fn the_whole_spindle_stands_over_its_tip_with_the_tool_in_front_of_the_nose() {
        let facets = spindle_facets(&Dims::default(), (50.0, 40.0, 33.0), 1.0, 9.0);
        let lowest = facets
            .iter()
            .flat_map(|f| f.pts.iter().map(|p| p.2))
            .fold(f64::MAX, f64::min);
        let highest = facets
            .iter()
            .flat_map(|f| f.pts.iter().map(|p| p.2))
            .fold(f64::MIN, f64::max);
        assert!(
            (lowest - 33.0).abs() < 1e-9,
            "the tool tip is at the board: {lowest}"
        );
        assert!(
            (highest - (33.0 + 9.0 + 174.0)).abs() < 1.0 + 60.0,
            "{highest}"
        );
        // the tool is the thinnest part
        assert!(facets.iter().any(|f| f
            .pts
            .iter()
            .any(|p| ((p.0 - 50.0).powi(2) + (p.1 - 40.0).powi(2)).sqrt() < 0.6)));
    }

    #[test]
    fn what_faces_the_eye_is_drawn_far_to_near_in_any_turn_of_the_view() {
        let facets = spindle_facets(&Dims::default(), (0.0, 0.0, 0.0), 1.0, 9.0);
        for yaw in [0.0, 37.0, 90.0, 215.0] {
            let iso = Iso {
                k: 1.0,
                origin: (0.0, 0.0),
                yaw_deg: yaw,
                pivot: (0.0, 0.0),
                pitch_deg: 30.0,
            };
            let project = |x: f64, y: f64, z: f64| {
                let (a, b) = iso.project(x, y, z);
                (a as f32, b as f32)
            };
            let seen = visible(&facets, &iso, &project);
            // about half the sides of a round part face away
            assert!(!seen.is_empty() && seen.len() < facets.len(), "yaw {yaw}");
        }
    }

    #[test]
    fn a_shiny_facet_that_mirrors_the_light_is_brighter_than_one_that_does_not() {
        let iso = Iso {
            k: 1.0,
            origin: (0.0, 0.0),
            yaw_deg: 0.0,
            pivot: (0.0, 0.0),
            pitch_deg: 30.0,
        };
        let (eye, right) = eye_and_right(&iso);
        let mk = |n: (f64, f64, f64)| Facet {
            pts: vec![(0.0, 0.0, 0.0)],
            normal: norm(n),
            color: CHROME,
            shiny: true,
        };
        let toward = lit(&mk(add(eye, scale(right, -0.4))), eye, right);
        let away = lit(&mk(scale(right, 1.0)), eye, right);
        assert!(toward.r() > away.r());
    }
}
