//! The aluminium profile of the axes of the machine: 40x80L, I-type, slot 8 (the one the original machine uses). Its measures are kept in the settings file (the starting
//! values are the ones of its data sheet), and the section of it that the plan view draws and the rails and the beam of the 3D view are built from the same measures.

/// The measures of the profile, in mm unless said otherwise.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProfileDims {
    /// The long side of the section and the short one.
    pub width_mm: f64,
    pub height_mm: f64,
    /// The opening of the slots, and how far the first slot of a long face is from the corner.
    pub slot_mm: f64,
    pub slot_edge_mm: f64,
    /// The two bores of the core: their diameter and the distance between their centres.
    pub bore_mm: f64,
    pub bore_pitch_mm: f64,
    /// What the data sheet says about the profile (not used to draw): the mass in kg/m, the moments of inertia in cm4, the section moduli in cm3 and the area in cm2.
    pub mass_kg_m: f64,
    pub ix_cm4: f64,
    pub iy_cm4: f64,
    pub wx_cm3: f64,
    pub wy_cm3: f64,
    pub area_cm2: f64,
}

impl Default for ProfileDims {
    fn default() -> Self {
        ProfileDims {
            width_mm: 80.0,
            height_mm: 40.0,
            slot_mm: 8.0,
            slot_edge_mm: 12.25,
            bore_mm: 6.8,
            bore_pitch_mm: 40.0,
            mass_kg_m: 3.04,
            ix_cm4: 16.60,
            iy_cm4: 69.54,
            wx_cm3: 8.30,
            wy_cm3: 17.38,
            area_cm2: 11.38,
        }
    }
}

impl ProfileDims {
    /// The line of the settings file: the twelve numbers, separated by commas.
    pub fn to_text(&self) -> String {
        format!(
            "{},{},{},{},{},{},{},{},{},{},{},{}",
            self.width_mm,
            self.height_mm,
            self.slot_mm,
            self.slot_edge_mm,
            self.bore_mm,
            self.bore_pitch_mm,
            self.mass_kg_m,
            self.ix_cm4,
            self.iy_cm4,
            self.wx_cm3,
            self.wy_cm3,
            self.area_cm2
        )
    }

    /// The measures of a line of the settings file; what is not a valid number (or does not make a section that can be drawn) keeps the starting value.
    pub fn from_text(v: &str) -> ProfileDims {
        let mut d = ProfileDims::default();
        let f: Vec<Option<f64>> = v
            .split(',')
            .map(|x| {
                x.trim()
                    .parse::<f64>()
                    .ok()
                    .filter(|n| n.is_finite() && *n > 0.0)
            })
            .collect();
        let slots = [
            &mut d.width_mm,
            &mut d.height_mm,
            &mut d.slot_mm,
            &mut d.slot_edge_mm,
            &mut d.bore_mm,
            &mut d.bore_pitch_mm,
            &mut d.mass_kg_m,
            &mut d.ix_cm4,
            &mut d.iy_cm4,
            &mut d.wx_cm3,
            &mut d.wy_cm3,
            &mut d.area_cm2,
        ];
        for (slot, n) in slots.into_iter().zip(f) {
            if let Some(n) = n {
                *slot = n;
            }
        }
        // the slots of a long face and the bores must fit in the section, and the long side is the long one
        let sane = d.width_mm >= d.height_mm
            && d.height_mm >= 10.0
            && d.slot_mm < d.height_mm / 2.0
            && 2.0 * (d.slot_edge_mm + d.slot_mm) < d.width_mm
            && d.bore_pitch_mm + d.bore_mm < d.width_mm
            && d.bore_mm < d.height_mm / 2.0;
        if sane {
            d
        } else {
            ProfileDims::default()
        }
    }

