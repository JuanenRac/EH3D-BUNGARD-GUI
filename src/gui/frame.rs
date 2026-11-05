//! The frame of the machine: where the aluminium profiles of the X and Y axes are and how big they are. The 3D view builds its rails and its beam from it, with the
//! profile standing on its long side. The plan view does not draw them. The measures of the profile are in the settings file.

use super::*;

/// The places and sizes of the profiles, in mm, in the frame of the 3D view (X along the rails, Y across them, the machine's Y towards the operator).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Frame {
    /// The plate goes from `fx0` to `fx1` along the rails and from `fy0` to `fy1` across them.
    pub fx0: f64,
    pub fx1: f64,
    pub fy0: f64,
    pub fy1: f64,
    /// The rails of the X axis stand on their long side: this wide across (the short side of the section) and this tall (the long side).
    pub rail_across: f64,
    pub rail_tall: f64,
    /// The beam of the Y axis stands as the rails do: its long side (80 mm) up, this wide along the rails (the short side) and this tall.
    pub beam_across: f64,
    pub beam_tall: f64,
}

/// How far the rails go past the plate at each end (the end caps are outside that).
pub(crate) const RAIL_OVERHANG: f64 = 34.0;
/// How far behind the head the beam begins, along the rails.
pub(crate) const BEAM_BEHIND: f64 = 64.0;

impl Frame {
    /// The length of a rail, end to end.
    pub fn rail_len(&self) -> f64 {
        (self.fx1 - self.fx0) + 2.0 * RAIL_OVERHANG
    }

    /// The length of the beam across the machine, from the outside of one rail to the outside of the other.
    pub fn beam_len(&self) -> f64 {
        (self.fy1 - self.fy0) + 2.0 * self.rail_across
    }
}

/// The direction a profile runs in: along the rails (X) or across them (Y).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Axis {
    X,
    Y,
}

type P3 = (f64, f64, f64);

/// The end of a profile as the 3D view draws it: the real section (the outline, the T-slots, the two bores and the hollow of the core) put at the end of the bar.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Cap {
    pub outline: Vec<P3>,
    pub slots: Vec<Vec<P3>>,
    pub bores: Vec<Vec<P3>>,
    pub cavity: Vec<P3>,
}

/// A profile as a bar in 3D: the section at its end and the dark strips of the slots along its visible faces.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Extruded {
    pub cap: Cap,
    /// Boxes (x, y, z, dx, dy, dz) a hair high over the face, where the mouths of the slots are.
    pub strips: Vec<(f64, f64, f64, f64, f64, f64)>,
}

/// The bar of `len` mm that starts at `start` (its corner of least x, y and z) and runs along `axis`, with the section of `dims` standing on its long side (its short side across,
/// its long side up).
/// The section is the one the plan view draws; the cap is at the far end, the one the 3D view sees.
pub(crate) fn extrude(
    dims: &crate::profile::ProfileDims,
    axis: Axis,
    start: P3,
    len: f64,
) -> Extruded {
    let sec = dims.section();
    // the section is drawn in (long side, short side); the bar stands it up: the long side is vertical and the short one lies across
    let (w, h) = (dims.width_mm, dims.height_mm);
    let (u0, v0) = match axis {
        Axis::X => (start.1 + h / 2.0, start.2 + w / 2.0),
        Axis::Y => (start.0 + h / 2.0, start.2 + w / 2.0),
    };
    let a_end = match axis {
        Axis::X => start.0 + len,
        Axis::Y => start.1 + len,
    };
    let at = |p: &(f64, f64)| -> P3 {
        match axis {
            Axis::X => (a_end, u0 + p.1, v0 + p.0),
            Axis::Y => (u0 + p.1, a_end, v0 + p.0),
        }
    };
    let ring = |c: (f64, f64), d: f64| -> Vec<P3> {
        (0..16)
            .map(|i| {
                let t = i as f64 * std::f64::consts::TAU / 16.0;
                at(&(c.0 + d / 2.0 * t.cos(), c.1 + d / 2.0 * t.sin()))
            })
            .collect()
    };
    let cap = Cap {
        outline: sec.outline.iter().map(at).collect(),
        slots: sec
            .slots
            .iter()
            .map(|s| s.iter().map(at).collect())
            .collect(),
        bores: sec.bores.iter().map(|(c, d)| ring(*c, *d)).collect(),
        cavity: sec.cavity.iter().map(at).collect(),
    };
    // the mouths of the slots on the visible long side (the side face) and on the top face (the short side, one slot in the middle)
    let s = dims.slot_mm / 2.0;
    let first = -w / 2.0 + dims.slot_edge_mm + s;
    let hair = 0.4;
    let bx = |cu: f64, cu_len: f64, cv: f64, cv_len: f64| -> (f64, f64, f64, f64, f64, f64) {
        match axis {
            Axis::X => (start.0, cu, cv, len, cu_len, cv_len),
            Axis::Y => (cu, start.1, cv, cu_len, len, cv_len),
        }
    };
    let mut strips = Vec::new();
    for c in [first, -first] {
        strips.push(bx(u0 + h / 2.0, hair, v0 + c - s, 2.0 * s));
    }
    strips.push(bx(u0 - s, 2.0 * s, start.2 + w, hair));
    Extruded { cap, strips }
}

