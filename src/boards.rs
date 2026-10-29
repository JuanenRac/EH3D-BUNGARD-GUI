//! A library of the copper-clad base materials that are bought for a machine like this one: who makes them, what they are made of, the
//! thicknesses, the copper, the standard formats, whether they have one or two copper sides. The values are the nominal or typical ones
//! of the makers' and the sellers' published lists, to choose a starting point for the board and to show it; the exact figures of a
//! batch are the datasheet's.
//!
//! Sources of the nominal values: the formats and the thicknesses of the photosensitive boards of Bungard (ordering codes with the
//! sizes) as the sellers Rapid Electronics, Buerklin and Conrad list them, the plain boards of Fortex and CIF as RS lists them, the
//! RF laminates of Rogers, Taconic and Isola from their datasheets, and IPC-4101 for the base materials.

/// What the insulating core is made of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Core {
    /// Paper and phenolic resin (FR-2).
    Fr2,
    /// Paper and epoxy (FR-3).
    Fr3,
    /// Paper core with woven glass faces and epoxy (CEM-1).
    Cem1,
    /// Non-woven glass core with woven glass faces and epoxy (CEM-3).
    Cem3,
    /// Woven glass and epoxy (FR-4 and its high-Tg variants).
    Fr4,
    /// Glass or ceramic filled hydrocarbon (the Rogers RO4000 series and the like).
    Hydrocarbon,
    /// PTFE (the RT/duroid and TLY series).
    Ptfe,
    /// Polyimide on glass.
    Polyimide,
    /// A metal core: aluminium with a thin dielectric (LED boards).
    Aluminium,
    /// A flexible polyimide film.
    Flex,
}

impl Core {
    pub fn name(self) -> &'static str {
        match self {
            Core::Fr2 => "FR-2",
            Core::Fr3 => "FR-3",
            Core::Cem1 => "CEM-1",
            Core::Cem3 => "CEM-3",
            Core::Fr4 => "FR-4",
            Core::Hydrocarbon => "Hydrocarbon",
            Core::Ptfe => "PTFE",
            Core::Polyimide => "Polyimide",
            Core::Aluminium => "Aluminium",
            Core::Flex => "Flex",
        }
    }
}

/// One base material of the library.
#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub maker: &'static str,
    pub name: &'static str,
    pub core: Core,
    /// The thicknesses of the core with its copper, in mm.
    pub thickness_mm: &'static [f64],
    /// The copper that the material comes with, in micrometres (18 is 1/2 oz, 35 is 1 oz, 70 is 2 oz, 105 is 3 oz).
    pub copper_um: &'static [f64],
    /// Copper on one side only, on both sides, or either.
    pub sides: Sides,
    /// The standard formats (width, depth) in mm; empty when it is sold by the panel or cut to size.
    pub formats_mm: &'static [(f64, f64)],
    /// The dielectric constant and the loss tangent (a typical value, where the maker gives one).
    pub dk: Option<f64>,
    pub df: Option<f64>,
    /// The glass transition temperature in degrees Celsius.
    pub tg_c: Option<f64>,
    /// Coated with a photosensitive resist, ready to be exposed.
    pub photoresist: bool,
    pub note: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sides {
    One,
    Two,
    Either,
}

const STD_BUNGARD: &[(f64, f64)] = &[
    (100.0, 60.0),
    (100.0, 75.0),
    (160.0, 100.0),
    (200.0, 150.0),
    (250.0, 250.0),
    (300.0, 210.0),
];

const EUROCARD: &[(f64, f64)] = &[(100.0, 160.0), (160.0, 233.4), (233.4, 160.0)];
const SHEETS: &[(f64, f64)] = &[
    (200.0, 300.0),
    (300.0, 600.0),
    (305.0, 457.0),
    (457.0, 610.0),
];
const RF_PANEL: &[(f64, f64)] = &[(305.0, 457.0), (457.0, 610.0)];

