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
    /// Alumina or aluminium nitride.
    Ceramic,
    /// A metal core: aluminium with a thin dielectric (LED boards).
    Aluminium,
    /// A flexible polyimide film.
    Flex,
    /// A solid aluminium alloy sheet (not the metal core of an LED board, which is `Aluminium`).
    Metal,
    /// A solid copper sheet.
    Copper,
    /// Wood and wood products (plywood, MDF, balsa).
    Wood,
    /// A rigid thermoplastic that cuts clean (acrylic, polycarbonate, POM, PVC).
    Plastic,
    /// A thermoplastic that softens and sticks to the tool (ABS, nylon, HDPE, PETG).
    SoftPlastic,
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
            Core::Ceramic => "Ceramic",
            Core::Aluminium => "Aluminium",
            Core::Flex => "Flex",
            Core::Metal => "Metal",
            Core::Copper => "Copper",
            Core::Wood => "Wood",
            Core::Plastic => "Plastic",
            Core::SoftPlastic => "Soft plastic",
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
    /// No copper at all: a plain sheet to mill and drill (metal, wood, plastic, bare laminate).
    NoCopper,
}

/// The standard cuttings of the Bungard boards (the catalogue of 2024/25: 1.5 mm, 35 and 70 um) and the two panels they are cut from.
const STD_BUNGARD: &[(f64, f64)] = &[
    (75.0, 100.0),
    (100.0, 160.0),
    (125.0, 175.0),
    (150.0, 200.0),
    (150.0, 250.0),
    (160.0, 233.4),
    (200.0, 250.0),
    (210.0, 300.0),
    (510.0, 570.0),
    (510.0, 1150.0),
];

/// The panels the Rogers data sheets list: 12 x 18, 24 x 18, 24 x 36 and 48 x 36 inches.
const RF_PANEL: &[(f64, f64)] = &[
    (304.8, 457.2),
    (609.6, 457.2),
    (609.6, 914.4),
    (1219.2, 914.4),
];
/// The master sheets of the glass-epoxy laminates: 36 x 48, 40 x 48 (the Shengyi data sheets) and 42 x 48 inches; the maker cuts them to the panel the fabricator asks for.
const MASTER_SHEETS: &[(f64, f64)] = &[(914.4, 1219.2), (1016.0, 1219.2), (1066.8, 1219.2)];
/// The sizes of the Fortex boards of 1.6 mm (the maker's sheet): 160 x 100, 100 x 220, 203 x 114, 233.4 x 160, 233.4 x 220, 305 x 457 and 510 x 1150 mm.
const FORTEX_SIZES: &[(f64, f64)] = &[
    (160.0, 100.0),
    (100.0, 220.0),
    (203.0, 114.0),
    (233.4, 160.0),
    (233.4, 220.0),
    (305.0, 457.0),
    (510.0, 1150.0),
];
/// The sizes of the 500 series of MG Chemicals, in inches: 3 x 5, 4 x 6, 6 x 6, 8 x 10 and 12 x 12.
const MG_SIZES: &[(f64, f64)] = &[
    (76.2, 127.0),
    (101.6, 152.4),
    (152.4, 152.4),
    (203.2, 254.0),
    (304.8, 304.8),
];

/// The master sheets that the Shengyi data sheets list, as the maker writes them in mm: 36 x 48, 40 x 48 and 42 x 48 inches.
const SHENGYI_SHEETS: &[(f64, f64)] = &[(915.0, 1220.0), (1020.0, 1220.0), (1070.0, 1220.0)];
/// The two sheets (36 x 48 and 40 x 48 inches) of the Shengyi PTFE data sheets.
const SHENGYI_SHEETS_2: &[(f64, f64)] = &[(915.0, 1220.0), (1020.0, 1220.0)];
/// The panels of the Rogers RO3000 data sheet: 12 x 18 and 24 x 18 inches.
const RF_PANEL_SMALL: &[(f64, f64)] = &[(304.8, 457.2), (609.6, 457.2)];
/// The sheets of the Taconic RF-35: 12 x 18, 16 x 18 and 18 x 24 inches and the standard sheet of 36 x 48 inches.
const TACONIC_RF35: &[(f64, f64)] = &[
    (304.8, 457.2),
    (406.4, 457.2),
    (457.2, 609.6),
    (914.0, 1220.0),
];
/// The sheets of the Taconic TLC data sheet, in mm as the maker writes them, and the standard sheet of 36 x 48 inches.
const TACONIC_TLC: &[(f64, f64)] = &[
    (304.0, 457.0),
    (406.0, 457.0),
    (457.0, 610.0),
    (406.0, 914.0),
    (610.0, 914.0),
    (457.0, 1220.0),
    (914.0, 1220.0),
];

const TH_FR4: &[f64] = &[0.1, 0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.6, 2.0, 2.4, 3.2];
const TH_THIN: &[f64] = &[0.1, 0.2, 0.4, 0.8, 1.0, 1.6, 2.0];
const TH_THICK: &[f64] = &[0.5, 0.8, 1.0, 1.2, 1.6, 2.0, 2.4, 3.2];
const TH_PAPER: &[f64] = &[0.8, 1.0, 1.2, 1.6];
const TH_RF: &[f64] = &[0.127, 0.254, 0.508, 0.762, 0.787, 1.524];
const TH_CERAMIC: &[f64] = &[0.38, 0.635, 1.0];
const TH_METAL: &[f64] = &[1.0, 1.5, 2.0, 3.0];
const TH_FLEX: &[f64] = &[0.05, 0.075, 0.1, 0.125];
/// The thicknesses of a laminate that the data sheet gives as a range of 0.05 mm up to 1.5, 2.4 and 3.2 mm: the usual steps inside the range.
const TH_TO_15: &[f64] = &[0.05, 0.1, 0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.5];
const TH_TO_24: &[f64] = &[0.05, 0.1, 0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.6, 2.0, 2.4];
const TH_TO_32: &[f64] = &[0.05, 0.1, 0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.6, 2.0, 2.4, 3.2];
/// The usual thicknesses of aluminium sheet inside 0.2 to 5 mm.
const TH_ALUMINIUM_SHEET: &[f64] = &[
    0.2, 0.3, 0.4, 0.5, 0.6, 0.8, 1.0, 1.2, 1.5, 2.0, 2.5, 3.0, 4.0, 5.0,
];
/// The usual thicknesses of copper sheet inside 0.1 to 5 mm.
const TH_COPPER_SHEET: &[f64] = &[0.1, 0.2, 0.3, 0.5, 0.8, 1.0, 1.5, 2.0, 3.0, 4.0, 5.0];
/// The usual thicknesses of PTFE sheet inside 0.2 to 5 mm.
const TH_PTFE_SHEET: &[f64] = &[0.2, 0.5, 0.8, 1.0, 1.5, 2.0, 3.0, 4.0, 5.0];
/// The usual thicknesses of plywood and the light woods inside 0.5 to 5 mm.
const TH_WOOD: &[f64] = &[0.5, 0.8, 1.0, 1.5, 2.0, 3.0, 4.0, 5.0];
const TH_MDF: &[f64] = &[2.5, 3.0, 4.0, 5.0];
const TH_BALSA: &[f64] = &[0.5, 0.8, 1.0, 1.5, 2.0, 2.5, 3.0, 4.0, 5.0];
/// The usual thicknesses of the extruded and cast plastic sheet up to 5 mm.
const TH_PLASTIC: &[f64] = &[1.0, 1.5, 2.0, 3.0, 4.0, 5.0];
const TH_PLASTIC_THIN: &[f64] = &[0.5, 1.0, 1.5, 2.0, 3.0, 4.0, 5.0];
const TH_PLASTIC_THICK: &[f64] = &[1.0, 2.0, 3.0, 4.0, 5.0];
const TH_G10: &[f64] = &[0.5, 0.8, 1.0, 1.6, 2.0, 3.0, 4.0, 5.0];
const TH_PERTINAX: &[f64] = &[0.8, 1.0, 1.5, 2.0, 3.0, 4.0, 5.0];
const CU_STD: &[f64] = &[18.0, 35.0, 70.0, 105.0];
/// 12 to 105 um (3/8 to 3 oz), the usual steps of a range of the data sheets.
const CU_12_105: &[f64] = &[12.0, 18.0, 35.0, 70.0, 105.0];
/// 1/2, 1 and 2 oz.
const CU_ISOLA: &[f64] = &[18.0, 35.0, 70.0];
const CU_RF: &[f64] = &[17.0, 35.0];
const CU_THIN: &[f64] = &[12.0, 18.0, 35.0];
const CU_PAPER: &[f64] = &[35.0];
const CU_CERAMIC: &[f64] = &[35.0, 70.0];
const CU_METAL: &[f64] = &[35.0, 70.0, 105.0];

/// A material of the library with no standard formats (it is sold by the panel or cut to size).
#[allow(clippy::too_many_arguments)]
const fn m(
    maker: &'static str,
    name: &'static str,
    core: Core,
    thickness_mm: &'static [f64],
    copper_um: &'static [f64],
    sides: Sides,
    dk: Option<f64>,
    df: Option<f64>,
    tg_c: Option<f64>,
    note: &'static str,
) -> Material {
    Material {
        maker,
        name,
        core,
        thickness_mm,
        copper_um,
        sides,
        formats_mm: &[],
        dk,
        df,
        tg_c,
        photoresist: false,
        note,
    }
}

