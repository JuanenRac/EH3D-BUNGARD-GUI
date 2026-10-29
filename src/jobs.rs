//! From a drilling file or an HP-GL file to the scene the views draw.

use eh3d_drillpro::config;
use eh3d_drillpro::ncd::DrillFile;
use eh3d_view::scene::{Hole, Path, Scene, Table};
use eh3d_routepro::hpgl::{Plot, UNIT_MM};

/// The table of the Bungard CCD when the configuration says nothing else.
pub const DEFAULT_TABLE: Table = Table::new(495.0, 325.0);

/// The table chosen in the configuration file (the clipping of X and Y); the default one when the text is not a configuration file.
pub fn table_from_config(text: &str, route: bool) -> Table {
    let settings = config::parse(text).and_then(|c| {
        if route {
            c.route_settings()
        } else {
            c.settings()
        }
    });
    match settings {
        Some(s) if s.clip_mm.0 > 0.0 && s.clip_mm.1 > 0.0 => Table::new(s.clip_mm.0, s.clip_mm.1),
        _ => DEFAULT_TABLE,
    }
}

/// The diameter of a tool in mm. The header of the file says it when it can (`T01C0.8`, in inches when the file is not metric);
/// otherwise the number of the tool is the diameter in tenths of a millimetre, as the tool tables of the machines are numbered (04 is
/// 0.4 mm, 10 is 1.0 mm, 30 is 3.0 mm), and the numbers below 4 are taken as 1.0 mm.
pub fn tool_diameter(tool: u8, declared: &[(u8, f64)], metric: bool) -> f64 {
    if let Some(&(_, v)) = declared.iter().find(|(t, _)| *t == tool) {
        return if metric { v } else { v * 25.4 };
    }
    if tool >= 4 {
        tool as f64 / 10.0
    } else {
        1.0
    }
}

/// The holes of a drilling file. `unit_mm` is the unit of its coordinates (0.0254 for mils, the usual one).
pub fn scene_from_drill(file: &DrillFile, table: Table, unit_mm: f64) -> Scene {
    let mut scene = Scene::new(table);
    scene.holes = file
        .holes
        .iter()
        .map(|h| Hole {
            x_mm: h.x_mil as f64 * unit_mm,
            y_mm: h.y_mil as f64 * unit_mm,
            diameter_mm: tool_diameter(h.tool, &file.diameters, file.metric),
            tool: h.tool,
            done: false,
        })
        .collect();
    scene
}

/// The paths of an HP-GL file (one plotter step is 0.025 mm); the pen is the tool.
pub fn scene_from_plot(plot: &Plot, table: Table) -> Scene {
    let mut scene = Scene::new(table);
    scene.paths = plot
        .paths
        .iter()
        .map(|p| Path {
            tool: p.pen,
            points: p
                .points
                .iter()
                .map(|&(x, y)| (x as f64 * UNIT_MM, y as f64 * UNIT_MM))
                .collect(),
            reached: 0,
        })
        .collect();
    scene
}

/// Reads a text file as the machines' files are written (one byte per letter).
pub fn read_latin1(path: &std::path::Path) -> std::io::Result<String> {
    Ok(std::fs::read(path)?.iter().map(|&c| c as char).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use eh3d_drillpro::ncd;
    use eh3d_routepro::hpgl;

    #[test]
    fn the_holes_come_with_their_diameters_in_mm() {
        let f = ncd::parse(
            "M48\nMETRIC\nT01C0.8\n%\nT01\nX1000Y2000\nT10\nX0Y500\nT30\nX100Y100\nM30\n",
        )
        .unwrap();
        let s = scene_from_drill(&f, DEFAULT_TABLE, 0.0254);
        assert_eq!(s.holes.len(), 3);
        assert_eq!(s.holes[0].diameter_mm, 0.8);
        assert!((s.holes[0].x_mm - 25.4).abs() < 1e-9 && (s.holes[0].y_mm - 50.8).abs() < 1e-9);
        assert_eq!(s.holes[1].diameter_mm, 1.0);
        assert_eq!(s.holes[2].diameter_mm, 3.0);
        assert_eq!(s.sizes(), vec![0.8, 1.0, 3.0]);
    }

    #[test]
    fn a_header_in_inches_is_taken_as_inches() {
        assert!((tool_diameter(1, &[(1, 0.0315)], false) - 0.80010).abs() < 1e-4);
        assert_eq!(tool_diameter(1, &[(1, 0.8)], true), 0.8);
        assert_eq!(tool_diameter(2, &[], true), 1.0);
        assert_eq!(tool_diameter(4, &[], true), 0.4);
    }

    #[test]
    fn the_paths_of_an_hpgl_file_are_in_mm() {
        let p = hpgl::parse("IN;SP1;PU0,0;PD400,0,400,800;SP2;PU40,40;PD80,80;");
        let s = scene_from_plot(&p, DEFAULT_TABLE);
        assert_eq!(s.paths.len(), 2);
        assert_eq!(s.paths[0].tool, 1);
        assert_eq!(s.paths[0].points[1], (10.0, 0.0));
        assert_eq!(s.paths[0].points[2], (10.0, 20.0));
        assert_eq!(s.paths[1].tool, 2);
        assert_eq!(s.bounds(), Some((0.0, 0.0, 10.0, 20.0)));
    }

    #[test]
    fn a_drilling_and_a_routing_job_overlay() {
        let mut s = scene_from_drill(
            &ncd::parse("%\nT10\nX1000Y1000\nM30\n").unwrap(),
            DEFAULT_TABLE,
            0.0254,
        );
        s.overlay(&scene_from_plot(
            &hpgl::parse("SP1;PU0,0;PD400,0;"),
            DEFAULT_TABLE,
        ));
        assert_eq!((s.holes.len(), s.paths.len()), (1, 1));
        assert_eq!(s.progress(), 0.0);
    }

    #[test]
    fn the_table_is_the_one_of_the_configuration() {
        let dcf = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../EH3D-DRILLPRO/config/DRILLPRO.DCF"
        ))
        .unwrap();
        let text: String = dcf.iter().map(|&c| c as char).collect();
        assert_eq!(table_from_config(&text, false), Table::new(495.0, 325.0));
        assert_eq!(table_from_config("nothing", false), DEFAULT_TABLE);
    }
}