/// Every material of the library.
pub const MATERIALS: &[Material] = &[
    // ---- Bungard: the boards for the machines of the same maker, with and without photosensitive resist
    Material {
        maker: "Bungard",
        name: "FR4 epoxy, positive photoresist",
        core: Core::Fr4,
        thickness_mm: &[0.5, 1.0, 1.5],
        copper_um: &[35.0, 70.0],
        sides: Sides::Either,
        formats_mm: STD_BUNGARD,
        dk: Some(4.5),
        df: None,
        tg_c: Some(130.0),
        photoresist: true,
        note: "Base to IPC-4101; resist sensitive to 350-450 nm, resolution better than 30 um",
    },
    Material {
        maker: "Bungard",
        name: "FR4 epoxy, bare copper",
        core: Core::Fr4,
        thickness_mm: &[0.2, 0.5, 1.0, 1.5, 2.0],
        copper_um: &[18.0, 35.0, 70.0],
        sides: Sides::Either,
        formats_mm: &[
            (150.0, 200.0),
            (150.0, 250.0),
            (200.0, 250.0),
            (210.0, 300.0),
            (160.0, 233.4),
        ],
        dk: Some(4.5),
        df: None,
        tg_c: Some(130.0),
        photoresist: false,
        note: "Carrier 1.55 +/- 0.08 mm, copper +/- 5 um at 35 um",
    },
    Material {
        maker: "Bungard",
        name: "Laminated paper (FR2), photoresist",
        core: Core::Fr2,
        thickness_mm: &[1.5],
        copper_um: &[35.0],
        sides: Sides::One,
        formats_mm: STD_BUNGARD,
        dk: Some(4.2),
        df: None,
        tg_c: Some(105.0),
        photoresist: true,
        note: "Cheap single-sided boards for simple circuits",
    },
    Material {
        maker: "Bungard",
        name: "FR2 / FR3 / CEM-1 sheets",
        core: Core::Cem1,
        thickness_mm: &[1.5],
        copper_um: &[35.0, 70.0],
        sides: Sides::One,
        formats_mm: &[(510.0, 1150.0)],
        dk: Some(4.4),
        df: None,
        tg_c: Some(120.0),
        photoresist: false,
        note: "Large sheets to be cut",
    },
    // ---- Plain boards of the electronics shops
    Material {
        maker: "Fortex",
        name: "FR4 1.55 mm",
        core: Core::Fr4,
        thickness_mm: &[1.55],
        copper_um: &[35.0],
        sides: Sides::Either,
        formats_mm: EUROCARD,
        dk: Some(4.5),
        df: None,
        tg_c: Some(130.0),
        photoresist: false,
        note: "Single-sided, 100 x 160 mm and 233.4 x 160 mm",
    },
    Material {
        maker: "CIF",
        name: "FR4 1.6 mm",
        core: Core::Fr4,
        thickness_mm: &[1.6],
        copper_um: &[18.0, 35.0],
        sides: Sides::Two,
        formats_mm: SHEETS,
        dk: Some(4.5),
        df: None,
        tg_c: Some(130.0),
        photoresist: false,
        note: "Double-sided boards for prototypes, 200 x 300 and 300 x 600 mm",
    },
    Material {
        maker: "Generic",
        name: "FR-4 (IPC-4101/21)",
        core: Core::Fr4,
        thickness_mm: &[0.2, 0.4, 0.8, 1.0, 1.2, 1.6, 2.0, 2.4, 3.2],
        copper_um: &[18.0, 35.0, 70.0, 105.0],
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(4.5),
        df: Some(0.02),
        tg_c: Some(135.0),
        photoresist: false,
        note: "The usual rigid board; 1.6 mm is the standard thickness (tolerance about 0.15 mm)",
    },
    Material {
        maker: "Generic",
        name: "FR-4 high Tg (IPC-4101/126)",
        core: Core::Fr4,
        thickness_mm: &[0.8, 1.0, 1.6, 2.0],
        copper_um: &[18.0, 35.0, 70.0],
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(4.4),
        df: Some(0.018),
        tg_c: Some(170.0),
        photoresist: false,
        note: "For lead-free soldering and many layers",
    },
    Material {
        maker: "Generic",
        name: "CEM-1",
        core: Core::Cem1,
        thickness_mm: &[0.8, 1.0, 1.2, 1.6],
        copper_um: &[18.0, 35.0, 70.0],
        sides: Sides::One,
        formats_mm: &[],
        dk: Some(4.4),
        df: Some(0.025),
        tg_c: Some(120.0),
        photoresist: false,
        note: "Cheap single-sided boards; no plated holes",
    },
    Material {
        maker: "Generic",
        name: "CEM-3",
        core: Core::Cem3,
        thickness_mm: &[0.8, 1.0, 1.2, 1.6, 2.0],
        copper_um: &[18.0, 35.0, 70.0],
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(4.4),
        df: Some(0.02),
        tg_c: Some(125.0),
        photoresist: false,
        note: "A cheaper stand-in for FR-4 in boards of one or two layers",
    },
    Material {
        maker: "Generic",
        name: "FR-2",
        core: Core::Fr2,
        thickness_mm: &[0.8, 1.0, 1.6],
        copper_um: &[35.0],
        sides: Sides::One,
        formats_mm: &[],
        dk: Some(4.2),
        df: Some(0.03),
        tg_c: Some(105.0),
        photoresist: false,
        note: "Paper and phenolic; punched, not drilled, in volume",
    },
    // ---- High-speed and high-temperature epoxy
    Material {
        maker: "Isola",
        name: "370HR",
        core: Core::Fr4,
        thickness_mm: &[0.1, 0.2, 0.4, 0.8, 1.0, 1.6, 2.0],
        copper_um: &[18.0, 35.0, 70.0],
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(4.04),
        df: Some(0.021),
        tg_c: Some(180.0),
        photoresist: false,
        note: "High-Tg FR-4 for multilayer boards",
    },
    Material {
        maker: "Isola",
        name: "FR408HR",
        core: Core::Fr4,
        thickness_mm: &[0.1, 0.2, 0.4, 0.8, 1.0, 1.6],
        copper_um: &[18.0, 35.0],
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(3.68),
        df: Some(0.0095),
        tg_c: Some(190.0),
        photoresist: false,
        note: "Low-loss epoxy for fast digital boards",
    },
    Material {
        maker: "Panasonic",
        name: "Megtron 6 (R-5775)",
        core: Core::Fr4,
        thickness_mm: &[0.1, 0.2, 0.4, 0.8],
        copper_um: &[18.0, 35.0],
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(3.7),
        df: Some(0.002),
        tg_c: Some(185.0),
        photoresist: false,
        note: "Very low loss, for the fastest links",
    },
    Material {
        maker: "Shengyi",
        name: "S1000-2",
        core: Core::Fr4,
        thickness_mm: &[0.2, 0.4, 0.8, 1.0, 1.6, 2.0],
        copper_um: &[18.0, 35.0, 70.0],
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(4.3),
        df: Some(0.016),
        tg_c: Some(170.0),
        photoresist: false,
        note: "High-Tg FR-4 widely used in volume production",
    },
    Material {
        maker: "ITEQ",
        name: "IT-180A",
        core: Core::Fr4,
        thickness_mm: &[0.2, 0.4, 0.8, 1.0, 1.6, 2.0],
        copper_um: &[18.0, 35.0, 70.0],
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(4.4),
        df: Some(0.015),
        tg_c: Some(175.0),
        photoresist: false,
        note: "High-Tg FR-4",
    },
    Material {
        maker: "Ventec",
        name: "VT-47",
        core: Core::Fr4,
        thickness_mm: &[0.2, 0.4, 0.8, 1.0, 1.6, 2.0],
        copper_um: &[18.0, 35.0, 70.0],
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(4.4),
        df: Some(0.016),
        tg_c: Some(180.0),
        photoresist: false,
        note: "High-Tg FR-4",
    },
    Material {
        maker: "Nelco",
        name: "N4000-13",
        core: Core::Fr4,
        thickness_mm: &[0.1, 0.2, 0.4, 0.8, 1.6],
        copper_um: &[18.0, 35.0],
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(3.7),
        df: Some(0.009),
        tg_c: Some(210.0),
        photoresist: false,
        note: "Low-loss epoxy",
    },
    Material {
        maker: "Arlon",
        name: "85N polyimide",
        core: Core::Polyimide,
        thickness_mm: &[0.2, 0.4, 0.8, 1.6],
        copper_um: &[18.0, 35.0, 70.0],
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(4.2),
        df: Some(0.01),
        tg_c: Some(250.0),
        photoresist: false,
        note: "For high temperature",
    },
    // ---- Radio-frequency laminates
    Material {
        maker: "Rogers",
        name: "RO4003C",
        core: Core::Hydrocarbon,
        thickness_mm: &[0.203, 0.305, 0.406, 0.508, 0.813, 1.524],
        copper_um: &[17.0, 35.0, 70.0],
        sides: Sides::Either,
        formats_mm: RF_PANEL,
        dk: Some(3.38),
        df: Some(0.0027),
        tg_c: Some(280.0),
        photoresist: false,
        note: "Hydrocarbon ceramic; drills like FR-4 but wears the tools faster",
    },
    Material {
        maker: "Rogers",
        name: "RO4350B",
        core: Core::Hydrocarbon,
        thickness_mm: &[0.168, 0.254, 0.338, 0.508, 0.762, 1.524],
        copper_um: &[17.0, 35.0, 70.0],
        sides: Sides::Either,
        formats_mm: RF_PANEL,
        dk: Some(3.48),
        df: Some(0.0037),
        tg_c: Some(280.0),
        photoresist: false,
        note: "Flame retardant (UL 94 V-0) RF laminate",
    },
    Material {
        maker: "Rogers",
        name: "RO3003",
        core: Core::Ptfe,
        thickness_mm: &[0.13, 0.25, 0.5, 0.75, 1.52],
        copper_um: &[17.0, 35.0],
        sides: Sides::Either,
        formats_mm: RF_PANEL,
        dk: Some(3.0),
        df: Some(0.0013),
        tg_c: None,
        photoresist: false,
        note: "Ceramic filled PTFE for millimetre waves; soft, needs sharp tools",
    },
    Material {
        maker: "Rogers",
        name: "RO3010",
        core: Core::Ptfe,
        thickness_mm: &[0.25, 0.64, 1.28],
        copper_um: &[17.0, 35.0],
        sides: Sides::Either,
        formats_mm: RF_PANEL,
        dk: Some(10.2),
        df: Some(0.0022),
        tg_c: None,
        photoresist: false,
        note: "High dielectric constant for small circuits",
    },
    Material {
        maker: "Rogers",
        name: "RT/duroid 5880",
        core: Core::Ptfe,
        thickness_mm: &[0.127, 0.254, 0.508, 0.787, 1.575],
        copper_um: &[17.0, 35.0],
        sides: Sides::Either,
        formats_mm: RF_PANEL,
        dk: Some(2.2),
        df: Some(0.0009),
        tg_c: None,
        photoresist: false,
        note: "Glass microfibre PTFE, the lowest loss",
    },
    Material {
        maker: "Rogers",
        name: "RT/duroid 6002",
        core: Core::Ptfe,
        thickness_mm: &[0.127, 0.254, 0.508, 0.762, 1.524],
        copper_um: &[17.0, 35.0],
        sides: Sides::Either,
        formats_mm: RF_PANEL,
        dk: Some(2.94),
        df: Some(0.0012),
        tg_c: None,
        photoresist: false,
        note: "Ceramic filled PTFE, stable over temperature",
    },
    Material {
        maker: "Taconic",
        name: "TLY-5",
        core: Core::Ptfe,
        thickness_mm: &[0.127, 0.254, 0.508, 0.787, 1.524],
        copper_um: &[17.0, 35.0],
        sides: Sides::Either,
        formats_mm: RF_PANEL,
        dk: Some(2.2),
        df: Some(0.0009),
        tg_c: None,
        photoresist: false,
        note: "Woven glass PTFE",
    },
    Material {
        maker: "Taconic",
        name: "RF-35",
        core: Core::Ptfe,
        thickness_mm: &[0.254, 0.508, 0.762, 1.524],
        copper_um: &[17.0, 35.0],
        sides: Sides::Either,
        formats_mm: RF_PANEL,
        dk: Some(3.5),
        df: Some(0.0018),
        tg_c: None,
        photoresist: false,
        note: "Ceramic filled PTFE with glass",
    },
    // ---- Special bases
    Material {
        maker: "Bergquist",
        name: "Thermal Clad (aluminium)",
        core: Core::Aluminium,
        thickness_mm: &[1.0, 1.5, 2.0, 3.0],
        copper_um: &[35.0, 70.0, 105.0],
        sides: Sides::One,
        formats_mm: &[],
        dk: None,
        df: None,
        tg_c: None,
        photoresist: false,
        note: "Aluminium core for LED and power boards; carbide tools",
    },
    Material {
        maker: "DuPont",
        name: "Pyralux AP flexible",
        core: Core::Flex,
        thickness_mm: &[0.05, 0.075, 0.1, 0.125],
        copper_um: &[12.0, 18.0, 35.0],
        sides: Sides::Either,
        formats_mm: &[(250.0, 300.0)],
        dk: Some(3.4),
        df: Some(0.002),
        tg_c: None,
        photoresist: false,
        note: "Polyimide film with copper, for flexible circuits",
    },
];

