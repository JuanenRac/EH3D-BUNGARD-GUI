//! The machines the interface knows: the size of the work area, the size of the whole machine and the places of the tool changer, as the
//! maker publishes them. The view of the machine is built from them, so a bigger table in the configuration shows a bigger machine.
//!
//! The machine is told by the table of the configuration: the smallest machine whose work area holds it. The sizes of the whole machine
//! are the published ones (length x width x height); which of the two flat sizes runs along the rails is taken from the photographs and
//! is not confirmed.

use eh3d_view::scene::Table;

/// The family a machine belongs to; the XL and the Premium have their own control unit, which this program does not drive yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Ccd,
    Ccd2,
    CcdXl,
    CcdPremium,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Profile {
    pub family: Family,
    pub name: &'static str,
    /// The work area in mm: X along the rails, Y along the gantry, Z the travel of the spindle.
    pub work_mm: (f64, f64, f64),
    /// The whole machine in mm: along the rails, across them, and the height.
    pub overall_mm: (f64, f64, f64),
    pub weight_kg: f64,
    /// The places of the tool changer of the model that has one (the standard one has 15 and the test pin).
    pub tool_places: u8,
}

pub const PROFILES: [Profile; 4] = [
    Profile {
        family: Family::Ccd2,
        name: "CCD/2",
        work_mm: (325.0, 270.0, 38.0),
        overall_mm: (700.0, 550.0, 300.0),
        weight_kg: 30.0,
        tool_places: 15,
    },
    Profile {
        family: Family::Ccd,
        name: "CCD",
        work_mm: (495.0, 325.0, 38.0),
        overall_mm: (800.0, 700.0, 300.0),
        weight_kg: 35.0,
        tool_places: 15,
    },
    Profile {
        family: Family::CcdPremium,
        name: "CCD Premium",
        work_mm: (500.0, 400.0, 64.0),
        overall_mm: (950.0, 800.0, 600.0),
        weight_kg: 80.0,
        tool_places: 30,
    },
    Profile {
        family: Family::CcdXl,
        name: "CCD XL",
        work_mm: (600.0, 500.0, 38.0),
        overall_mm: (950.0, 950.0, 300.0),
        weight_kg: 49.0,
        tool_places: 25,
    },
];

/// How the tools are changed: by the operator (the MTC models) or by the machine from its rack (the ATC models).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolChange {
    Manual,
    Automatic,
}

/// A model of the machine as it is sold: a machine of the family with the tool change by hand or by the changer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Model {
    pub profile: usize,
    pub change: ToolChange,
}

/// Every model: each machine of the family as the MTC (manual change) and as the ATC (changer with its rack).
pub fn models() -> Vec<Model> {
    (0..PROFILES.len())
        .flat_map(|profile| {
            [ToolChange::Manual, ToolChange::Automatic].map(|change| Model { profile, change })
        })
        .collect()
}

impl Model {
    pub fn profile(&self) -> &'static Profile {
        &PROFILES[self.profile.min(PROFILES.len() - 1)]
    }

    /// The name of the model: the machine and how it changes the tools (`CCD MTC`, `CCD/2 ATC`).
    pub fn name(&self) -> String {
        format!(
            "{} {}",
            self.profile().name,
            match self.change {
                ToolChange::Manual => "MTC",
                ToolChange::Automatic => "ATC",
            }
        )
    }

    /// The places of the rack: none on the models with the manual change.
    pub fn rack_places(&self) -> u8 {
        match self.change {
            ToolChange::Manual => 0,
            ToolChange::Automatic => self.profile().tool_places,
        }
    }

    /// The model of a name (`CCD MTC`); `None` when there is none.
    pub fn from_name(name: &str) -> Option<usize> {
        models().iter().position(|m| m.name() == name.trim())
    }

    /// The model that a table is most like, with the manual change.
    pub fn for_table(table: &Table) -> usize {
        let p = for_table(table);
        models()
            .iter()
            .position(|m| m.profile().name == p.name && m.change == ToolChange::Manual)
            .unwrap_or(0)
    }
}

/// The machine for a table: the one whose published work area is the most like it. The published sizes are where the machines start;
/// the usable area is whatever the configuration says, and a machine often has a couple of centimetres more in X and Y, so a table a
/// little bigger (or smaller) than the published one stays on the same machine.
pub fn for_table(table: &Table) -> &'static Profile {
    let distance = |p: &Profile| {
        (table.width_mm / p.work_mm.0).ln().abs() + (table.depth_mm / p.work_mm.1).ln().abs()
    };
    PROFILES
        .iter()
        .min_by(|a, b| distance(a).total_cmp(&distance(b)))
        .unwrap_or(&PROFILES[0])
}

