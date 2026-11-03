//! The frame of the machine: where the aluminium profiles of the X and Y axes are and how big they are. The 3D view builds its rails and its beam from it and the plan
//! view draws the same profiles (their strips and their section), so the two agree. The measures of the profile are in the settings file.

use super::*;

/// The places and sizes of the profiles, in mm, in the frame of the 3D view (X along the rails, Y across them, the machine's Y towards the operator).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Frame {
    /// The plate goes from `fx0` to `fx1` along the rails and from `fy0` to `fy1` across them.
    pub fx0: f64,
    pub fx1: f64,
    pub fy0: f64,
    pub fy1: f64,
    /// The rails of the X axis lie flat: this wide across and this tall.
    pub rail_across: f64,
    pub rail_tall: f64,
    /// The beam of the Y axis stands on its edge: this wide along the rails and this tall.
    pub beam_across: f64,
    pub beam_tall: f64,
}

/// A profile as the plan view draws it: a strip of the table, in mm of the files (Y away from the operator).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct AxisStrip {
    /// `'X'` for a rail of the X axis, `'Y'` for the beam of the Y axis.
    pub axis: char,
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
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

    /// The profiles of the plan view: the two rails and the beam with the head at `hx` (the table is `table_depth` deep).
    pub fn strips(&self, table_depth: f64, hx: f64) -> [AxisStrip; 3] {
        let td = table_depth;
        let w = self.rail_across;
        let x0 = self.fx0 - RAIL_OVERHANG;
        [
            AxisStrip {
                axis: 'X',
                x0,
                y0: td - self.fy0,
                x1: x0 + self.rail_len(),
                y1: td - self.fy0 + w,
            },
            AxisStrip {
                axis: 'X',
                x0,
                y0: td - self.fy1 - w,
                x1: x0 + self.rail_len(),
                y1: td - self.fy1,
            },
            AxisStrip {
                axis: 'Y',
                x0: hx - BEAM_BEHIND,
                y0: td - self.fy1 - w,
                x1: hx - BEAM_BEHIND + self.beam_across,
                y1: td - self.fy1 - w + self.beam_len(),
            },
        ]
    }
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
            rail_across: p.width_mm,
            rail_tall: p.height_mm,
            beam_across: p.height_mm,
            beam_tall: p.width_mm,
        }
    }

    /// Draws the aluminium profiles of the axes in the plan view: the strips of the rails and of the beam where the 3D view has them, and the section of each axis at its end.
    pub(super) fn paint_plan_axes(
        &self,
        cv: &mut Cv,
        screen: &dyn Fn(f64, f64) -> (f32, f32),
        hx: f64,
    ) {
        let t = cv.t.clone();
        let frame = self.machine_frame();
        let strips = frame.strips(self.table.depth_mm, hx);
        let fill = alpha(col(t.dim), 46);
        let line = alpha(col(t.dim), 150);
        for s in strips {
            let (a, b) = screen(s.x0, s.y1);
            let (c, d) = screen(s.x1, s.y0);
            let r = cv.rect(a, b, c - a, d - b);
            cv.p.rect_filled(r, 2.0 * cv.u, fill);
            cv.p.rect_stroke(
                r,
                2.0 * cv.u,
                Stroke::new(1.0 * cv.u, line),
                StrokeKind::Inside,
            );
        }
        // the section of the profile of each axis, beside the end of the axis: the rail as it lies, the beam on its edge
        let section = self.settings.profile.section();
        let rail = &strips[0];
        let beam = &strips[2];
        let turned = section.turned();
        let spots = [
            (
                &section,
                rail.x0 - frame.rail_across,
                (rail.y0 + rail.y1) / 2.0,
            ),
            (
                &turned,
                (beam.x0 + beam.x1) / 2.0,
                beam.y1 + frame.beam_tall / 2.0 + 6.0,
            ),
        ];
        for (sec, cx, cy) in spots {
            let map = |p: &(f64, f64)| screen(cx + p.0, cy + p.1);
            let pts = |v: &[(f64, f64)]| v.iter().map(map).collect::<Vec<_>>();
            cv.poly(
                &pts(&sec.outline),
                alpha(col(t.panel_raised), 255),
                Stroke::new(1.2 * cv.u, col(t.dim)),
            );
            for s in &sec.slots {
                cv.poly(
                    &pts(s),
                    Color32::TRANSPARENT,
                    Stroke::new(1.0 * cv.u, col(t.text)),
                );
            }
            cv.poly(
                &pts(&sec.cavity),
                alpha(Color32::BLACK, 120),
                Stroke::new(0.8 * cv.u, line),
            );
            for ((bx, by), dia) in &sec.bores {
                let (px, py) = map(&(*bx, *by));
                let (qx, _) = screen(cx + bx + dia / 2.0, cy + by);
                cv.p.circle_stroke(
                    cv.pt(px, py),
                    ((qx - px).abs() * cv.u).max(1.0),
                    Stroke::new(1.0 * cv.u, line),
                );
            }
        }
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
        assert_eq!((f.rail_across, f.rail_tall), (80.0, 40.0));
        assert_eq!((f.beam_across, f.beam_tall), (40.0, 80.0));
        a.settings.profile = crate::profile::ProfileDims {
            width_mm: 90.0,
            height_mm: 45.0,
            ..crate::profile::ProfileDims::default()
        };
        let f = a.machine_frame();
        assert_eq!(
            (f.rail_across, f.rail_tall, f.beam_across, f.beam_tall),
            (90.0, 45.0, 45.0, 90.0)
        );
    }

    #[test]
    fn the_strips_of_the_plan_are_the_rails_and_the_beam_of_the_3d_view() {
        let a = app();
        let f = a.machine_frame();
        let td = a.table.depth_mm;
        let s = f.strips(td, 100.0);
        // two rails as long as the 3D rails and as wide as the profile, one each side of the plate
        for r in &s[..2] {
            assert_eq!(r.axis, 'X');
            assert!(((r.x1 - r.x0) - f.rail_len()).abs() < 1e-9);
            assert!(((r.y1 - r.y0) - f.rail_across).abs() < 1e-9);
        }
        // the plate is between them (3D: the back rail ends where the plate begins, the front one where it ends)
        assert!((s[0].y0 - (td - f.fy0)).abs() < 1e-9);
        assert!((s[1].y1 - (td - f.fy1)).abs() < 1e-9);
        assert!(s[1].y1 <= s[0].y0);
        // the beam goes across both rails, as wide as the short side of the profile, behind the head
        let b = &s[2];
        assert_eq!(b.axis, 'Y');
        assert!(((b.x1 - b.x0) - f.beam_across).abs() < 1e-9);
        assert!(((b.y1 - b.y0) - f.beam_len()).abs() < 1e-9);
        assert!((b.x0 - (100.0 - BEAM_BEHIND)).abs() < 1e-9);
        assert!(b.y0 <= s[1].y0 && b.y1 >= s[0].y1 - 1e-9);
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
        assert!((span(&xs) - f.rail_across).abs() < 1e-9);
        assert!((span(&ys) - f.rail_tall).abs() < 1e-9);
        let turned = sec.turned();
        let tx: Vec<f64> = turned.outline.iter().map(|p| p.0).collect();
        let ty: Vec<f64> = turned.outline.iter().map(|p| p.1).collect();
        assert!((span(&tx) - f.beam_across).abs() < 1e-9);
        assert!((span(&ty) - f.beam_tall).abs() < 1e-9);
    }
}