/// Every material of the library.
pub const MATERIALS: &[Material] = &[
    // ---- Bungard: the boards for the machines of the same maker, with and without photosensitive resist
    Material {
        maker: "Bungard",
        name: "FR4 epoxy, positive photoresist",
        core: Core::Fr4,
        thickness_mm: &[0.5, 0.8, 1.0, 1.5, 2.0, 2.5],
        copper_um: &[18.0, 35.0, 70.0, 105.0],
        sides: Sides::Either,
        formats_mm: STD_BUNGARD,
        dk: Some(4.5),
        df: None,
        tg_c: Some(130.0),
        photoresist: true,
        note: "Bungard catalogue: FR4 in 0.5 to 2.5 mm (the 1.5 mm is sold as 1.5 or 1.6) with 18, 35 or 70 um copper (105 um at 1.5 mm); standard cuttings and panels of 510 x 570 and 510 x 1150 mm; resist 5 um, sensitive to 350-400 nm",
    },
    Material {
        maker: "Bungard",
        name: "FR4 epoxy, bare copper",
        core: Core::Fr4,
        thickness_mm: &[0.5, 0.8, 1.0, 1.5, 2.0, 2.5],
        copper_um: &[18.0, 35.0, 70.0],
        sides: Sides::Either,
        formats_mm: STD_BUNGARD,
        dk: Some(4.5),
        df: None,
        tg_c: Some(130.0),
        photoresist: false,
        note: "The same boards without resist, cut to any size from 50 x 50 mm (0.1 mm accuracy; boards of 2 mm or more are sawn, 3 mm lost per cut); carrier 1.55 +/- 0.08 mm, copper +/- 5 um at 35 um",
    },
    Material {
        maker: "Bungard",
        name: "Laminated paper (FR2), photoresist",
        core: Core::Fr2,
        thickness_mm: &[1.5],
        copper_um: &[35.0],
        sides: Sides::Either,
        formats_mm: STD_BUNGARD,
        dk: Some(4.2),
        df: None,
        tg_c: Some(105.0),
        photoresist: true,
        note: "Cheap boards for simple circuits (single or double sided, 35 um); FR2 is the best for isolation milling because the routers last longer; panels of 480 x 1000 mm",
    },
    Material {
        maker: "Bungard",
        name: "FR2 / FR3 / CEM-1 sheets",
        core: Core::Cem1,
        thickness_mm: &[1.5],
        copper_um: &[35.0],
        sides: Sides::Either,
        formats_mm: &[(480.0, 1000.0), (510.0, 1150.0)],
        dk: Some(4.4),
        df: None,
        tg_c: Some(120.0),
        photoresist: false,
        note: "Large panels to be cut: FR2 480 x 1000 mm (35 um, single or double sided), FR3 510 x 1150 mm (35 um, double sided) and CEM-1 510 x 1150 mm (35 um, single sided)",
    },
    // ---- Plain boards of the electronics shops
    // ---- MG Chemicals: plain boards of the 500 series
    Material {
        maker: "MG Chemicals",
        name: "FR4 500 series, 1.6 mm",
        core: Core::Fr4,
        thickness_mm: &[1.6],
        copper_um: &[17.0, 35.0],
        sides: Sides::Either,
        formats_mm: MG_SIZES,
        dk: Some(4.5),
        df: None,
        tg_c: Some(130.0),
        photoresist: false,
        note: "1/16 inch; single-sided in 3 x 5, 4 x 6, 6 x 6, 8 x 10 and 12 x 12 inches, double-sided in 3 x 5, 6 x 6 and 12 x 12; copper 1 oz (some listings say 1/2 oz)",
    },
    Material {
        maker: "MG Chemicals",
        name: "FR4 500 series, 0.8 mm",
        core: Core::Fr4,
        thickness_mm: &[0.8],
        copper_um: &[17.0],
        sides: Sides::One,
        formats_mm: &[(152.4, 228.6)],
        dk: Some(4.5),
        df: None,
        tg_c: Some(130.0),
        photoresist: false,
        note: "1/32 inch with 1/2 oz copper, single sided, 152 x 228 mm (data sheet of the 500 series)",
    },
    Material {
        maker: "MG Chemicals",
        name: "FR4 600 series, positive presensitized",
        core: Core::Fr4,
        thickness_mm: &[0.8, 1.6],
        copper_um: &[35.0],
        sides: Sides::Either,
        formats_mm: &[(152.0, 230.0), (150.0, 230.0)],
        dk: Some(4.45),
        df: Some(0.0175),
        tg_c: Some(137.5),
        photoresist: true,
        note: "1 oz copper with positive photoresist, translucent FR4, UL 94 V-0; 1.6 mm double sided 152 x 230 mm, 0.8 mm single or double sided 150 x 230 mm; Tg 130-145 C, Dk 4.2-4.7 and Df 0.015-0.020 at 1 MHz (data sheet of the 600 series; the figures are the middle of the typical ranges)",
    },
    Material {
        maker: "Fortex",
        name: "FR4 1.6 mm",
        core: Core::Fr4,
        thickness_mm: &[1.6],
        copper_um: &[35.0],
        sides: Sides::Either,
        formats_mm: FORTEX_SIZES,
        dk: Some(4.5),
        df: None,
        tg_c: Some(130.0),
        photoresist: false,
        note: "Single-sided (35 / 0 um) or double-sided (35 / 35 um), in the seven sizes of the maker; other sizes and thicknesses on request",
    },
    Material {
        maker: "CIF",
        name: "FR4 1.6 mm",
        core: Core::Fr4,
        thickness_mm: &[1.6],
        copper_um: CU_ISOLA,
        sides: Sides::Two,
        formats_mm: &[(200.0, 300.0), (300.0, 600.0), (305.0, 457.0), (457.0, 610.0), (927.0, 1232.0), (1082.0, 1232.0)],
        dk: Some(4.8),
        df: Some(0.027),
        tg_c: Some(135.0),
        photoresist: false,
        note: "Double-sided FR-4 for prototypes to IPC-4101/21, UL 94 V-0, Tg 135 C by DSC, comparative tracking index 230 V typical; Dk and Df at 1 MHz; copper 18, 35 and 70 um; sheets of 927 x 1232 and 1082 x 1232 mm and cut panels; the data sheet gives its figures for a 1.6 mm board",
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
        thickness_mm: TH_TO_32,
        copper_um: CU_ISOLA,
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(4.04),
        df: Some(0.021),
        tg_c: Some(180.0),
        photoresist: false,
        note: "Lead-free high-Tg FR-4 to IPC-4101/98/99/101/126, Td 340 C, CAF resistant, EN 45545-2 certified; 0.05 to 3.2 mm as a range (the steps shown are the usual ones); 1/2, 1 and 2 oz copper; the data sheet gives no sheet sizes",
    },
    Material {
        maker: "Isola",
        name: "IS410",
        core: Core::Fr4,
        thickness_mm: TH_TO_32,
        copper_um: CU_ISOLA,
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(3.97),
        df: Some(0.02),
        tg_c: Some(180.0),
        photoresist: false,
        note: "Lead-free FR-4 epoxy to IPC-4101/21/24/26/121/124/129, Td 350 C, 6 x 288 C solder float, EN 45545-2 certified; 0.05 to 3.2 mm as a range (the steps shown are the usual ones); 1/2, 1 and 2 oz copper; the data sheet gives no sheet sizes",
    },
    Material {
        maker: "Isola",
        name: "G200",
        core: Core::Fr4,
        thickness_mm: TH_TO_32,
        copper_um: CU_ISOLA,
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(3.7),
        df: Some(0.013),
        tg_c: Some(180.0),
        photoresist: false,
        note: "BT-epoxy (bismaleimide/triazine) to IPC-4101/30, Td 325 C, CAF resistant, for large high layer count boards; 0.05 to 3.2 mm as a range (the steps shown are the usual ones); 1/2, 1 and 2 oz copper; the data sheet gives no sheet sizes",
    },
    Material {
        maker: "Isola",
        name: "FR408HR",
        core: Core::Fr4,
        thickness_mm: &[0.1, 0.2, 0.4, 0.8, 1.0, 1.6],
        copper_um: &[18.0, 35.0],
        sides: Sides::Either,
        formats_mm: MASTER_SHEETS,
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
        thickness_mm: &[0.05, 0.065, 0.075, 0.09, 0.1, 0.125, 0.127, 0.13, 0.146, 0.15, 0.178, 0.2, 0.25, 0.3, 0.4, 0.5, 0.555, 0.625, 0.74, 0.75, 0.925],
        copper_um: &[35.0],
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(3.61),
        df: Some(0.004),
        tg_c: Some(185.0),
        photoresist: false,
        note: "Ultra low loss multilayer material (laminate R-5775, prepreg R-5670), UL 94 V-0; Tg 185 C by DSC (210 C by DMA), Td 410 C, T288 above 120 min; Dk 3.61 and Df 0.004 at 10 GHz on the 0.75 mm core (3.71 and 0.002 at 1 GHz; the low Dk glass cloth R-5775N gives 3.34 and 0.0037 at 13 GHz); core thicknesses of the data sheet, 1 oz H-VLP copper; the sheet lists no sheet sizes",
    },
    Material {
        maker: "Shengyi",
        name: "S1000-2",
        core: Core::Fr4,
        thickness_mm: TH_TO_32,
        copper_um: CU_12_105,
        sides: Sides::Either,
        formats_mm: SHENGYI_SHEETS,
        dk: Some(4.8),
        df: Some(0.013),
        tg_c: Some(180.0),
        photoresist: false,
        note: "Lead-free FR-4, UL 94 V-0; Tg 180 C by DSC (185 C by DMA), Td 345 C, T260 60 min, T288 20 min; Dk and Df at 1 MHz; the sheet gives the thickness as 0.002 to 0.125 in (0.5 to 3.2 mm in its text) and the copper from 3/8 to 3 oz (12 to 105 um) as ranges, so the steps shown are the usual ones; sheets of 36 x 48, 40 x 48 and 42 x 48 in",
    },
    Material {
        maker: "ITEQ",
        name: "IT-180A",
        core: Core::Fr4,
        thickness_mm: &[0.2, 0.4, 0.8, 1.0, 1.6, 2.0],
        copper_um: CU_ISOLA,
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(4.4),
        df: Some(0.015),
        tg_c: Some(175.0),
        photoresist: false,
        note: "High-Tg low-CTE epoxy to IPC-4101C/99/101/126, UL 94 V-0, Tg 175 C by DSC (170 C minimum), Td 345 C, T260 above 60 min, T288 above 30 min, CAF resistant; Dk and Df at 1 MHz; the data sheet lists no thicknesses, copper weights or sheet sizes (the ones shown are the usual of FR-4)",
    },
    Material {
        maker: "Ventec",
        name: "VT-47",
        core: Core::Fr4,
        thickness_mm: &[0.2, 0.4, 0.8, 1.0, 1.6, 2.0],
        copper_um: &[18.0, 35.0, 70.0],
        sides: Sides::Either,
        formats_mm: MASTER_SHEETS,
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
        formats_mm: MASTER_SHEETS,
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
        note: "Polyimide laminate and prepreg for high temperature to IPC-4101/40 and /41, halogen-free; Tg above 250 C by TMA, Td 407 C, T260, T288 and T300 above 60 min, z expansion 1.2 % from 50 to 260 C; Dk and Df at 1 MHz; the data sheet lists no laminate thicknesses, copper weights or sheet sizes (the ones shown are the usual)",
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
        thickness_mm: &[0.101, 0.168, 0.254, 0.338, 0.422, 0.508, 0.762, 1.524],
        copper_um: &[17.0, 35.0, 70.0],
        sides: Sides::Either,
        formats_mm: RF_PANEL,
        dk: Some(3.48),
        df: Some(0.0037),
        tg_c: Some(280.0),
        photoresist: false,
        note: "Flame retardant (UL 94 V-0) RF laminate, drop-in for RO4003C; Dk 3.48 +/- 0.05 and Df 0.0037 at 10 GHz, Td 390 C; the 0.101 mm material only in panels up to 24 x 18 in; LoPro foil in 1/2 and 1 oz for PIM-sensitive uses",
    },
    Material {
        maker: "Rogers",
        name: "RO3003",
        core: Core::Ptfe,
        thickness_mm: &[0.13, 0.25, 0.5, 0.75, 1.52],
        copper_um: &[17.0, 35.0, 70.0],
        sides: Sides::Either,
        formats_mm: RF_PANEL_SMALL,
        dk: Some(3.0),
        df: Some(0.0013),
        tg_c: None,
        photoresist: false,
        note: "Ceramic filled PTFE for millimetre waves up to 77 GHz; Dk 3.00 +/- 0.04 and Df 0.0013 at 10 GHz, Td 500 C, UL 94 V-0; soft, needs sharp tools; panels of 12 x 18 and 24 x 18 in",
    },
    Material {
        maker: "Rogers",
        name: "RO3010",
        core: Core::Ptfe,
        thickness_mm: &[0.13, 0.25, 0.64, 1.28],
        copper_um: &[17.0, 35.0, 70.0],
        sides: Sides::Either,
        formats_mm: RF_PANEL_SMALL,
        dk: Some(10.2),
        df: Some(0.0022),
        tg_c: None,
        photoresist: false,
        note: "High dielectric constant for small circuits; Dk 10.2 +/- 0.30 and Df 0.0022 at 10 GHz, Td 500 C, UL 94 V-0; panels of 12 x 18 and 24 x 18 in",
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
        formats_mm: &[(457.2, 609.6)],
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
        thickness_mm: &[0.25, 0.5, 0.76, 1.52],
        copper_um: &[17.0, 35.0, 70.0],
        sides: Sides::Either,
        formats_mm: TACONIC_RF35,
        dk: Some(3.5),
        df: Some(0.0018),
        tg_c: Some(315.0),
        photoresist: false,
        note: "Ceramic filled PTFE on woven glass (ORCER), UL 94 V-0, Tg above 315 C; Dk 3.50 and Df 0.0018 at 1.9 GHz; 0.010, 0.020, 0.030 and 0.060 in; 1/2, 1 and 2 oz copper; standard sheet 36 x 48 in",
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
        thickness_mm: &[0.0254, 0.0508, 0.0762, 0.1016, 0.127, 0.1524],
        copper_um: &[9.0, 12.0, 18.0, 35.0, 70.0],
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(3.4),
        df: Some(0.002),
        tg_c: Some(220.0),
        photoresist: false,
        note: "All-polyimide adhesiveless laminate, UL 94 V-0, IPC-4204/11, 180 C maximum operating temperature; dielectric of 1 to 6 mil, copper 9 to 70 um (rolled or electrodeposited); Dk 3.4 and Df 0.002 at 1 MHz (0.003 at 1 mil)",
    },
    // ---- more makers and more kinds of base material
    m(
        "Generic",
        "FR-4 Tg 150 (IPC-4101/24)",
        Core::Fr4,
        TH_FR4,
        CU_STD,
        Sides::Either,
        Some(4.4),
        Some(0.019),
        Some(150.0),
        "Medium glass transition; for lead-free soldering",
    ),
    m(
        "Generic",
        "FR-4 halogen free",
        Core::Fr4,
        TH_FR4,
        CU_STD,
        Sides::Either,
        Some(4.4),
        Some(0.015),
        Some(150.0),
        "Without halogens, for equipment that must be halogen free",
    ),
    m(
        "Generic",
        "FR-5 (glass epoxy, high temperature)",
        Core::Fr4,
        TH_THICK,
        CU_STD,
        Sides::Either,
        Some(4.6),
        Some(0.02),
        Some(170.0),
        "Keeps its strength at high temperature",
    ),
    m(
        "Generic",
        "G-10 (glass epoxy)",
        Core::Fr4,
        TH_THICK,
        CU_STD,
        Sides::Either,
        Some(4.8),
        Some(0.02),
        Some(130.0),
        "The glass epoxy without flame retardant",
    ),
    m(
        "Generic",
        "CEM-2",
        Core::Cem1,
        TH_THICK,
        CU_STD,
        Sides::One,
        Some(4.4),
        Some(0.025),
        Some(120.0),
        "Paper core with epoxy and glass faces",
    ),
    m(
        "Generic",
        "FR-1 (paper, phenolic)",
        Core::Fr2,
        TH_PAPER,
        CU_PAPER,
        Sides::One,
        Some(4.2),
        Some(0.04),
        Some(105.0),
        "The cheapest board, punched rather than drilled in volume",
    ),
    m(
        "Generic",
        "FR-3 (paper, epoxy)",
        Core::Fr3,
        TH_PAPER,
        CU_PAPER,
        Sides::One,
        Some(4.3),
        Some(0.03),
        Some(105.0),
        "Paper with epoxy, better than FR-2",
    ),
    m(
        "Generic",
        "Polyimide glass (IPC-4101/40)",
        Core::Polyimide,
        TH_FR4,
        CU_STD,
        Sides::Either,
        Some(4.2),
        Some(0.012),
        Some(250.0),
        "For high temperature and military boards",
    ),
    m(
        "Generic",
        "PTFE woven glass",
        Core::Ptfe,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(2.5),
        Some(0.002),
        None,
        "Soft; sharp tools and slow feeds",
    ),
    m(
        "Generic",
        "Ceramic filled PTFE",
        Core::Ptfe,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(3.0),
        Some(0.0015),
        None,
        "RF boards with a stable dielectric constant",
    ),
    m(
        "Generic",
        "Alumina ceramic 96 %",
        Core::Ceramic,
        TH_CERAMIC,
        CU_CERAMIC,
        Sides::Either,
        Some(9.8),
        Some(0.0003),
        None,
        "Hard: diamond or laser, not drills",
    ),
    m(
        "Generic",
        "Aluminium nitride ceramic",
        Core::Ceramic,
        TH_CERAMIC,
        CU_CERAMIC,
        Sides::Either,
        Some(8.6),
        Some(0.0005),
        None,
        "Very good heat conduction; diamond or laser",
    ),
    m(
        "Generic",
        "Aluminium IMS 1 W/mK",
        Core::Aluminium,
        TH_METAL,
        CU_METAL,
        Sides::One,
        None,
        None,
        None,
        "LED boards with a thermal dielectric of 1 W/mK",
    ),
    m(
        "Generic",
        "Aluminium IMS 2 W/mK",
        Core::Aluminium,
        TH_METAL,
        CU_METAL,
        Sides::One,
        None,
        None,
        None,
        "LED boards with a thermal dielectric of 2 W/mK",
    ),
    m(
        "Generic",
        "Aluminium IMS 3 W/mK",
        Core::Aluminium,
        TH_METAL,
        CU_METAL,
        Sides::One,
        None,
        None,
        None,
        "Power boards with a thermal dielectric of 3 W/mK",
    ),
    m(
        "Generic",
        "Polyester flex (PET)",
        Core::Flex,
        TH_FLEX,
        CU_THIN,
        Sides::Either,
        Some(3.2),
        Some(0.02),
        None,
        "Cheap flexible circuits that do not stand soldering",
    ),
    m(
        "Generic",
        "Polyimide flex",
        Core::Flex,
        TH_FLEX,
        CU_THIN,
        Sides::Either,
        Some(3.4),
        Some(0.002),
        None,
        "Flexible circuits that stand soldering",
    ),
    m(
        "Isola",
        "FR406",
        Core::Fr4,
        TH_FR4,
        CU_STD,
        Sides::Either,
        Some(4.0),
        Some(0.02),
        Some(170.0),
        "Mid-Tg FR-4 for multilayer boards",
    ),
    Material {
        maker: "Isola",
        name: "IS420",
        core: Core::Fr4,
        thickness_mm: TH_TO_24,
        copper_um: CU_ISOLA,
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(4.04),
        df: Some(0.021),
        tg_c: Some(170.0),
        photoresist: false,
        note: "High performance FR-4 to IPC-4101/98/99/101, Td 350 C, T260 60 min, T288 above 15 min, UV blocking and AOI fluorescence; 0.05 to 2.4 mm as a range (the steps shown are the usual ones); 1/2, 1 and 2 oz copper",
    },
    m(
        "Isola",
        "I-Speed",
        Core::Fr4,
        TH_THIN,
        CU_STD,
        Sides::Either,
        Some(3.63),
        Some(0.0071),
        Some(180.0),
        "Low-loss epoxy for fast digital boards",
    ),
    m(
        "Isola",
        "I-Tera MT40",
        Core::Fr4,
        TH_THIN,
        CU_STD,
        Sides::Either,
        Some(3.45),
        Some(0.0031),
        Some(200.0),
        "Very low loss for the fastest links",
    ),
    m(
        "Isola",
        "Tachyon 100G",
        Core::Fr4,
        TH_THIN,
        CU_STD,
        Sides::Either,
        Some(3.02),
        Some(0.0021),
        Some(200.0),
        "Ultra low loss for 100 Gb/s links",
    ),
    m(
        "Isola",
        "Astra MT77",
        Core::Fr4,
        TH_THIN,
        CU_STD,
        Sides::Either,
        Some(3.0),
        Some(0.0017),
        Some(200.0),
        "Low loss with a low dielectric constant for radar",
    ),
    Material {
        maker: "Panasonic",
        name: "R-1566",
        core: Core::Fr4,
        thickness_mm: TH_FR4,
        copper_um: CU_STD,
        sides: Sides::Either,
        formats_mm: &[],
        dk: None,
        df: None,
        tg_c: Some(148.0),
        photoresist: false,
        note: "Halogen-free FR-4 (laminate R-1566, prepreg R-1551), Tg 148 C by DSC, Td 350 C, CTI 600 V, T288 3 min (the S grade has Tg 175 C and Td 355 C; the WN grade T288 10 min); the data sheet lists no Dk, Df, thicknesses, copper weights or sheet sizes (those shown are the usual)",
    },
    Material {
        maker: "Panasonic",
        name: "Megtron 4 (R-5670)",
        core: Core::Fr4,
        thickness_mm: TH_THIN,
        copper_um: CU_STD,
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(3.68),
        df: Some(0.0074),
        tg_c: Some(176.0),
        photoresist: false,
        note: "Low loss material (laminate R-5725, prepreg R-5620); Tg 176 C by DSC, T288 30 min; Dk 3.68 and Df 0.0074 at 13 GHz on a 0.8 mm sample; the data sheet lists no thicknesses, copper weights or sheet sizes (the ones shown are the usual)",
    },
    Material {
        maker: "Panasonic",
        name: "Megtron 7 (R-5785)",
        core: Core::Fr4,
        thickness_mm: TH_THIN,
        copper_um: CU_STD,
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(3.31),
        df: Some(0.0023),
        tg_c: Some(200.0),
        photoresist: false,
        note: "Ultra low loss material (laminate R-5785, prepreg R-5680); Tg 200 C by DSC, T288 above 120 min; Dk 3.31 and Df 0.0023 at 14 GHz with the low Dk glass cloth (3.60 and 0.0034 at 13 GHz with normal glass) on a 0.75 mm sample; the data sheet lists no thicknesses, copper weights or sheet sizes (the ones shown are the usual)",
    },
    Material {
        maker: "Shengyi",
        name: "S1141",
        core: Core::Fr4,
        thickness_mm: TH_TO_32,
        copper_um: CU_12_105,
        sides: Sides::Either,
        formats_mm: SHENGYI_SHEETS,
        dk: Some(4.6),
        df: Some(0.015),
        tg_c: Some(140.0),
        photoresist: false,
        note: "UL ANSI FR-4 with UV blocking, UL 94 V-0, Tg 140 C by DSC (the specification asks 130 C), Td 310 C, CTI PLC 3, not for anti-CAF use; Dk and Df at 1 MHz; 0.05 to 3.2 mm and 12 to 105 um as ranges (the steps shown are the usual ones); UL-recognised boards from 0.38 mm; sheets of 36 x 48, 40 x 48 and 42 x 48 in",
    },
    m(
        "Shengyi",
        "S1170",
        Core::Fr4,
        TH_FR4,
        CU_STD,
        Sides::Either,
        Some(4.3),
        Some(0.018),
        Some(170.0),
        "High-Tg FR-4",
    ),
    m(
        "Shengyi",
        "S1000H",
        Core::Fr4,
        TH_FR4,
        CU_STD,
        Sides::Either,
        Some(4.4),
        Some(0.016),
        Some(150.0),
        "Mid-Tg FR-4 for lead-free soldering",
    ),
    Material {
        maker: "ITEQ",
        name: "IT-158",
        core: Core::Fr4,
        thickness_mm: TH_FR4,
        copper_um: CU_STD,
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(4.3),
        df: Some(0.016),
        tg_c: Some(155.0),
        photoresist: false,
        note: "Medium-Tg filled epoxy to IPC-4101C/99, UL 94 V-0, Tg 155 C typical by DSC (150 C minimum), Td 345 C, T260 above 60 min, T288 above 30 min, CAF resistant; Dk and Df at 1 MHz; the data sheet lists no thicknesses, copper weights or sheet sizes (the ones shown are the usual of FR-4)",
    },
    m(
        "ITEQ",
        "IT-968",
        Core::Fr4,
        TH_THIN,
        CU_STD,
        Sides::Either,
        None,
        None,
        Some(180.0),
        "Low-loss epoxy",
    ),
    m(
        "TUC",
        "TU-752",
        Core::Fr4,
        TH_FR4,
        CU_STD,
        Sides::Either,
        Some(4.4),
        None,
        Some(150.0),
        "Mid-Tg FR-4",
    ),
    m(
        "TUC",
        "TU-768",
        Core::Fr4,
        TH_FR4,
        CU_STD,
        Sides::Either,
        None,
        None,
        Some(180.0),
        "High-Tg FR-4",
    ),
    m(
        "TUC",
        "TU-872 SLK",
        Core::Fr4,
        TH_THIN,
        CU_STD,
        Sides::Either,
        None,
        None,
        Some(200.0),
        "Low-loss epoxy",
    ),
    m(
        "Nanya",
        "NP-140",
        Core::Fr4,
        TH_FR4,
        CU_STD,
        Sides::Either,
        None,
        None,
        Some(140.0),
        "General FR-4",
    ),
    m(
        "Nanya",
        "NP-175",
        Core::Fr4,
        TH_FR4,
        CU_STD,
        Sides::Either,
        None,
        None,
        Some(175.0),
        "High-Tg FR-4",
    ),
    m(
        "KB",
        "KB-6160",
        Core::Fr4,
        TH_FR4,
        CU_STD,
        Sides::Either,
        None,
        None,
        Some(140.0),
        "General FR-4",
    ),
    m(
        "KB",
        "KB-6167",
        Core::Fr4,
        TH_FR4,
        CU_STD,
        Sides::Either,
        None,
        None,
        Some(170.0),
        "High-Tg FR-4",
    ),
    m(
        "Hitachi",
        "MCL-E-679FG",
        Core::Fr4,
        TH_FR4,
        CU_STD,
        Sides::Either,
        None,
        None,
        Some(170.0),
        "Halogen free high-Tg FR-4",
    ),
    m(
        "Ventec",
        "VT-901",
        Core::Polyimide,
        TH_FR4,
        CU_STD,
        Sides::Either,
        Some(4.0),
        None,
        Some(250.0),
        "Polyimide for high temperature",
    ),
    m(
        "Rogers",
        "RO4360G2",
        Core::Hydrocarbon,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(6.15),
        Some(0.0038),
        Some(280.0),
        "High dielectric constant, processed like FR-4",
    ),
    m(
        "Rogers",
        "RO4835",
        Core::Hydrocarbon,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(3.48),
        Some(0.0037),
        Some(280.0),
        "Oxidation resistant RO4350B",
    ),
    m(
        "Rogers",
        "RO4730G3",
        Core::Hydrocarbon,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(3.0),
        Some(0.0029),
        Some(280.0),
        "Light antenna laminate",
    ),
    Material {
        maker: "Rogers",
        name: "RO3006",
        core: Core::Ptfe,
        thickness_mm: &[0.13, 0.25, 0.64, 1.28],
        copper_um: &[17.0, 35.0, 70.0],
        sides: Sides::Either,
        formats_mm: RF_PANEL_SMALL,
        dk: Some(6.15),
        df: Some(0.002),
        tg_c: None,
        photoresist: false,
        note: "Ceramic filled PTFE with a high dielectric constant; Dk 6.15 +/- 0.15 and Df 0.0020 at 10 GHz, Td 500 C, UL 94 V-0; panels of 12 x 18 and 24 x 18 in",
    },
    m(
        "Rogers",
        "RO3035",
        Core::Ptfe,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(3.5),
        Some(0.0017),
        None,
        "Ceramic filled PTFE",
    ),
    m(
        "Rogers",
        "RO3203",
        Core::Ptfe,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(3.02),
        Some(0.0016),
        None,
        "Glass reinforced ceramic filled PTFE",
    ),
    m(
        "Rogers",
        "RT/duroid 5870",
        Core::Ptfe,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(2.33),
        Some(0.0012),
        None,
        "Glass microfibre PTFE",
    ),
    m(
        "Rogers",
        "RT/duroid 6006",
        Core::Ptfe,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(6.15),
        Some(0.0027),
        None,
        "Ceramic PTFE with a high dielectric constant",
    ),
    m(
        "Rogers",
        "RT/duroid 6010.2LM",
        Core::Ptfe,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(10.2),
        Some(0.0023),
        None,
        "Ceramic PTFE with a very high dielectric constant",
    ),
    m(
        "Rogers",
        "TMM 3",
        Core::Hydrocarbon,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(3.27),
        Some(0.002),
        None,
        "Thermoset microwave material",
    ),
    m(
        "Rogers",
        "TMM 4",
        Core::Hydrocarbon,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(4.5),
        Some(0.002),
        None,
        "Thermoset microwave material",
    ),
    m(
        "Rogers",
        "TMM 6",
        Core::Hydrocarbon,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(6.0),
        Some(0.0023),
        None,
        "Thermoset microwave material",
    ),
    m(
        "Rogers",
        "TMM 10",
        Core::Hydrocarbon,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(9.2),
        Some(0.0022),
        None,
        "Thermoset microwave material",
    ),
    m(
        "Rogers",
        "TMM 10i",
        Core::Hydrocarbon,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(9.8),
        Some(0.002),
        None,
        "Thermoset microwave material, tighter tolerance",
    ),
    m(
        "Taconic",
        "RF-60A",
        Core::Ptfe,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(6.15),
        Some(0.0028),
        None,
        "Ceramic filled PTFE with glass",
    ),
    Material {
        maker: "Taconic",
        name: "TLX-8",
        core: Core::Ptfe,
        thickness_mm: &[0.13, 0.48, 0.5, 0.76, 0.78],
        copper_um: CU_ISOLA,
        sides: Sides::Either,
        formats_mm: &[(914.0, 1220.0)],
        dk: Some(2.55),
        df: Some(0.0019),
        tg_c: None,
        photoresist: false,
        note: "PTFE woven glass, UL 94 V-0; Dk 2.55 +/- 0.04 and Df about 0.0019 at 10 GHz; the sheet lists 0.005 to 0.019 in (0.13 to 0.48 mm), 0.020 to 0.030 in (0.50 to 0.76 mm) and 0.031 in (0.78 mm); 1/2, 1 and 2 oz copper; standard sheet 36 x 48 in",
    },
    Material {
        maker: "Taconic",
        name: "TLC-30",
        core: Core::Ptfe,
        thickness_mm: &[0.5, 0.78, 1.5],
        copper_um: CU_ISOLA,
        sides: Sides::Either,
        formats_mm: TACONIC_TLC,
        dk: Some(3.0),
        df: None,
        tg_c: None,
        photoresist: false,
        note: "Low cost RF substrate on PTFE woven glass, UL 94 V-0; Dk 3.00 +/- 0.05 nominal; 0.020 in and 0.030 to 0.062 in; 1/2, 1 and 2 oz copper; the table of the sheet lists 0.37, 0.50, 0.78 to 1.5 and 0.78 mm for the three types TLC-27, TLC-30 and TLC-32",
    },
    m(
        "Taconic",
        "CER-10",
        Core::Ptfe,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(10.0),
        Some(0.0035),
        None,
        "Ceramic PTFE with a high dielectric constant",
    ),
    m(
        "Arlon",
        "25N",
        Core::Hydrocarbon,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(3.38),
        Some(0.0025),
        Some(260.0),
        "Ceramic filled thermoset",
    ),
    m(
        "Arlon",
        "25FR",
        Core::Hydrocarbon,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(3.58),
        Some(0.0035),
        Some(260.0),
        "Flame retardant 25N",
    ),
    m(
        "Arlon",
        "AD250C",
        Core::Ptfe,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(2.5),
        Some(0.0013),
        None,
        "Ceramic PTFE",
    ),
    m(
        "Arlon",
        "AD300D",
        Core::Ptfe,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(2.97),
        Some(0.002),
        None,
        "Ceramic PTFE",
    ),
    m(
        "Arlon",
        "AD350A",
        Core::Ptfe,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(3.5),
        Some(0.003),
        None,
        "Ceramic PTFE",
    ),
    m(
        "Arlon",
        "DiClad 880",
        Core::Ptfe,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(2.2),
        Some(0.0009),
        None,
        "Woven glass PTFE",
    ),
    m(
        "Arlon",
        "DiClad 527",
        Core::Ptfe,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(2.55),
        Some(0.0018),
        None,
        "Woven glass PTFE",
    ),
    m(
        "Arlon",
        "CuClad 217",
        Core::Ptfe,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(2.17),
        Some(0.0009),
        None,
        "Woven glass PTFE",
    ),
    m(
        "Arlon",
        "CLTE-XT",
        Core::Ptfe,
        TH_RF,
        CU_RF,
        Sides::Either,
        Some(2.79),
        Some(0.0012),
        None,
        "Ceramic woven glass PTFE",
    ),
    m(
        "Arlon",
        "35N polyimide",
        Core::Polyimide,
        TH_FR4,
        CU_STD,
        Sides::Either,
        Some(4.2),
        Some(0.012),
        Some(250.0),
        "Polyimide for high temperature",
    ),
    m(
        "Nelco",
        "N4000-13EP",
        Core::Fr4,
        TH_THIN,
        CU_STD,
        Sides::Either,
        Some(3.7),
        Some(0.009),
        Some(210.0),
        "Low-loss epoxy",
    ),
    m(
        "Nelco",
        "N4000-6",
        Core::Fr4,
        TH_FR4,
        CU_STD,
        Sides::Either,
        Some(4.2),
        None,
        Some(170.0),
        "High-Tg FR-4",
    ),
    m(
        "Nelco",
        "N4000-29",
        Core::Fr4,
        TH_FR4,
        CU_STD,
        Sides::Either,
        Some(4.3),
        None,
        Some(185.0),
        "High-Tg epoxy for many layers",
    ),
    m(
        "DuPont",
        "Pyralux AC adhesiveless",
        Core::Flex,
        TH_FLEX,
        CU_THIN,
        Sides::Either,
        Some(3.4),
        Some(0.003),
        None,
        "Polyimide flex without adhesive",
    ),
    m(
        "Bergquist",
        "Thermal Clad HT",
        Core::Aluminium,
        TH_METAL,
        CU_METAL,
        Sides::One,
        None,
        None,
        None,
        "Insulated metal substrate with a high thermal conductivity",
    ),
    m(
        "Bergquist",
        "Thermal Clad LTI",
        Core::Aluminium,
        TH_METAL,
        CU_METAL,
        Sides::One,
        None,
        None,
        None,
        "Insulated metal substrate with a low thermal impedance",
    ),
    m(
        "Totking",
        "Aluminium IMS",
        Core::Aluminium,
        TH_METAL,
        CU_METAL,
        Sides::One,
        None,
        None,
        None,
        "Aluminium boards for LED lamps",
    ),
    // ---- from the data sheets of the makers
    Material {
        maker: "Shengyi",
        name: "S1000-2M",
        core: Core::Fr4,
        thickness_mm: TH_TO_32,
        copper_um: CU_12_105,
        sides: Sides::Either,
        formats_mm: SHENGYI_SHEETS,
        dk: Some(4.6),
        df: Some(0.018),
        tg_c: Some(180.0),
        photoresist: false,
        note: "UL ANSI FR-4.0, anti-CAF, lead-free; Tg 180 C by DSC (185 C by DMA), Td 355 C, T260 above 60 min, T288 30 min, T300 15 min, CTI PLC 3, UL 94 V-0; Dk and Df at 1 GHz on a 1.6 mm sample; 0.05 to 3.2 mm and 12 to 105 um as ranges (the steps shown are the usual ones)",
    },
    Material {
        maker: "Shengyi",
        name: "S7136H",
        core: Core::Hydrocarbon,
        thickness_mm: &[0.254, 0.51, 0.76, 1.52],
        copper_um: CU_RF,
        sides: Sides::Either,
        formats_mm: SHENGYI_SHEETS,
        dk: Some(3.42),
        df: Some(0.003),
        tg_c: Some(280.0),
        photoresist: false,
        note: "Hydrocarbon woven glass ceramic, UL 94 V-0, made with the FR-4 processes; Dk 3.42 +/- 0.05 (process) and 3.61 (design) and Df 0.0030 at 10 GHz; Tg above 280 C by DSC, Td 390 C; 0.010, 0.020, 0.030 and 0.060 in; 1/2 and 1 oz copper; sheets of 36 x 48, 40 x 48 and 42 x 48 in",
    },
    Material {
        maker: "Shengyi",
        name: "SCGA-500 GF220",
        core: Core::Ptfe,
        thickness_mm: &[0.254, 0.508, 0.762, 1.524],
        copper_um: CU_RF,
        sides: Sides::Either,
        formats_mm: SHENGYI_SHEETS_2,
        dk: Some(2.2),
        df: Some(0.0009),
        tg_c: None,
        photoresist: false,
        note: "PTFE woven glass, UL 94 V-0; Dk 2.20 +/- 0.02 and Df 0.0009 at 10 GHz; 0.010, 0.020, 0.030 and 0.060 in; 1/2 and 1 oz copper; sheets of 36 x 48 and 40 x 48 in",
    },
    Material {
        maker: "Shengyi",
        name: "SCGA-500 GF255",
        core: Core::Ptfe,
        thickness_mm: &[0.254, 0.508, 0.762, 1.524],
        copper_um: CU_RF,
        sides: Sides::Either,
        formats_mm: SHENGYI_SHEETS_2,
        dk: Some(2.55),
        df: Some(0.0014),
        tg_c: None,
        photoresist: false,
        note: "PTFE woven glass, UL 94 V-0; Dk 2.55 +/- 0.04 and Df 0.0014 at 10 GHz; 0.010, 0.020, 0.030 and 0.060 in; 1/2 and 1 oz copper; sheets of 36 x 48 and 40 x 48 in",
    },
    Material {
        maker: "Shengyi",
        name: "SCGA-500 GF265",
        core: Core::Ptfe,
        thickness_mm: &[0.254, 0.508, 0.762, 1.524],
        copper_um: CU_RF,
        sides: Sides::Either,
        formats_mm: SHENGYI_SHEETS_2,
        dk: Some(2.65),
        df: Some(0.0017),
        tg_c: None,
        photoresist: false,
        note: "PTFE woven glass, UL 94 V-0; Dk 2.65 +/- 0.04 and Df 0.0017 at 10 GHz; 0.010, 0.020, 0.030 and 0.060 in; 1/2 and 1 oz copper; sheets of 36 x 48 and 40 x 48 in",
    },
    Material {
        maker: "Shengyi",
        name: "SCGA-500 GF300",
        core: Core::Ptfe,
        thickness_mm: &[0.254, 0.508, 0.762, 1.524],
        copper_um: CU_RF,
        sides: Sides::Either,
        formats_mm: SHENGYI_SHEETS_2,
        dk: Some(3.0),
        df: Some(0.0023),
        tg_c: None,
        photoresist: false,
        note: "PTFE woven glass, UL 94 V-0; Dk 3.00 +/- 0.04 and Df 0.0023 at 10 GHz; 0.010, 0.020, 0.030 and 0.060 in; 1/2 and 1 oz copper; sheets of 36 x 48 and 40 x 48 in",
    },
    Material {
        maker: "Shengyi",
        name: "AeroWave 300",
        core: Core::Hydrocarbon,
        thickness_mm: &[0.127, 0.254, 0.508, 0.762, 1.524],
        copper_um: &[35.0],
        sides: Sides::Either,
        formats_mm: SHENGYI_SHEETS,
        dk: Some(3.0),
        df: Some(0.0031),
        tg_c: Some(200.0),
        photoresist: false,
        note: "Thermoset low loss RF laminate, UL 94 V-0; Dk 3.0 +/- 0.05 (design 2.98) and Df 0.0031 at 10 GHz; Tg 200 C by DMA, Td 400 C; 0.005 to 0.060 in; HVLP low profile copper (the sheet gives the peel strength for 1 oz and lists no weights); sheets of 36 x 48, 40 x 48 and 42 x 48 in",
    },
    Material {
        maker: "Shengyi",
        name: "mmWave77",
        core: Core::Ptfe,
        thickness_mm: &[0.127, 0.254, 0.508, 0.762],
        copper_um: &[18.0, 35.0],
        sides: Sides::Either,
        formats_mm: &[(457.0, 610.0), (534.0, 610.0)],
        dk: Some(3.0),
        df: Some(0.001),
        tg_c: None,
        photoresist: false,
        note: "UL 94 V-0; Dk 3.0 +/- 0.04 and Df 0.0010 at 10 GHz, stable from 1 to 100 GHz, Td 538 C by TGA; 0.005 to 0.030 in; 1/2 and 1 oz HVLP copper; sheets of 18 x 24 and 21 x 24 in; the sheet does not name the resin",
    },
    Material {
        maker: "Shengyi",
        name: "mmWave",
        core: Core::Hydrocarbon,
        thickness_mm: &[0.127, 0.254, 0.508, 0.762, 1.524],
        copper_um: &[18.0, 35.0, 70.0],
        sides: Sides::Either,
        formats_mm: &[(457.0, 305.0), (457.0, 610.0), (915.0, 1220.0), (1067.0, 1220.0)],
        dk: Some(3.0),
        df: Some(0.002),
        tg_c: Some(190.0),
        photoresist: false,
        note: "PPE woven glass, UL 94 V-0; Dk 3.00 +/- 0.05 (design 3.064) and Df 0.002 at 10 GHz; Tg 190 C by TMA, Td 400 C; 0.005 to 0.060 in; 1/2 and 1 oz HVLP copper and 2 oz RTF; sheets of 18 x 12, 18 x 24, 36 x 48 and 42 x 48 in",
    },
    Material {
        maker: "Shengyi",
        name: "SF305",
        core: Core::Flex,
        thickness_mm: &[0.0125, 0.025, 0.05],
        copper_um: CU_THIN,
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(3.6),
        df: Some(0.03),
        tg_c: None,
        photoresist: false,
        note: "Halogen-free three-layer flexible copper clad laminate, UL 94 V-0; polyimide film of 12.5, 25 and 50 um with 13 or 20 um adhesive; copper 12, 18 and 35 um, rolled or electrodeposited, single or double sided; Dk 3.6 and Df 0.030 at 1 MHz; rolls 250 or 500 mm wide",
    },
    Material {
        maker: "Isola",
        name: "IS400",
        core: Core::Fr4,
        thickness_mm: TH_TO_24,
        copper_um: CU_ISOLA,
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(3.9),
        df: Some(0.022),
        tg_c: Some(150.0),
        photoresist: false,
        note: "Temperature resistant epoxy to IPC-4101/97/98/99/101, Td 330 C, T260 above 60 min, T288 above 10 min, CAF resistant, EN 45545-2 and NF F 16-101 rail classes; Dk and Df at 500 MHz; 0.05 to 2.4 mm as a range (the steps shown are the usual ones); 1/2 to 2 oz copper",
    },
    Material {
        maker: "Isola",
        name: "IS620i",
        core: Core::Fr4,
        thickness_mm: TH_TO_15,
        copper_um: CU_ISOLA,
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(3.58),
        df: Some(0.006),
        tg_c: Some(225.0),
        photoresist: false,
        note: "Low loss epoxy for 2 to 15 GHz to IPC-4101/135, Td 364 C, T260 60 min, T288 above 20 min; 0.05 to 1.5 mm as a range (the steps shown are the usual ones); 1/2, 1 and 2 oz copper",
    },
    Material {
        maker: "Isola",
        name: "DE104",
        core: Core::Fr4,
        thickness_mm: TH_TO_24,
        copper_um: CU_ISOLA,
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(4.37),
        df: Some(0.022),
        tg_c: Some(135.0),
        photoresist: false,
        note: "Duraver FR-4 to IPC-4101/21, Td 315 C, T260 12 min, CAF resistant, EN 45545-2 certified; 0.05 to 2.4 mm as a range (the steps shown are the usual ones); 1/2 to 2 oz copper",
    },
    Material {
        maker: "Isola",
        name: "DE104 KF",
        core: Core::Fr4,
        thickness_mm: TH_TO_24,
        copper_um: CU_ISOLA,
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(4.0),
        df: Some(0.022),
        tg_c: Some(135.0),
        photoresist: false,
        note: "Track resistant FR-4 (CTI 400) to IPC-4101/21, UL 94 V-0, Td 315 C; Dk and Df at 1 MHz; 0.05 to 2.4 mm as a range (the steps shown are the usual ones); 1/2 to 2 oz copper",
    },
    Material {
        maker: "Isola",
        name: "DE156",
        core: Core::Fr4,
        thickness_mm: TH_TO_15,
        copper_um: CU_ISOLA,
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(4.0),
        df: Some(0.02),
        tg_c: Some(155.0),
        photoresist: false,
        note: "Halogen-free FR-4 to IPC-4101/94, UL 94 V-0 without antimony, Td 390 C, T260 60 min, T288 above 10 min; 0.05 to 1.5 mm as a range (the steps shown are the usual ones); 1/2 to 2 oz copper",
    },
    Material {
        maker: "Ventec",
        name: "VT-4A1",
        core: Core::Aluminium,
        thickness_mm: &[0.6, 0.8, 1.0, 1.2, 1.5, 2.0, 3.0],
        copper_um: &[18.0, 35.0, 70.0, 105.0, 140.0, 210.0, 350.0],
        sides: Sides::One,
        formats_mm: &[(457.0, 610.0), (508.0, 610.0), (533.0, 610.0), (457.0, 1220.0), (508.0, 1220.0), (533.0, 1220.0)],
        dk: Some(5.0),
        df: Some(0.015),
        tg_c: Some(130.0),
        photoresist: false,
        note: "Aluminium base laminate, dielectric of 75 to 200 um, thermal conductivity 1.6 W/mK, Td 380 C, CTI 600 V, UL 94 V-0, halogen-free; Dk and Df at 1 MHz; aluminium 0.6 to 3.0 mm, copper 1/2 to 10 oz; sheets of 18, 20 and 21 x 24 and 48 in",
    },
    Material {
        maker: "Ventec",
        name: "VT-4A2",
        core: Core::Aluminium,
        thickness_mm: &[0.6, 0.8, 1.0, 1.2, 1.5, 2.0, 3.0],
        copper_um: &[18.0, 35.0, 70.0, 105.0, 140.0, 210.0, 350.0],
        sides: Sides::One,
        formats_mm: &[(457.0, 610.0), (508.0, 610.0), (533.0, 610.0), (457.0, 1220.0), (508.0, 1220.0), (533.0, 1220.0)],
        dk: Some(5.1),
        df: Some(0.014),
        tg_c: Some(130.0),
        photoresist: false,
        note: "Aluminium base laminate, dielectric of 75 to 200 um, thermal conductivity 2.2 W/mK, Td 380 C, CTI 600 V, UL 94 V-0, halogen-free; Dk and Df at 1 MHz; aluminium 0.6 to 3.0 mm, copper 1/2 to 10 oz; sheets of 18, 20 and 21 x 24 and 48 in",
    },
    Material {
        maker: "Ventec",
        name: "VT-4A2H",
        core: Core::Aluminium,
        thickness_mm: &[0.6, 0.8, 1.0, 1.2, 1.5, 2.0, 3.0],
        copper_um: &[18.0, 35.0, 70.0, 105.0, 140.0, 210.0, 350.0],
        sides: Sides::One,
        formats_mm: &[(457.0, 610.0), (508.0, 610.0), (533.0, 610.0), (457.0, 1220.0), (508.0, 1220.0), (533.0, 1220.0)],
        dk: Some(5.1),
        df: Some(0.014),
        tg_c: Some(130.0),
        photoresist: false,
        note: "Aluminium base laminate, dielectric of 75 to 200 um, thermal conductivity 2.2 W/mK, Td 380 C, CTI 600 V, UL 94 V-0, halogen-free; Dk and Df at 1 MHz; aluminium 0.6 to 3.0 mm, copper 1/2 to 10 oz; sheets of 18, 20 and 21 x 24 and 48 in",
    },
    Material {
        maker: "Ventec",
        name: "VT-4B1",
        core: Core::Aluminium,
        thickness_mm: &[0.6, 0.8, 1.0, 1.2, 1.5],
        copper_um: &[18.0, 35.0, 70.0],
        sides: Sides::One,
        formats_mm: &[(460.0, 610.0), (510.0, 610.0), (533.0, 610.0), (460.0, 1220.0), (510.0, 1220.0), (533.0, 1220.0)],
        dk: Some(4.8),
        df: Some(0.016),
        tg_c: Some(100.0),
        photoresist: false,
        note: "Aluminium base laminate for bending (3D lighting), ceramic filled dielectric of 50 to 100 um, thermal conductivity 1.0 W/mK, Td 380 C, CTI 600 V, UL 94 V-0, halogen-free; Dk and Df at 1 MHz; aluminium 0.6 to 1.5 mm, copper 1/2 to 2 oz",
    },
    Material {
        maker: "Ventec",
        name: "VT-4B3",
        core: Core::Aluminium,
        thickness_mm: &[0.6, 0.8, 1.0, 1.2, 1.5, 2.0, 3.0],
        copper_um: &[12.0, 18.0, 35.0, 70.0, 105.0, 140.0, 210.0],
        sides: Sides::One,
        formats_mm: &[(457.0, 610.0), (508.0, 610.0), (533.0, 610.0), (457.0, 1220.0), (508.0, 1220.0), (533.0, 1220.0)],
        dk: Some(4.8),
        df: Some(0.016),
        tg_c: Some(130.0),
        photoresist: false,
        note: "Aluminium base laminate, ceramic filled dielectric of 50 to 230 um, thermal conductivity 3.0 W/mK, Td 380 C, CTI 600 V, UL 94 V-0, halogen-free; Dk and Df at 1 MHz; aluminium 0.6 to 3.0 mm, copper 1/3 to 6 oz",
    },
    Material {
        maker: "Yuxinda",
        name: "AL-200",
        core: Core::Aluminium,
        thickness_mm: &[0.2, 1.0, 1.5, 2.0, 3.0, 6.0],
        copper_um: &[12.0, 18.0, 35.0, 70.0, 105.0, 140.0, 210.0],
        sides: Sides::One,
        formats_mm: &[(1000.0, 1200.0), (1080.0, 1240.0)],
        dk: Some(4.1),
        df: Some(0.018),
        tg_c: None,
        photoresist: false,
        note: "Aluminium copper clad, thermal conductivity 2.05 W/mK typical, CTI 600 V, UL 94 V-0, halogen- and sulphur-free; Dk at 1 MHz and Df at most 0.018; the thickness is 0.2 to 6.0 mm and the copper 1/3 to 6 oz as ranges (the steps shown are the usual ones)",
    },
    Material {
        maker: "Yuxinda",
        name: "AL-300",
        core: Core::Aluminium,
        thickness_mm: &[0.2, 1.0, 1.5, 2.0, 3.0, 6.0],
        copper_um: &[12.0, 18.0, 35.0, 70.0, 105.0, 140.0, 210.0],
        sides: Sides::One,
        formats_mm: &[(1000.0, 1200.0), (1080.0, 1240.0)],
        dk: Some(4.1),
        df: Some(0.018),
        tg_c: None,
        photoresist: false,
        note: "Aluminium copper clad, thermal conductivity 3.01 W/mK typical, CTI 600 V, UL 94 V-0, halogen- and sulphur-free; Dk at 1 MHz and Df at most 0.018; the thickness is 0.2 to 6.0 mm and the copper 1/3 to 6 oz as ranges (the steps shown are the usual ones)",
    },
    Material {
        maker: "MSC",
        name: "Polytherm TC-Lam 2.0",
        core: Core::Aluminium,
        thickness_mm: &[0.5, 1.0, 1.5, 2.0, 3.0],
        copper_um: &[18.0, 35.0, 70.0, 105.0, 140.0, 210.0],
        sides: Sides::One,
        formats_mm: &[(480.0, 580.0), (480.0, 600.0), (460.0, 610.0), (530.0, 630.0)],
        dk: Some(6.5),
        df: None,
        tg_c: Some(100.0),
        photoresist: false,
        note: "Insulated metal substrate, aluminium alloy 5052 H34 or 1100 H24, dielectric of 75 to 200 um, thermal conductivity 2.0 W/mK, dielectric strength 5 kV, UL 94 V-0, CTI PLC 0; Dk at 1 MHz; aluminium 0.5 to 3.0 mm, copper 18 to 210 um",
    },
    Material {
        maker: "ThinFlex",
        name: "W-0505ED-N4",
        core: Core::Flex,
        thickness_mm: &[0.0127],
        copper_um: &[18.0],
        sides: Sides::Two,
        formats_mm: &[],
        dk: Some(3.3),
        df: Some(0.01),
        tg_c: None,
        photoresist: false,
        note: "Adhesiveless double-sided flexible laminate, halogen-free, polyimide film of 0.5 mil with 1/2 oz electrodeposited copper on both sides, UL 94 V-0; Dk and Df at 1 GHz; sheets 250 or 500 mm wide and 400 to 700 mm long",
    },
    Material {
        maker: "Panasonic",
        name: "FELIOS RF775",
        core: Core::Flex,
        thickness_mm: &[0.0127, 0.0147, 0.0203, 0.0254, 0.0508, 0.0762, 0.1016],
        copper_um: &[9.0, 12.0, 18.0, 35.0, 70.0],
        sides: Sides::Two,
        formats_mm: &[],
        dk: Some(3.2),
        df: Some(0.002),
        tg_c: Some(343.0),
        photoresist: false,
        note: "Adhesiveless double-sided flexible laminate, all polyimide, halogen- and antimony-free, UL 94 V-0, 160 C UL operating temperature, IPC-4204/11; Dk and Df at 1 MHz; film of 0.0005 to 0.004 in with rolled copper of 1/4, 1/3, 1/2, 1 or 2 oz depending on the film",
    },
    Material {
        maker: "Taconic",
        name: "TLC-27",
        core: Core::Ptfe,
        thickness_mm: &[0.37],
        copper_um: CU_ISOLA,
        sides: Sides::Either,
        formats_mm: TACONIC_TLC,
        dk: Some(2.75),
        df: None,
        tg_c: None,
        photoresist: false,
        note: "Low cost RF substrate on PTFE woven glass, UL 94 V-0; Dk 2.75 +/- 0.05 nominal; 0.0145 in; 1/2, 1 and 2 oz copper; the table of the sheet lists 0.37, 0.50, 0.78 to 1.5 and 0.78 mm for the three types TLC-27, TLC-30 and TLC-32",
    },
    Material {
        maker: "Taconic",
        name: "TLC-32",
        core: Core::Ptfe,
        thickness_mm: &[0.78],
        copper_um: CU_ISOLA,
        sides: Sides::Either,
        formats_mm: TACONIC_TLC,
        dk: Some(3.2),
        df: Some(0.003),
        tg_c: None,
        photoresist: false,
        note: "Low cost RF substrate on PTFE woven glass, UL 94 V-0; Dk 3.20 +/- 0.05 and Df 0.0030 at 10 GHz; 0.030 in; 1/2, 1 and 2 oz copper",
    },
    Material {
        maker: "Taconic",
        name: "TLX-9",
        core: Core::Ptfe,
        thickness_mm: &[0.13, 0.48, 0.5, 0.76, 0.78],
        copper_um: CU_ISOLA,
        sides: Sides::Either,
        formats_mm: &[(914.0, 1220.0)],
        dk: Some(2.5),
        df: Some(0.0019),
        tg_c: None,
        photoresist: false,
        note: "PTFE woven glass, UL 94 V-0; Dk 2.50 +/- 0.04 and Df 0.0019 at 10 GHz; the sheet lists 0.005 to 0.019 in (0.13 to 0.48 mm), 0.020 to 0.030 in (0.50 to 0.76 mm) and 0.031 in (0.78 mm); 1/2, 1 and 2 oz copper; standard sheet 36 x 48 in",
    },
    Material {
        maker: "Panasonic",
        name: "Megtron 8 (R-579Y)",
        core: Core::Fr4,
        thickness_mm: &[0.75],
        copper_um: &[35.0],
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(3.08),
        df: Some(0.0012),
        tg_c: Some(220.0),
        photoresist: false,
        note: "Ultra low loss material (laminate R-579Y, prepreg R-569Y); Tg 220 C by DMA, T288 above 120 min; Dk 3.08 and Df 0.0012 at 14 GHz with the low Dk glass cloth on a 0.75 mm sample; the data sheet lists only that sample and 1 oz H-VLP3 copper",
    },
    Material {
        maker: "Panasonic",
        name: "Megtron 2 (R-1577)",
        core: Core::Fr4,
        thickness_mm: &[0.8],
        copper_um: &[35.0],
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(4.0),
        df: Some(0.013),
        tg_c: Some(170.0),
        photoresist: false,
        note: "Halogen-free low loss, lead-free soldering (laminate R-1577, prepreg R-1570); Tg 170 C by DSC, T288 25 min; Dk and Df at 10 GHz; the data sheet lists only the 0.8 mm sample and 1 oz copper",
    },
    Material {
        maker: "Panasonic",
        name: "R-1755V",
        core: Core::Fr4,
        thickness_mm: &[0.8],
        copper_um: &[35.0],
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(4.4),
        df: Some(0.016),
        tg_c: Some(173.0),
        photoresist: false,
        note: "Highly heat resistant low CTE FR-4 (laminate R-1755V), Tg 173 C by DSC, UL 94 V-0 class FR-4; Dk 4.4 and Df 0.016 at 1 GHz; T288 20 min; the data sheet lists only the 0.8 mm sample and 1 oz copper",
    },
    Material {
        maker: "Panasonic",
        name: "R-1755D",
        core: Core::Fr4,
        thickness_mm: &[0.8],
        copper_um: &[35.0],
        sides: Sides::Either,
        formats_mm: &[],
        dk: None,
        df: None,
        tg_c: Some(163.0),
        photoresist: false,
        note: "Highly heat resistant low CTE FR-4 (laminate R-1755D), Tg 163 C by DSC, Td 345 C, UL 94 V-0 class FR-4; the data sheet lists only the 0.8 mm sample and 1 oz copper",
    },
    Material {
        maker: "Panasonic",
        name: "R-1755M",
        core: Core::Fr4,
        thickness_mm: &[0.8],
        copper_um: &[35.0],
        sides: Sides::Either,
        formats_mm: &[],
        dk: None,
        df: None,
        tg_c: Some(153.0),
        photoresist: false,
        note: "Highly heat resistant low CTE FR-4 (laminate R-1755M), Tg 153 C by DSC, Td 355 C, UL 94 V-0 class FR-4; the data sheet lists only the 0.8 mm sample and 1 oz copper",
    },
    Material {
        maker: "Panasonic",
        name: "R-1755E",
        core: Core::Fr4,
        thickness_mm: &[0.8],
        copper_um: &[35.0],
        sides: Sides::Either,
        formats_mm: &[],
        dk: None,
        df: None,
        tg_c: Some(133.0),
        photoresist: false,
        note: "Highly heat resistant low CTE FR-4 (laminate R-1755E), Tg 133 C by DSC, Td 370 C, UL 94 V-0 class FR-4; the data sheet lists only the 0.8 mm sample and 1 oz copper",
    },
    Material {
        maker: "Panasonic",
        name: "R-1785",
        core: Core::Cem3,
        thickness_mm: &[0.8, 1.6],
        copper_um: &[35.0],
        sides: Sides::Two,
        formats_mm: &[],
        dk: None,
        df: None,
        tg_c: Some(150.0),
        photoresist: false,
        note: "High reliability CEM-3 glass composite, double sided, CTI 600 V, Tg 150 C by TMA; better solder joint life and CAF resistance than usual CEM-3; the data sheet gives results for 1.6 mm (0.8 mm in brackets) boards with 35 um copper",
    },
    Material {
        maker: "Panasonic",
        name: "R-1786",
        core: Core::Cem3,
        thickness_mm: &[0.8, 1.6],
        copper_um: &[18.0, 35.0, 70.0],
        sides: Sides::Either,
        formats_mm: &[],
        dk: Some(4.0),
        df: Some(0.007),
        tg_c: Some(140.0),
        photoresist: false,
        note: "Glass composite (R-1786 double sided, R-1781 single sided), CTI 600 V, UL 94 V-0, Tg 140 C; Dk 4.0 and Df 0.007 at 1 GHz (4.2 and 0.011 at 1 MHz); 18 and 35 um copper, and the thick 70 um copper type for high currents; the data sheets give results for 1.6 mm (0.8 mm in brackets) boards",
    },
    Material {
        maker: "Panasonic",
        name: "R-1787",
        core: Core::Cem3,
        thickness_mm: &[1.6],
        copper_um: &[35.0],
        sides: Sides::Two,
        formats_mm: &[],
        dk: None,
        df: None,
        tg_c: Some(140.0),
        photoresist: false,
        note: "High thermal conductivity glass composite (CEM-3 grade) for LED boards, double sided, thermal conductivity 1.10 W/mK, CTI 600 V, Tg 140 C by TMA; the data sheet gives results for 1.6 mm boards with 1 oz copper",
    },
    // ---- sheet and plate without copper, to mill and to drill
    m(
        "Generic",
        "Aluminium sheet (5052 / 6061)",
        Core::Metal,
        TH_ALUMINIUM_SHEET,
        &[],
        Sides::NoCopper,
        None,
        None,
        None,
        "Alloy sheet 5052 (easy to form) or 6061 (stiffer, machines better); Usual thicknesses, sold cut to size; no copper, no electrical figures given (they depend on the grade); slow turns, a lubricant and shallow passes; clamp it flat and remove the chips",
    ),
    m(
        "Generic",
        "Copper sheet (C110)",
        Core::Copper,
        TH_COPPER_SHEET,
        &[],
        Sides::NoCopper,
        None,
        None,
        None,
        "Pure copper sheet C110 (ETP); Usual thicknesses, sold cut to size; no copper, no electrical figures given (they depend on the grade); soft and gummy, so low turns, a moderate feed, shallow passes and a lubricant; use sharp single-flute tools",
    ),
    m(
        "Generic",
        "Birch plywood",
        Core::Wood,
        TH_WOOD,
        &[],
        Sides::NoCopper,
        None,
        None,
        None,
        "Birch plywood with a thin cross-grained layup; Usual thicknesses, sold cut to size; no copper, no electrical figures given (they depend on the grade); high turns with a single-flute router, a clean exit side needs a sacrificial board; the dust is to be extracted",
    ),
    m(
        "Generic",
        "MDF",
        Core::Wood,
        TH_MDF,
        &[],
        Sides::NoCopper,
        None,
        None,
        None,
        "Medium density fibreboard; Usual thicknesses, sold cut to size; no copper, no electrical figures given (they depend on the grade); machines clean and dulls the tools quickly; the dust is fine and must be extracted",
    ),
    m(
        "Generic",
        "Balsa wood",
        Core::Wood,
        TH_BALSA,
        &[],
        Sides::NoCopper,
        None,
        None,
        None,
        "Balsa sheet for models and mock-ups; Usual thicknesses, sold cut to size; no copper, no electrical figures given (they depend on the grade); very soft: high turns, a fast feed and a sharp tool, held down gently",
    ),
    m(
        "Generic",
        "Basswood (lime)",
        Core::Wood,
        TH_WOOD,
        &[],
        Sides::NoCopper,
        None,
        None,
        None,
        "Basswood (lime) sheet for models; Usual thicknesses, sold cut to size; no copper, no electrical figures given (they depend on the grade); soft and even grained, cuts clean at high turns",
    ),
    m(
        "Generic",
        "PTFE sheet",
        Core::Ptfe,
        TH_PTFE_SHEET,
        &[],
        Sides::NoCopper,
        None,
        None,
        None,
        "Solid PTFE sheet (virgin), not the glass laminate for circuits; Usual thicknesses, sold cut to size; no copper, no electrical figures given (they depend on the grade); soft and slippery, it moves under the clamp and does not take glue; do not let the tool heat it",
    ),
    m(
        "Generic",
        "Acrylic (PMMA)",
        Core::Plastic,
        TH_PLASTIC,
        &[],
        Sides::NoCopper,
        None,
        None,
        None,
        "Cast or extruded acrylic sheet; Usual thicknesses, sold cut to size; no copper, no electrical figures given (they depend on the grade); single-flute tools cut it clean; too many turns with a slow feed melt it and it chips if the tool is dull",
    ),
    m(
        "Generic",
        "Polycarbonate (PC)",
        Core::Plastic,
        TH_PLASTIC_THIN,
        &[],
        Sides::NoCopper,
        None,
        None,
        None,
        "Polycarbonate sheet; Usual thicknesses, sold cut to size; no copper, no electrical figures given (they depend on the grade); tough, it does not chip but it softens with heat: sharp tools and a steady feed",
    ),
    m(
        "Generic",
        "ABS",
        Core::SoftPlastic,
        TH_PLASTIC,
        &[],
        Sides::NoCopper,
        None,
        None,
        None,
        "ABS sheet; Usual thicknesses, sold cut to size; no copper, no electrical figures given (they depend on the grade); it softens and sticks to the tool when it heats: moderate turns and a fast feed",
    ),
    m(
        "Generic",
        "POM",
        Core::Plastic,
        TH_PLASTIC,
        &[],
        Sides::NoCopper,
        None,
        None,
        None,
        "Acetal copolymer or homopolymer (POM, sold as Delrin and other trade names); Usual thicknesses, sold cut to size; no copper, no electrical figures given (they depend on the grade); machines very clean with sharp tools",
    ),
    m(
        "Generic",
        "Nylon (PA6)",
        Core::SoftPlastic,
        TH_PLASTIC_THICK,
        &[],
        Sides::NoCopper,
        None,
        None,
        None,
        "Polyamide 6 sheet; Usual thicknesses, sold cut to size; no copper, no electrical figures given (they depend on the grade); it takes up water and moves, and it leaves burrs: sharp tools, moderate turns and a fast feed",
    ),
    m(
        "Generic",
        "PVC",
        Core::Plastic,
        TH_PLASTIC_THIN,
        &[],
        Sides::NoCopper,
        None,
        None,
        None,
        "Rigid PVC sheet; Usual thicknesses, sold cut to size; no copper, no electrical figures given (they depend on the grade); it cuts clean but gives off harmful fumes when it overheats: keep it cool, extract the dust and the fumes",
    ),
    m(
        "Generic",
        "HDPE",
        Core::SoftPlastic,
        TH_PLASTIC_THICK,
        &[],
        Sides::NoCopper,
        None,
        None,
        None,
        "High density polyethylene sheet; Usual thicknesses, sold cut to size; no copper, no electrical figures given (they depend on the grade); waxy and soft, it melts and rolls up into strings: a fast feed and a sharp single-flute tool",
    ),
    m(
        "Generic",
        "PETG",
        Core::SoftPlastic,
        TH_PLASTIC_THIN,
        &[],
        Sides::NoCopper,
        None,
        None,
        None,
        "PETG sheet; Usual thicknesses, sold cut to size; no copper, no electrical figures given (they depend on the grade); tough and it melts when it heats: moderate turns and a fast feed",
    ),
    m(
        "Generic",
        "G10 (FR4 without copper)",
        Core::Fr4,
        TH_G10,
        &[],
        Sides::NoCopper,
        None,
        None,
        None,
        "Glass-epoxy laminate (G10 or FR4) with no copper, for jigs, spacers and fixtures; Usual thicknesses, sold cut to size; no copper, no electrical figures given (they depend on the grade); the glass wears the tools like the board material does, and the dust must be extracted",
    ),
    m(
        "Generic",
        "Phenolic paper (Pertinax)",
        Core::Fr2,
        TH_PERTINAX,
        &[],
        Sides::NoCopper,
        None,
        None,
        None,
        "Paper and phenolic resin laminate (FR2 grade, known as Pertinax) with no copper; Usual thicknesses, sold cut to size; no copper, no electrical figures given (they depend on the grade); soft and easy to cut, it frays at the exit side without a backing board",
    ),
];