/// The makers in the library, once each, in the order of the list.
pub fn makers() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = Vec::new();
    for m in MATERIALS {
        if !v.contains(&m.maker) {
            v.push(m.maker);
        }
    }
    v
}

/// The materials of one maker.
pub fn of_maker(maker: &str) -> Vec<&'static Material> {
    MATERIALS.iter().filter(|m| m.maker == maker).collect()
}

/// The board as it is chosen: the material, its thickness and copper, the copper sides and the size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Board {
    pub material: usize,
    pub thickness_mm: f64,
    pub copper_um: f64,
    /// 1 for copper on one side, 2 for both sides.
    pub layers: u8,
    /// The size in mm; `None` takes the size from the files of the job.
    pub size_mm: Option<(f64, f64)>,
}

impl Board {
    /// The usual board: FR-4 of 1.6 mm with 35 um copper on both sides.
    pub fn standard() -> Board {
        let material = MATERIALS
            .iter()
            .position(|m| m.maker == "Generic" && m.name.starts_with("FR-4 (IPC"))
            .unwrap_or(0);
        Board {
            material,
            thickness_mm: 1.6,
            copper_um: 35.0,
            layers: 2,
            size_mm: None,
        }
    }

    pub fn material(&self) -> &'static Material {
        &MATERIALS[self.material.min(MATERIALS.len() - 1)]
    }

    /// Takes another material: the thickness and the copper go to the nearest the new one offers, and the sides to what it can have.
    pub fn with_material(self, index: usize) -> Board {
        let m = &MATERIALS[index.min(MATERIALS.len() - 1)];
        let near = |list: &[f64], v: f64| {
            list.iter()
                .copied()
                .min_by(|a, b| (a - v).abs().total_cmp(&(b - v).abs()))
                .unwrap_or(v)
        };
        Board {
            material: index.min(MATERIALS.len() - 1),
            thickness_mm: near(m.thickness_mm, self.thickness_mm),
            copper_um: near(m.copper_um, self.copper_um),
            layers: match m.sides {
                Sides::One => 1,
                Sides::Two => 2,
                Sides::Either => self.layers.clamp(1, 2),
            },
            size_mm: self.size_mm,
        }
    }

    /// A one-line description for the screen.
    pub fn describe(&self) -> String {
        let m = self.material();
        format!(
            "{} {} - {:.2} mm - {:.0} um Cu - {}",
            m.maker,
            m.name,
            self.thickness_mm,
            self.copper_um,
            if self.layers == 1 {
                "1 layer"
            } else {
                "2 layers"
            }
        )
    }
}