    /// The section drawn from these measures, centred on (0, 0), the long side along x.
    pub fn section(&self) -> Section {
        let (hw, hh) = (self.width_mm / 2.0, self.height_mm / 2.0);
        let s = self.slot_mm / 2.0;
        // a T-slot: the opening in the face and the wider room under it (a depth of the opening of 4.5 mm and of the room of 6 mm, wider than the opening by 6 mm)
        let (open_depth, room_depth, room_half) = (4.5, 6.0, s + 3.0);
        let t_slot = |cx: f64, cy: f64, dir: (f64, f64)| -> Vec<(f64, f64)> {
            // `dir` points from the face to the inside; the slot is built facing down from the top face and turned
            let local = [
                (-s, 0.0),
                (s, 0.0),
                (s, -open_depth),
                (room_half, -open_depth),
                (room_half, -open_depth - room_depth),
                (-room_half, -open_depth - room_depth),
                (-room_half, -open_depth),
                (-s, -open_depth),
            ];
            // the top face has dir (0, -1): its local frame is the plane; the others turn it
            let (ux, uy) = (dir.0, dir.1);
            local
                .iter()
                .map(|&(lx, ly)| {
                    // the local x axis is perpendicular to `dir`, the local y axis points against it
                    (cx + lx * -uy + ly * -ux, cy + lx * ux + ly * -uy)
                })
                .collect()
        };
        let first = -hw + self.slot_edge_mm + s;
        let slots = vec![
            t_slot(first, hh, (0.0, -1.0)),
            t_slot(-first, hh, (0.0, -1.0)),
            t_slot(first, -hh, (0.0, 1.0)),
            t_slot(-first, -hh, (0.0, 1.0)),
            t_slot(-hw, 0.0, (1.0, 0.0)),
            t_slot(hw, 0.0, (-1.0, 0.0)),
        ];
        let b = self.bore_pitch_mm / 2.0;
        // the hollow of the core between the two bores
        let c = (b - self.bore_mm / 2.0 - 3.0).max(2.0);
        let cavity = vec![
            (-c, -hh * 0.45),
            (-c * 0.55, -hh * 0.62),
            (c * 0.55, -hh * 0.62),
            (c, -hh * 0.45),
            (c, hh * 0.45),
            (c * 0.55, hh * 0.62),
            (-c * 0.55, hh * 0.62),
            (-c, hh * 0.45),
        ];
        Section {
            outline: vec![(-hw, -hh), (hw, -hh), (hw, hh), (-hw, hh)],
            slots,
            bores: vec![((-b, 0.0), self.bore_mm), ((b, 0.0), self.bore_mm)],
            cavity,
        }
    }
}

/// The silhouette of the section: the outline, the T-slots, the two bores and the hollow of the core.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub outline: Vec<(f64, f64)>,
    pub slots: Vec<Vec<(f64, f64)>>,
    /// The centre and the diameter of each bore.
    pub bores: Vec<((f64, f64), f64)>,
    pub cavity: Vec<(f64, f64)>,
}