/// The makers in the library, once each, in the order of the list.
impl Material {
    /// The text with the name of this material in the languages of the interface, when the name has words (the names that are only a designation, such as RO4003C, are the same everywhere).
    pub fn name_text(&self) -> Option<eh3d_view::texts::Text> {
        use eh3d_view::texts::Text;
        Some(match self.name {
            "FR4 epoxy, positive photoresist" => Text::MatFr4PositiveResist,
            "FR4 epoxy, bare copper" => Text::MatFr4Bare,
            "Laminated paper (FR2), photoresist" => Text::MatPaperResist,
            "FR2 / FR3 / CEM-1 sheets" => Text::MatSheets,
            "FR4 500 series, 1.6 mm" => Text::MatFr4Series16,
            "FR4 500 series, 0.8 mm" => Text::MatFr4Series08,
            "FR4 600 series, positive presensitized" => Text::MatFr4Series600,
            "Thermal Clad (aluminium)" => Text::MatThermalClad,
            "Pyralux AP flexible" => Text::MatPyralux,
            "Aluminium sheet (5052 / 6061)" => Text::MatAluminiumSheet,
            "Copper sheet (C110)" => Text::MatCopperSheet,
            "Birch plywood" => Text::MatBirchPlywood,
            "Balsa wood" => Text::MatBalsa,
            "Basswood (lime)" => Text::MatBasswood,
            "PTFE sheet" => Text::MatPtfeSheet,
            "Acrylic (PMMA)" => Text::MatAcrylic,
            "Polycarbonate (PC)" => Text::MatPolycarbonate,
            "Nylon (PA6)" => Text::MatNylon,
            "G10 (FR4 without copper)" => Text::MatG10Plain,
            "Phenolic paper (Pertinax)" => Text::MatPhenolicPaper,
            _ => return None,
        })
    }
}

