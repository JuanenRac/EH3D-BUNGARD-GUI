//! `bungard-gui --report`: the report of a job written to the terminal, with no window. The job is the drilling file (`--drill`) and the routing file (`--route`),
//! or files given without an option (a drilling file by `.ncd` or `.drl`, a routing file by `.plt`, `.hpgl` or `.hpg`); `--panel COLUMNS,ROWS` repeats the work in
//! a panel, `--lang` chooses the language, and the figures of the estimate are those of `report_defaults.cfg` over which `bungard-gui.report.cfg` next to the
//! settings file takes its place.

use std::path::{Path, PathBuf};

use eh3d_view::texts::Lang;

use crate::cli::Options;
use crate::jobs;
use crate::report::{self, Config};

/// Which of the files given without an option is a drilling file and which a routing file.
fn sort_files(o: &Options) -> (Option<PathBuf>, Option<PathBuf>) {
    let mut drill = o.drill.clone();
    let mut route = o.route.clone();
    for f in &o.open {
        let ext = f
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .unwrap_or_default();
        match ext.as_str() {
            "ncd" | "drl" if drill.is_none() => drill = Some(f.clone()),
            "plt" | "hpgl" | "hpg" if route.is_none() => route = Some(f.clone()),
            _ => {}
        }
    }
    (drill, route)
}

fn read(path: &Path) -> Result<String, String> {
    jobs::read_latin1(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// The report of the job the options name, as text. An error says what could not be read.
pub fn run(o: &Options, cfg: &Config) -> Result<String, String> {
    let (drill, route) = sort_files(o);
    if drill.is_none() && route.is_none() {
        return Err(
            "--report needs a drilling file (--drill FILE) or a routing file (--route FILE)".into(),
        );
    }
    let mut scene = eh3d_view::scene::Scene::new(jobs::DEFAULT_TABLE);
    if let Some(p) = &drill {
        let file = eh3d_drillpro::ncd::parse(&read(p)?)
            .map_err(|_| format!("{}: not a drilling file", p.display()))?;
        scene.overlay(&jobs::scene_from_drill(&file, jobs::DEFAULT_TABLE, 0.0254));
    }
    if let Some(p) = &route {
        let plot = eh3d_routepro::hpgl::parse(&read(p)?);
        scene.overlay(&jobs::scene_from_plot(&plot, jobs::DEFAULT_TABLE));
    }
    let (columns, rows) = o.panel.unwrap_or((1, 1));
    let (columns, rows) = (columns.max(1), rows.max(1));
    let panel = report::panel(&scene, columns, rows, cfg.panel_gap_mm);
    let r = report::build(&panel, columns * rows, cfg);
    Ok(report::text(&r, o.lang.unwrap_or(Lang::En), &[]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(args: &str) -> Options {
        Options::parse(args.split_whitespace().map(String::from)).unwrap()
    }

    fn file(name: &str, text: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("eh3d-report-{}-{name}", std::process::id()));
        std::fs::write(&p, text).unwrap();
        p
    }

    #[test]
    fn the_files_are_told_apart_by_their_extension() {
        let o = options("a.NCD b.plt c.zip --route r.hpgl");
        assert_eq!(
            sort_files(&o),
            (Some(PathBuf::from("a.NCD")), Some(PathBuf::from("r.hpgl")))
        );
        assert_eq!(sort_files(&options("")), (None, None));
    }

    #[test]
    fn without_a_file_the_report_says_what_it_needs() {
        let e = run(&options("--report"), &Config::defaults()).unwrap_err();
        assert!(e.contains("--drill"), "{e}");
    }

    #[test]
    fn a_missing_file_is_named() {
        let o = options("--report --drill /no/such/folder/a.ncd");
        let e = run(&o, &Config::defaults()).unwrap_err();
        assert!(e.contains("a.ncd"), "{e}");
    }

    #[test]
    fn a_drilling_and_a_routing_file_give_their_counts_and_a_panel_multiplies_them() {
        let ncd = file(
            "a.ncd",
            "%\nT10\nX1000Y1000\nX2000Y1000\nT30\nX100Y100\nM30\n",
        );
        let plt = file("a.plt", "IN;SP1;PU0,0;PD400,0,400,800;");
        let one = Options {
            report: true,
            drill: Some(ncd.clone()),
            route: Some(plt.clone()),
            ..Default::default()
        };
        let text = run(&one, &Config::defaults()).unwrap();
        assert!(text.contains("Pieces: 1"), "{text}");
        assert!(text.contains("Holes: 3 (3 per piece)"), "{text}");
        assert!(text.contains("Paths: 1"), "{text}");
        assert!(text.contains("T10  1.00 mm  x2"), "{text}");
        assert!(text.contains("Routed length: 30.0 mm"), "{text}");
        let four = Options {
            panel: Some((2, 2)),
            lang: Some(Lang::Es),
            ..one
        };
        let text = run(&four, &Config::defaults()).unwrap();
        assert!(text.contains("Piezas: 4"), "{text}");
        assert!(text.contains("Agujeros: 12 (3 por pieza)"), "{text}");
        assert!(text.contains("Longitud fresada: 120.0 mm"), "{text}");
        let _ = std::fs::remove_file(ncd);
        let _ = std::fs::remove_file(plt);
    }
}