/// How the board is remembered between runs: the name of the material, the numbers and the size.
pub fn to_text(b: &Board) -> String {
    let m = b.material();
    let size = b.size_mm.map_or(String::new(), |(w, d)| format!("{w}x{d}"));
    format!(
        "{}|{}|{}|{}|{}|{}",
        m.maker, m.name, b.thickness_mm, b.copper_um, b.layers, size
    )
}

/// The other way round; a text that is not understood gives `None`.
pub fn from_text(t: &str) -> Option<Board> {
    let p: Vec<&str> = t.trim().split('|').collect();
    if p.len() != 6 {
        return None;
    }
    let material = MATERIALS
        .iter()
        .position(|m| m.maker == p[0] && m.name == p[1])?;
    let size_mm = match p[5] {
        "" => None,
        s => {
            let (w, d) = s.split_once('x')?;
            Some((w.parse().ok()?, d.parse().ok()?))
        }
    };
    Some(Board {
        material,
        thickness_mm: p[2].parse().ok()?,
        copper_um: p[3].parse().ok()?,
        layers: p[4].parse().ok().filter(|l| (1..=2).contains(l))?,
        size_mm,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_material_is_complete() {
        assert!(MATERIALS.len() >= 25);
        for m in MATERIALS {
            assert!(
                !m.thickness_mm.is_empty() && !m.copper_um.is_empty(),
                "{}",
                m.name
            );
            assert!(
                m.thickness_mm.iter().all(|t| (0.04..=4.0).contains(t)),
                "{}",
                m.name
            );
            assert!(
                m.copper_um.iter().all(|c| (9.0..=210.0).contains(c)),
                "{}",
                m.name
            );
            assert!(
                m.formats_mm.iter().all(|&(w, d)| w > 20.0 && d > 20.0),
                "{}",
                m.name
            );
            assert!(m.dk.is_none_or(|d| (2.0..=11.0).contains(&d)), "{}", m.name);
            assert!(!m.note.is_empty());
        }
    }

    #[test]
    fn a_material_is_listed_once() {
        for (i, a) in MATERIALS.iter().enumerate() {
            for b in &MATERIALS[i + 1..] {
                assert!(
                    !(a.maker == b.maker && a.name == b.name),
                    "{} {}",
                    a.maker,
                    a.name
                );
            }
        }
        assert!(makers().len() >= 12);
        assert!(!of_maker("Bungard").is_empty() && !of_maker("Rogers").is_empty());
    }

    #[test]
    fn a_new_material_keeps_the_nearest_thickness_and_copper() {
        let b = Board::standard();
        let rogers = MATERIALS.iter().position(|m| m.name == "RO4003C").unwrap();
        let r = b.with_material(rogers);
        assert!((r.thickness_mm - 1.524).abs() < 1e-9);
        assert_eq!(r.copper_um, 35.0);
        let cem1 = MATERIALS.iter().position(|m| m.name == "CEM-1").unwrap();
        assert_eq!(b.with_material(cem1).layers, 1);
    }

    #[test]
    fn the_board_comes_back_from_its_text() {
        let mut b = Board::standard();
        b.size_mm = Some((100.0, 80.0));
        assert_eq!(from_text(&to_text(&b)), Some(b));
        assert_eq!(from_text("x|y|1|2|3|"), None);
        assert_eq!(from_text(""), None);
        assert!(b.describe().contains("FR-4"));
    }
}