/// The text of a maker that is a word and not a name ("Generic").
pub fn maker_text(maker: &str) -> Option<eh3d_view::texts::Text> {
    (maker == "Generic").then_some(eh3d_view::texts::Text::MakerGeneric)
}

/// One letter as it is sorted: lower case and without the accent.
fn plain_letter(c: char) -> char {
    match c {
        '\u{e0}'..='\u{e5}' | '\u{101}' | '\u{103}' | '\u{105}' => 'a',
        '\u{e7}' | '\u{107}' | '\u{10d}' => 'c',
        '\u{10f}' | '\u{111}' => 'd',
        '\u{e8}'..='\u{eb}' | '\u{113}' | '\u{117}' | '\u{119}' | '\u{11b}' => 'e',
        '\u{11f}' => 'g',
        '\u{ec}'..='\u{ef}' | '\u{131}' | '\u{12b}' => 'i',
        '\u{142}' => 'l',
        '\u{f1}' | '\u{144}' | '\u{148}' => 'n',
        '\u{f2}'..='\u{f6}' | '\u{f8}' | '\u{151}' => 'o',
        '\u{159}' => 'r',
        '\u{15b}' | '\u{161}' | '\u{15f}' => 's',
        '\u{163}' | '\u{165}' => 't',
        '\u{f9}'..='\u{fc}' | '\u{16b}' | '\u{16f}' | '\u{171}' => 'u',
        '\u{fd}' | '\u{ff}' => 'y',
        '\u{17a}' | '\u{17c}' | '\u{17e}' => 'z',
        c => c,
    }
}