impl Profile {
    pub fn work_mm_xy(&self) -> (f64, f64) {
        (self.work_mm.0, self.work_mm.1)
    }

    /// The plate of the table is bigger than the work area: what is left on each side along and across the rails for a usable area of
    /// `usable` mm (the published one, or the bigger one the configuration says), so that the machine is as big as published (80 mm of
    /// rails and 200 mm of gantry and motor are taken from the whole size). The frame grows with a usable area bigger than the published.
    pub fn plate_margins(&self, usable: (f64, f64)) -> (f64, f64) {
        let wx = usable.0.max(self.work_mm.0);
        let wy = usable.1.max(self.work_mm.1);
        (
            ((self.overall_mm.0 - 80.0 - wx) / 2.0).max(20.0),
            ((self.overall_mm.1 - 200.0 - wy) / 2.0).max(20.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_machine_is_a_model_with_the_manual_and_with_the_automatic_change() {
        let all = models();
        assert_eq!(all.len(), PROFILES.len() * 2);
        for m in &all {
            assert_eq!(
                Model::from_name(&m.name()),
                models().iter().position(|x| x == m)
            );
            assert_eq!(m.rack_places() > 0, m.change == ToolChange::Automatic);
        }
        assert_eq!(
            all[Model::for_table(&Table::new(495.0, 325.0))].name(),
            "CCD MTC"
        );
        assert_eq!(
            Model::from_name("CCD/2 ATC").map(|i| all[i].rack_places()),
            Some(15)
        );
        assert_eq!(Model::from_name("nothing"), None);
    }

    #[test]
    fn the_machine_is_the_one_the_table_is_most_like() {
        assert_eq!(for_table(&Table::new(495.0, 325.0)).name, "CCD");
        assert_eq!(for_table(&Table::new(325.0, 270.0)).name, "CCD/2");
        assert_eq!(for_table(&Table::new(500.0, 400.0)).name, "CCD Premium");
        assert_eq!(for_table(&Table::new(600.0, 500.0)).name, "CCD XL");
        // a couple of centimetres more or less, as a real machine often has, stays on the same machine
        assert_eq!(for_table(&Table::new(520.0, 345.0)).name, "CCD");
        assert_eq!(for_table(&Table::new(470.0, 305.0)).name, "CCD");
        assert_eq!(for_table(&Table::new(345.0, 290.0)).name, "CCD/2");
        assert_eq!(for_table(&Table::new(300.0, 250.0)).name, "CCD/2");
        // nothing like any: the nearest
        assert_eq!(for_table(&Table::new(900.0, 900.0)).name, "CCD XL");
    }

    #[test]
    fn the_published_sizes_are_there() {
        let ccd = for_table(&Table::new(495.0, 325.0));
        assert_eq!(ccd.work_mm, (495.0, 325.0, 38.0));
        assert_eq!(ccd.overall_mm, (800.0, 700.0, 300.0));
        assert_eq!(ccd.weight_kg, 35.0);
        let two = for_table(&Table::new(325.0, 270.0));
        assert_eq!(two.overall_mm, (700.0, 550.0, 300.0));
        assert_eq!(two.weight_kg, 30.0);
    }

    #[test]
    fn the_plate_leaves_room_and_grows_with_the_usable_area() {
        for p in PROFILES {
            let (mx, my) = p.plate_margins(p.work_mm_xy());
            assert!(mx >= 20.0 && my >= 20.0, "{}", p.name);
            assert!(
                p.work_mm.0 + 2.0 * mx + 80.0 <= p.overall_mm.0 + 40.0,
                "{}",
                p.name
            );
        }
        let ccd = for_table(&Table::new(495.0, 325.0));
        let (m0, _) = ccd.plate_margins((495.0, 325.0));
        let (m1, _) = ccd.plate_margins((535.0, 325.0));
        // 4 cm more usable leave 2 cm less on each side
        assert!((m0 - m1 - 20.0).abs() < 1e-9);
        // a smaller usable area does not change the plate
        assert_eq!(
            ccd.plate_margins((400.0, 300.0)),
            ccd.plate_margins((495.0, 325.0))
        );
    }
}