impl App {
    /// The frame of the machine in use, from its model, the usable table and the measures of the profile in the settings.
    pub(crate) fn machine_frame(&self) -> Frame {
        let (tw, td) = (self.table.width_mm, self.table.depth_mm);
        let profile = match machines::Model::from_name(&self.settings.model) {
            Some(i) => machines::models()[i].profile(),
            None => machines::for_table(&self.table),
        };
        let (wx, wy) = (profile.work_mm.0.max(tw), profile.work_mm.1.max(td));
        let (px_m, py_m) = profile.plate_margins((tw, td));
        let p = self.settings.profile;
        Frame {
            fx0: -px_m,
            fx1: wx + px_m,
            fy0: -py_m,
            fy1: wy + py_m,
            rail_across: p.height_mm,
            rail_tall: p.width_mm,
            beam_across: p.height_mm,
            beam_tall: p.width_mm,
        }
    }
}

/// Paints the section at the end of a bar: the outline in the colour of the metal, and the slots, the bores and the hollow of the core dark.
pub(super) fn paint_cap(
    cv: &Cv,
    p: &dyn Fn(f64, f64, f64) -> (f32, f32),
    cap: &Cap,
    metal: Color32,
    dark: Color32,
    edge: Color32,
) {
    let pts = |v: &[P3]| v.iter().map(|q| p(q.0, q.1, q.2)).collect::<Vec<_>>();
    cv.poly(&pts(&cap.outline), metal, Stroke::new(1.0 * cv.u, edge));
    for s in &cap.slots {
        cv.poly(&pts(s), dark, Stroke::new(0.6 * cv.u, edge));
    }
    cv.poly(&pts(&cap.cavity), dark, Stroke::new(0.6 * cv.u, edge));
    for b in &cap.bores {
        cv.poly(&pts(b), dark, Stroke::new(0.6 * cv.u, edge));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Options;

    fn app() -> App {
        App::new(&Options {
            data: Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata")),
            screen: Some(Screen::Viewer),
            ..Default::default()
        })
    }

    #[test]
    fn the_profiles_of_the_frame_have_the_measures_of_the_settings() {
        let mut a = app();
        let f = a.machine_frame();
        assert_eq!((f.rail_across, f.rail_tall), (40.0, 80.0));
        // both axes carry the profile standing on its 80 mm side: 40 mm wide and 80 mm high
        assert_eq!((f.beam_across, f.beam_tall), (40.0, 80.0));
        a.settings.profile = crate::profile::ProfileDims {
            width_mm: 90.0,
            height_mm: 45.0,
            ..crate::profile::ProfileDims::default()
        };
        let f = a.machine_frame();
        assert_eq!(
            (f.rail_across, f.rail_tall, f.beam_across, f.beam_tall),
            (45.0, 90.0, 45.0, 90.0)
        );
    }

    #[test]
    fn the_section_and_the_frame_come_from_the_same_measures() {
        let a = app();
        let f = a.machine_frame();
        let sec = a.settings.profile.section();
        let xs: Vec<f64> = sec.outline.iter().map(|p| p.0).collect();
        let ys: Vec<f64> = sec.outline.iter().map(|p| p.1).collect();
        let span = |v: &[f64]| {
            v.iter().cloned().fold(f64::MIN, f64::max) - v.iter().cloned().fold(f64::MAX, f64::min)
        };
        // the section is drawn 80 wide and 40 high; the bars stand it up: the 80 mm side is the height and the 40 mm one lies across
        assert!((span(&xs) - f.rail_tall).abs() < 1e-9);
        assert!((span(&ys) - f.rail_across).abs() < 1e-9);
        assert!((span(&xs) - f.beam_tall).abs() < 1e-9);
        assert!((span(&ys) - f.beam_across).abs() < 1e-9);
    }

    fn spans(v: &[P3]) -> ((f64, f64), (f64, f64), (f64, f64)) {
        let r = |f: fn(&P3) -> f64| {
            (
                v.iter().map(f).fold(f64::MAX, f64::min),
                v.iter().map(f).fold(f64::MIN, f64::max),
            )
        };
        (r(|q| q.0), r(|q| q.1), r(|q| q.2))
    }

    #[test]
    fn the_rail_of_the_3d_view_is_the_section_standing_40_across_and_80_high() {
        let dims = crate::profile::ProfileDims::default();
        let sec = dims.section();
        let start = (10.0, 200.0, 0.0);
        let e = extrude(&dims, Axis::X, start, 500.0);
        let (x, y, z) = spans(&e.cap.outline);
        // the cap is at the far end of the bar, 40 mm across (y) and 80 mm up (z)
        assert_eq!(x, (510.0, 510.0));
        assert!(((y.1 - y.0) - 40.0).abs() < 1e-9 && ((z.1 - z.0) - 80.0).abs() < 1e-9);
        assert!((y.0 - 200.0).abs() < 1e-9 && (z.0 - 0.0).abs() < 1e-9);
        // it is the same polygon as the section, turned up point by point: the outline, the six slots, the two bores and the hollow of the core
        assert_eq!(e.cap.outline.len(), sec.outline.len());
        for (q, p) in e.cap.outline.iter().zip(&sec.outline) {
            assert!((q.1 - (220.0 + p.1)).abs() < 1e-9 && (q.2 - (40.0 + p.0)).abs() < 1e-9);
        }
        assert_eq!(e.cap.slots.len(), 6);
        assert_eq!(e.cap.slots.len(), sec.slots.len());
        for (cs, ss) in e.cap.slots.iter().zip(&sec.slots) {
            assert_eq!(cs.len(), ss.len());
        }
        assert_eq!(e.cap.bores.len(), 2);
        assert_eq!(e.cap.cavity.len(), sec.cavity.len());
    }

    #[test]
    fn the_beam_is_the_same_profile_standing_the_same_way_40_wide_and_80_high() {
        let dims = crate::profile::ProfileDims::default();
        let e = extrude(&dims, Axis::Y, (50.0, -100.0, 62.0), 700.0);
        let (x, y, z) = spans(&e.cap.outline);
        assert_eq!(y, (600.0, 600.0));
        assert!(
            ((x.1 - x.0) - 40.0).abs() < 1e-9,
            "40 mm along the rails: {x:?}"
        );
        assert!(((z.1 - z.0) - 80.0).abs() < 1e-9, "80 mm high: {z:?}");
        assert!((z.0 - 62.0).abs() < 1e-9);
        // the same section as the rail's, not another one
        let r = extrude(&dims, Axis::X, (0.0, 0.0, 0.0), 1.0);
        assert_eq!(e.cap.slots.len(), r.cap.slots.len());
        assert_eq!(e.cap.bores.len(), r.cap.bores.len());
    }

    #[test]
    fn the_mouths_of_the_slots_are_dark_strips_along_the_top_and_the_visible_side_and_the_bar_is_not_a_tube(
    ) {
        let dims = crate::profile::ProfileDims::default();
        let e = extrude(&dims, Axis::X, (0.0, 0.0, 0.0), 300.0);
        // two on the long side that is seen and one on the top face
        assert_eq!(e.strips.len(), 3);
        for s in &e.strips {
            assert_eq!(s.0, 0.0);
            assert_eq!(s.3, 300.0, "they run the whole length of the bar");
        }
        // the slot is 8 mm wide
        assert!(e
            .strips
            .iter()
            .all(|s| (s.4 - 8.0).abs() < 1e-9 || (s.5 - 8.0).abs() < 1e-9));
        // the bores and the hollow are in the cap, so the bar has a core and is not a hollow rectangle: the hollow is between the bores
        let (_, _, z) = spans(&e.cap.cavity);
        let (_, _, b0) = spans(&e.cap.bores[0]);
        let (_, _, b1) = spans(&e.cap.bores[1]);
        assert!(z.0 > b0.1 && z.1 < b1.0, "{z:?} {b0:?} {b1:?}");
        for b in &e.cap.bores {
            let (_, by, _) = spans(b);
            assert!(((by.1 - by.0) - 6.8).abs() < 0.2, "the bores are 6.8 mm");
        }
    }

    #[test]
    fn the_axes_of_the_frame_carry_the_profile_with_the_same_numbers_the_3d_bars_are_made_from() {
        let a = app();
        let f = a.machine_frame();
        let d = a.settings.profile;
        assert_eq!((f.rail_across, f.rail_tall), (d.height_mm, d.width_mm));
        assert_eq!((f.beam_across, f.beam_tall), (d.height_mm, d.width_mm));
        assert_eq!((f.rail_across, f.rail_tall), (40.0, 80.0));
    }
}