/// A piece of a name for sorting: a run of digits is a number, anything else is text.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum SortPart {
    Text(String),
    Number(u128),
}

/// The name cut in pieces for sorting: capitals and accents do not count, and a number is worth its value (so "FR4 500" comes before "FR4 1000").
fn sort_parts(name: &str) -> Vec<SortPart> {
    let mut parts = Vec::new();
    let mut text = String::new();
    let mut digits = String::new();
    let flush_text = |text: &mut String, parts: &mut Vec<SortPart>| {
        if !text.is_empty() {
            parts.push(SortPart::Text(std::mem::take(text)));
        }
    };
    for c in name.chars() {
        if c.is_ascii_digit() {
            flush_text(&mut text, &mut parts);
            digits.push(c);
        } else {
            if !digits.is_empty() {
                parts.push(SortPart::Number(digits.parse().unwrap_or(u128::MAX)));
                digits.clear();
            }
            for l in c.to_lowercase() {
                text.push(plain_letter(l));
            }
        }
    }
    if !digits.is_empty() {
        parts.push(SortPart::Number(digits.parse().unwrap_or(u128::MAX)));
    }
    flush_text(&mut text, &mut parts);
    parts
}

/// The order of the lists of makers and of materials: alphabetical without regard to capitals or accents, numbers by their value.
pub fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    sort_parts(a).cmp(&sort_parts(b)).then_with(|| a.cmp(b))
}