impl Section {
    /// The same section turned a quarter, to show it from the side of the other axis (the long side then runs along y).
    pub fn turned(&self) -> Section {
        let r = |p: &(f64, f64)| (-p.1, p.0);
        Section {
            outline: self.outline.iter().map(r).collect(),
            slots: self
                .slots
                .iter()
                .map(|s| s.iter().map(r).collect())
                .collect(),
            bores: self.bores.iter().map(|(c, d)| (r(c), *d)).collect(),
            cavity: self.cavity.iter().map(r).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(p: &[(f64, f64)]) -> (f64, f64, f64, f64) {
        p.iter()
            .fold((f64::MAX, f64::MAX, f64::MIN, f64::MIN), |b, &(x, y)| {
                (b.0.min(x), b.1.min(y), b.2.max(x), b.3.max(y))
            })
    }

    #[test]
    fn the_starting_values_are_the_ones_of_the_data_sheet() {
        let d = ProfileDims::default();
        assert_eq!((d.width_mm, d.height_mm, d.slot_mm), (80.0, 40.0, 8.0));
        assert_eq!(
            (d.slot_edge_mm, d.bore_mm, d.bore_pitch_mm),
            (12.25, 6.8, 40.0)
        );
        assert_eq!((d.mass_kg_m, d.ix_cm4, d.iy_cm4), (3.04, 16.60, 69.54));
        assert_eq!((d.wx_cm3, d.wy_cm3, d.area_cm2), (8.30, 17.38, 11.38));
    }

    #[test]
    fn the_measures_go_to_the_settings_line_and_come_back_and_a_bad_line_keeps_the_starting_ones() {
        let d = ProfileDims {
            width_mm: 90.0,
            height_mm: 45.0,
            slot_mm: 10.0,
            ..ProfileDims::default()
        };
        assert_eq!(ProfileDims::from_text(&d.to_text()), d);
        assert_eq!(
            ProfileDims::from_text(&ProfileDims::default().to_text()),
            ProfileDims::default()
        );
        assert_eq!(ProfileDims::from_text("x,y"), ProfileDims::default());
        // a section that could not be drawn (the short side longer than the long one) is not taken
        assert_eq!(
            ProfileDims::from_text("20,40,8,12.25,6.8,40"),
            ProfileDims::default()
        );
        // a part that is missing keeps its starting value
        assert_eq!(ProfileDims::from_text("100").width_mm, 100.0);
        assert_eq!(ProfileDims::from_text("100").height_mm, 40.0);
    }

    #[test]
    fn the_section_is_as_wide_and_as_tall_as_the_profile_with_its_slots_and_bores_where_the_sheet_puts_them(
    ) {
        let d = ProfileDims::default();
        let s = d.section();
        let (x0, y0, x1, y1) = bounds(&s.outline);
        assert_eq!((x1 - x0, y1 - y0), (80.0, 40.0));
        // four slots in the long faces (two each), one in each short face
        assert_eq!(s.slots.len(), 6);
        // the first slot of the top face opens 12.25 mm from the corner and is 8 mm wide
        let top: Vec<&Vec<(f64, f64)>> = s
            .slots
            .iter()
            .filter(|p| {
                p.iter().all(|q| q.1 <= 20.0 + 1e-9) && p.iter().any(|q| (q.1 - 20.0).abs() < 1e-9)
            })
            .collect();
        assert_eq!(top.len(), 2);
        let at_face: Vec<f64> = top[0]
            .iter()
            .filter(|q| (q.1 - 20.0).abs() < 1e-9)
            .map(|q| q.0)
            .collect();
        assert_eq!(at_face.len(), 2);
        let lo = at_face.iter().cloned().fold(f64::MAX, f64::min);
        let hi = at_face.iter().cloned().fold(f64::MIN, f64::max);
        assert!((hi - lo - 8.0).abs() < 1e-9);
        assert!(
            ((lo - -40.0).abs() - 12.25).abs() < 1e-9 || ((hi - 40.0).abs() - 12.25).abs() < 1e-9
        );
        // every slot stays inside the section
        for p in &s.slots {
            let (a, b, c, e) = bounds(p);
            assert!(a >= x0 - 1e-9 && c <= x1 + 1e-9 && b >= y0 - 1e-9 && e <= y1 + 1e-9);
        }
        // the two bores: 6.8 mm, 40 mm between centres, on the middle line
        assert_eq!(s.bores.len(), 2);
        assert!(s.bores.iter().all(|(c, dia)| c.1 == 0.0 && *dia == 6.8));
        assert!((s.bores[1].0 .0 - s.bores[0].0 .0 - 40.0).abs() < 1e-9);
        // the hollow of the core lies between the bores
        let (a, _, c, _) = bounds(&s.cavity);
        assert!(a > s.bores[0].0 .0 && c < s.bores[1].0 .0);
    }

    #[test]
    fn turned_for_the_other_axis_the_long_side_runs_the_other_way() {
        let s = ProfileDims::default().section();
        let t = s.turned();
        let (x0, y0, x1, y1) = bounds(&t.outline);
        assert_eq!((x1 - x0, y1 - y0), (40.0, 80.0));
        assert_eq!(t.slots.len(), s.slots.len());
        assert!((t.bores[0].0 .1 - -20.0).abs() < 1e-9 || (t.bores[0].0 .1 - 20.0).abs() < 1e-9);
    }
}