/// The makers, in alphabetical order (the data of the library keeps its own order).
pub fn makers() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = Vec::new();
    for m in MATERIALS {
        if !v.contains(&m.maker) {
            v.push(m.maker);
        }
    }
    v.sort_by(|a, b| natural_cmp(a, b));
    v
}

/// The materials the list shows: those of one maker, or (`None`) those of all the makers mixed, in alphabetical order of the name
/// that `name_of` gives them (the maker breaks a tie). Each comes with its place in [`MATERIALS`].
pub fn materials_view(
    maker: Option<&str>,
    name_of: &dyn Fn(&Material) -> String,
) -> Vec<(usize, &'static Material)> {
    let mut v: Vec<(usize, &'static Material)> = MATERIALS
        .iter()
        .enumerate()
        .filter(|(_, m)| maker.is_none_or(|k| m.maker == k))
        .collect();
    v.sort_by(|(_, a), (_, b)| {
        natural_cmp(&name_of(a), &name_of(b)).then_with(|| natural_cmp(a.maker, b.maker))
    });
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
            // a sheet without copper has none (0), and the layers stay as they were for when a copper board is chosen again
            copper_um: if m.copper_um.is_empty() {
                0.0
            } else {
                near(m.copper_um, self.copper_um)
            },
            layers: match m.sides {
                Sides::One => 1,
                Sides::Two => 2,
                Sides::Either | Sides::NoCopper => self.layers.clamp(1, 2),
            },
            size_mm: self.size_mm,
        }
    }

    /// A one-line description for the screen.
    pub fn describe(&self) -> String {
        let m = self.material();
        if self.has_no_copper() {
            return format!(
                "{} {} - {:.2} mm - no copper",
                m.maker, m.name, self.thickness_mm
            );
        }
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

    /// Whether the material is a plain sheet with no copper to etch or to isolate.
    pub fn has_no_copper(&self) -> bool {
        self.material().sides == Sides::NoCopper
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
        assert!(MATERIALS.len() >= 160);
        for m in MATERIALS {
            // a material has a thickness; it has copper unless it is a plain sheet (and then it has none at all)
            assert!(!m.thickness_mm.is_empty(), "{}", m.name);
            assert_eq!(
                m.copper_um.is_empty(),
                m.sides == Sides::NoCopper,
                "{}",
                m.name
            );
            assert!(
                m.thickness_mm.iter().all(|t| (0.01..=6.0).contains(t)),
                "{}",
                m.name
            );
            assert!(
                m.copper_um.iter().all(|c| (9.0..=350.0).contains(c)),
                "{}",
                m.name
            );
            assert!(
                m.formats_mm.iter().all(|&(w, d)| w > 20.0 && d > 20.0),
                "{}",
                m.name
            );
            assert!(m.dk.is_none_or(|d| (2.0..=11.0).contains(&d)), "{}", m.name);
            assert!(m.df.is_none_or(|d| (0.0..=0.05).contains(&d)), "{}", m.name);
            assert!(
                m.tg_c.is_none_or(|t| (50.0..=400.0).contains(&t)),
                "{}",
                m.name
            );
            assert!(!m.note.is_empty());
            // the lists are sorted and without repeated values
            assert!(m.thickness_mm.windows(2).all(|w| w[0] < w[1]), "{}", m.name);
            assert!(m.copper_um.windows(2).all(|w| w[0] < w[1]), "{}", m.name);
            // a material of the makers has at least one figure of its data sheet (the metal cores give their thermal conductivity in the note)
            if m.maker != "Generic" && m.core != Core::Aluminium {
                assert!(
                    m.dk.is_some() || m.df.is_some() || m.tg_c.is_some(),
                    "{}",
                    m.name
                );
            }
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
        assert!(makers().len() >= 24);
        assert!(!of_maker("Bungard").is_empty() && !of_maker("Rogers").is_empty());
    }

    #[test]
    fn the_materials_of_the_data_sheets_are_in_the_library() {
        let find = |maker: &str, name: &str| {
            MATERIALS
                .iter()
                .find(|m| m.maker == maker && m.name == name)
                .unwrap_or_else(|| panic!("{maker} {name}"))
        };
        // the published figures: Dk and Df at the frequency of the sheet, Tg by DSC where the sheet gives it
        let s7136 = find("Shengyi", "S7136H");
        assert_eq!(s7136.thickness_mm, &[0.254, 0.51, 0.76, 1.52]);
        assert_eq!(s7136.dk, Some(3.42));
        let gf220 = find("Shengyi", "SCGA-500 GF220");
        assert_eq!((gf220.dk, gf220.df), (Some(2.20), Some(0.0009)));
        assert_eq!(find("Shengyi", "mmWave77").formats_mm.len(), 2);
        assert_eq!(find("Isola", "IS620i").tg_c, Some(225.0));
        assert_eq!(find("Isola", "DE104 KF").df, Some(0.022));
        // Megtron 6 has the core thicknesses of its sheet and 1 oz copper
        let m6 = find("Panasonic", "Megtron 6 (R-5775)");
        assert_eq!(m6.thickness_mm.first(), Some(&0.05));
        assert_eq!(m6.thickness_mm.last(), Some(&0.925));
        assert_eq!(m6.copper_um, &[35.0]);
        // the RO4350B has the two thicknesses that the older sheets did not list
        let r = find("Rogers", "RO4350B");
        assert!(r.thickness_mm.contains(&0.101) && r.thickness_mm.contains(&0.422));
        // the metal core laminates carry copper up to 10 oz and the flexible ones film thinner than 0.04 mm
        assert_eq!(find("Ventec", "VT-4A1").copper_um.last(), Some(&350.0));
        assert!(find("ThinFlex", "W-0505ED-N4").thickness_mm[0] < 0.04);
        // a laminate with copper on both sides only, and one with the single side only
        assert_eq!(find("Panasonic", "R-1785").sides, Sides::Two);
        assert_eq!(find("Yuxinda", "AL-200").sides, Sides::One);
        // no note carries a year
        for m in MATERIALS {
            assert!(
                !m.note
                    .split(|c: char| !c.is_ascii_digit())
                    .any(|w| w.len() == 4 && w.starts_with("20")),
                "{}",
                m.name
            );
        }
    }

    fn sheet(name: &str) -> &'static Material {
        MATERIALS
            .iter()
            .find(|m| m.maker == "Generic" && m.name == name && m.sides == Sides::NoCopper)
            .unwrap_or_else(|| panic!("{name}"))
    }

    #[test]
    fn the_sheets_without_copper_have_the_thicknesses_that_were_asked_for() {
        let range = |m: &Material| (m.thickness_mm[0], *m.thickness_mm.last().unwrap());
        // aluminium alloy 0.2 to 5 mm, copper 0.1 to 5 mm, PTFE 0.2 to 5 mm
        assert_eq!(range(sheet("Aluminium sheet (5052 / 6061)")), (0.2, 5.0));
        assert_eq!(range(sheet("Copper sheet (C110)")), (0.1, 5.0));
        assert_eq!(range(sheet("PTFE sheet")), (0.2, 5.0));
        assert_eq!(sheet("Aluminium sheet (5052 / 6061)").core, Core::Metal);
        assert_eq!(sheet("Copper sheet (C110)").core, Core::Copper);
        assert_eq!(sheet("PTFE sheet").core, Core::Ptfe);
        // the woods: every one inside 0.5 to 5 mm, and between them they reach both ends
        let woods: Vec<&Material> = MATERIALS.iter().filter(|m| m.core == Core::Wood).collect();
        assert!(woods.len() >= 4);
        for w in &woods {
            let (lo, hi) = range(w);
            assert!(lo >= 0.5 && hi <= 5.0, "{}", w.name);
            assert_eq!(w.maker, "Generic");
        }
        assert!(woods.iter().any(|w| range(w).0 == 0.5) && woods.iter().any(|w| range(w).1 == 5.0));
        // the other plastics and the plain laminates are all there
        for name in [
            "Acrylic (PMMA)",
            "Polycarbonate (PC)",
            "ABS",
            "POM",
            "Nylon (PA6)",
            "PVC",
            "HDPE",
            "PETG",
            "G10 (FR4 without copper)",
            "Phenolic paper (Pertinax)",
            "Birch plywood",
            "MDF",
            "Balsa wood",
            "Basswood (lime)",
        ] {
            let m = sheet(name);
            assert!(!m.thickness_mm.is_empty(), "{name}");
        }
        assert_eq!(sheet("G10 (FR4 without copper)").core, Core::Fr4);
        assert_eq!(sheet("Phenolic paper (Pertinax)").core, Core::Fr2);
        for m in MATERIALS.iter().filter(|m| m.sides == Sides::NoCopper) {
            // no copper, no invented electrical figures, and the note says the thicknesses are the usual ones
            assert!(m.copper_um.is_empty() && !m.photoresist, "{}", m.name);
            assert!(
                m.dk.is_none() && m.df.is_none() && m.tg_c.is_none(),
                "{}",
                m.name
            );
            assert!(m.note.contains("Usual thicknesses"), "{}", m.name);
            assert!(
                m.thickness_mm.iter().all(|t| (0.1..=5.0).contains(t)),
                "{}",
                m.name
            );
        }
        assert!(
            MATERIALS
                .iter()
                .filter(|m| m.sides == Sides::NoCopper)
                .count()
                >= 17
        );
    }

    #[test]
    fn a_sheet_without_copper_has_copper_zero_and_comes_back_from_its_text() {
        let b = Board::standard();
        assert!(!b.has_no_copper());
        let idx = MATERIALS
            .iter()
            .position(|m| m.name == "Copper sheet (C110)")
            .unwrap();
        let s = b.with_material(idx);
        assert!(s.has_no_copper());
        assert_eq!(s.copper_um, 0.0);
        assert!((s.thickness_mm - 1.5).abs() < 1e-9 || (s.thickness_mm - 2.0).abs() < 1e-9);
        assert!(s.describe().contains("no copper"));
        assert_eq!(from_text(&to_text(&s)), Some(s));
        // going back to a laminate takes copper again
        let back = s.with_material(b.material);
        assert!(!back.has_no_copper() && back.copper_um > 0.0);
    }

    #[test]
    fn the_names_with_words_of_the_sheets_have_their_text() {
        let words = [
            "Aluminium sheet (5052 / 6061)",
            "Copper sheet (C110)",
            "Birch plywood",
            "Balsa wood",
            "Basswood (lime)",
            "PTFE sheet",
            "Acrylic (PMMA)",
            "Polycarbonate (PC)",
            "Nylon (PA6)",
            "G10 (FR4 without copper)",
            "Phenolic paper (Pertinax)",
        ];
        for w in words {
            assert!(sheet(w).name_text().is_some(), "{w}");
        }
        // the designations are the same in every language
        for n in ["MDF", "ABS", "POM", "PVC", "HDPE", "PETG"] {
            assert!(sheet(n).name_text().is_none(), "{n}");
        }
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

/// Whether a board of a size fits the table of the active machine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fit {
    /// It fits as it lies.
    Fits,
    /// It fits only turned 90 degrees (no warning: the operator can lay it that way).
    FitsTurned,
    /// It fits in neither way: how many mm it goes over the width and over the depth of the table in the way that goes over least.
    TooBig { over_w: f64, over_d: f64 },
    /// The size of the board or of the table is not known.
    Unknown,
}

/// Compares a board (width, depth) with the table (width, depth), both in mm. The board may be turned 90 degrees, so a warning is only for a board that
/// fits in neither way. A size that is not known (zero, negative or missing) gives [`Fit::Unknown`].
pub fn fit_on_table(board: Option<(f64, f64)>, table: Option<(f64, f64)>) -> Fit {
    const EPS: f64 = 0.01;
    let ok = |s: (f64, f64)| s.0.is_finite() && s.1.is_finite() && s.0 > 0.0 && s.1 > 0.0;
    let (Some(b), Some(t)) = (board.filter(|s| ok(*s)), table.filter(|s| ok(*s))) else {
        return Fit::Unknown;
    };
    if b.0 <= t.0 + EPS && b.1 <= t.1 + EPS {
        return Fit::Fits;
    }
    if b.1 <= t.0 + EPS && b.0 <= t.1 + EPS {
        return Fit::FitsTurned;
    }
    let as_lies = ((b.0 - t.0).max(0.0), (b.1 - t.1).max(0.0));
    let turned = ((b.1 - t.0).max(0.0), (b.0 - t.1).max(0.0));
    let (over_w, over_d) = if turned.0 + turned.1 < as_lies.0 + as_lies.1 {
        turned
    } else {
        as_lies
    };
    Fit::TooBig { over_w, over_d }
}

#[cfg(test)]
mod fit_tests {
    use super::*;

    const TABLE: Option<(f64, f64)> = Some((495.0, 325.0));

    #[test]
    fn a_board_that_lies_on_the_table_fits() {
        assert_eq!(fit_on_table(Some((160.0, 100.0)), TABLE), Fit::Fits);
        assert_eq!(fit_on_table(Some((495.0, 325.0)), TABLE), Fit::Fits);
    }

    #[test]
    fn a_board_that_fits_only_turned_gives_no_warning() {
        assert_eq!(fit_on_table(Some((320.0, 480.0)), TABLE), Fit::FitsTurned);
    }

    #[test]
    fn a_board_that_fits_in_neither_way_says_how_much_it_goes_over() {
        // 510 x 1150 is over in both ways; turned it goes over less
        let f = fit_on_table(Some((510.0, 1150.0)), TABLE);
        assert_eq!(
            f,
            Fit::TooBig {
                over_w: 15.0,
                over_d: 825.0
            }
        );
        // only the width is over
        assert_eq!(
            fit_on_table(Some((520.0, 300.0)), TABLE),
            Fit::TooBig {
                over_w: 25.0,
                over_d: 0.0
            }
        );
    }

    #[test]
    fn the_table_changes_the_answer() {
        let b = Some((520.0, 400.0));
        assert!(matches!(fit_on_table(b, TABLE), Fit::TooBig { .. }));
        assert_eq!(fit_on_table(b, Some((600.0, 500.0))), Fit::Fits);
    }

    #[test]
    fn an_unknown_size_gives_no_answer() {
        assert_eq!(fit_on_table(None, TABLE), Fit::Unknown);
        assert_eq!(fit_on_table(Some((100.0, 100.0)), None), Fit::Unknown);
        assert_eq!(fit_on_table(Some((0.0, 100.0)), TABLE), Fit::Unknown);
        assert_eq!(fit_on_table(Some((100.0, f64::NAN)), TABLE), Fit::Unknown);
    }
}

#[cfg(test)]
mod order_tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn capitals_and_accents_do_not_count() {
        assert_eq!(natural_cmp("Zebra", "apple"), Ordering::Greater);
        assert_eq!(natural_cmp("apple", "Banana"), Ordering::Less);
        assert_eq!(natural_cmp("\u{e9}cole", "Ecole2"), Ordering::Less);
        assert_eq!(natural_cmp("\u{e9}clair", "eclat"), Ordering::Less);
        assert_eq!(natural_cmp("\u{c1}rbol", "azul"), Ordering::Less);
        assert_eq!(natural_cmp("\u{c1}rbol", "Brazo"), Ordering::Less);
    }

    #[test]
    fn numbers_go_by_their_value() {
        assert_eq!(natural_cmp("FR4 500", "FR4 1000"), Ordering::Less);
        assert_eq!(natural_cmp("FR4 1000", "FR4 500"), Ordering::Greater);
        assert_eq!(natural_cmp("R-5775", "R-5670"), Ordering::Greater);
        assert_eq!(natural_cmp("a2", "a10"), Ordering::Less);
        assert_eq!(natural_cmp("a2", "a2"), Ordering::Equal);
    }

    #[test]
    fn the_makers_come_in_alphabetical_order() {
        let m = makers();
        for w in m.windows(2) {
            assert_ne!(natural_cmp(w[0], w[1]), Ordering::Greater, "{w:?}");
        }
        assert!(m.contains(&"Generic"));
    }

    #[test]
    fn the_materials_of_a_maker_are_in_order_and_are_the_makers() {
        let name = |m: &Material| m.name.to_string();
        for maker in makers() {
            let v = materials_view(Some(maker), &name);
            assert_eq!(v.len(), of_maker(maker).len());
            assert!(v.iter().all(|(_, m)| m.maker == maker));
            for w in v.windows(2) {
                assert_ne!(natural_cmp(w[0].1.name, w[1].1.name), Ordering::Greater);
            }
        }
    }

    #[test]
    fn the_view_of_all_makers_has_every_material_once_in_order_of_name() {
        let name = |m: &Material| m.name.to_string();
        let all = materials_view(None, &name);
        assert_eq!(all.len(), MATERIALS.len());
        let mut places: Vec<usize> = all.iter().map(|(i, _)| *i).collect();
        places.sort_unstable();
        places.dedup();
        assert_eq!(places.len(), MATERIALS.len());
        for w in all.windows(2) {
            assert_ne!(natural_cmp(w[0].1.name, w[1].1.name), Ordering::Greater);
        }
        // each one is the material of that place, so choosing it from this view gives the same board as from the view of its maker
        for (i, m) in &all {
            assert_eq!((m.maker, m.name), (MATERIALS[*i].maker, MATERIALS[*i].name));
        }
    }
}
