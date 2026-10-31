//! The check of the work through the camera of the drill head, without a window: which holes of the job the camera should see, the whole check (the ring of LEDs, the
//! picture, the analyser of the shop, the colour of the ring that says how it went), the numbers of the profile that the tolerance window changes and the file that keeps the
//! profile. The window that shows it is `gui/inspect.rs`.

use std::path::Path;
use std::time::Duration;

use eh3d_aux::{Cue, LightRequest, Link};
use eh3d_inspect::{inspect, suggest_light_level, ExpectedHole, Gray, Profile, Report, Verdict};
use eh3d_view::texts::Text;

/// The levels of the ring for the picture that the window offers: as it is (`None`) and three levels.
pub const LIGHT_CHOICES: [Option<u32>; 4] = [None, Some(30), Some(60), Some(80)];

/// The scale of a picture until the shop has calibrated it: a guess for a camera a few centimetres above the board.
pub const DEFAULT_PX_PER_MM: f64 = 40.0;

/// The sensor needs a moment to settle its exposure after the light changes.
const SETTLE: Duration = Duration::from_millis(450);

/// How wide the picture is, to choose the holes the camera can see before the first picture arrives.
pub const DEFAULT_PICTURE: (f64, f64) = (640.0, 480.0);

/// The holes of the job (x, y, width in mm, in the table's coordinates) that the camera sees when it is centred on `centre`, as the analyser wants them: in millimetres from
/// the centre of the picture. `fov` is the size of the picture in mm. The hole in the centre comes first.
pub fn holes_in_view(
    holes: &[(f64, f64, f64)],
    centre: usize,
    fov: (f64, f64),
) -> Vec<ExpectedHole> {
    let Some(&(cx, cy, _)) = holes.get(centre) else {
        return Vec::new();
    };
    let (hw, hh) = (fov.0 / 2.0, fov.1 / 2.0);
    let mut out: Vec<ExpectedHole> = Vec::new();
    for (i, &(x, y, d)) in holes.iter().enumerate() {
        let (dx, dy) = (x - cx, y - cy);
        // a hole only half in the picture would be judged wrongly: it has to be wholly inside
        if dx.abs() + d / 2.0 > hw || dy.abs() + d / 2.0 > hh {
            continue;
        }
        let e = ExpectedHole {
            x_mm: dx,
            y_mm: dy,
            dia_mm: d,
        };
        if i == centre {
            out.insert(0, e);
        } else {
            out.push(e);
        }
    }
    out
}

/// The scale that makes the hole in the centre of the last report measure what the job says it is, from the pixels it covered. `None` when no hole was found.
pub fn calibrated_scale(report: &Report, profile: &Profile) -> Option<f64> {
    let h = report
        .holes
        .iter()
        .find(|h| h.found && h.expected.dia_mm > 0.0)?;
    let d_px = h.dia_mm * profile.px_per_mm;
    Profile::calibrate_from_hole(d_px, h.expected.dia_mm)
}

/// The whole check with the node: the ring of LEDs is lit (when a level is given) in the white of the camera, a picture is taken, the analyser judges it, and the ring says how it
/// went. A picture that is too dark or too bright is taken again with the light corrected. The light is left as it was.
pub fn run_check(
    link: &Link,
    profile: &Profile,
    holes: &[ExpectedHole],
    light: Option<u32>,
) -> Result<Report, String> {
    let before = link.light().ok();
    let ceiling = before.as_ref().map_or(80, |l| l.max_level.max(1));
    let mut level = light.unwrap_or(0).min(100);
    if light.is_some() {
        link.set_light(&LightRequest::camera(level))
            .map_err(|e| format!("light: {e}"))?;
        std::thread::sleep(SETTLE);
    }
    let mut tries = 0;
    let report = loop {
        let jpeg = link.snapshot()?;
        let img = Gray::from_jpeg(&jpeg)?;
        let report = inspect(&img, holes, profile);
        let poor = report
            .findings
            .iter()
            .any(|f| f.code == "image_too_dark" || f.code == "image_too_bright");
        if poor && light.is_some() && tries < 2 {
            if let Some(next) = report
                .quality
                .and_then(|q| suggest_light_level(level, &q, 125.0, ceiling))
            {
                tries += 1;
                level = next;
                link.set_light(&LightRequest::camera(next))
                    .map_err(|e| format!("light: {e}"))?;
                std::thread::sleep(SETTLE);
                continue;
            }
        }
        break report;
    };
    // the ring says it too: green, red blinking, or amber when the picture was not good enough
    let cue = match report.verdict {
        Verdict::Ok => Cue::Ok,
        Verdict::NotOk => Cue::NotOk,
        Verdict::Unknown => Cue::Warn,
    };
    let _ = link.set_light(&LightRequest::cue(cue, None));
    if light.is_some() {
        if let Some(b) = before {
            let _ = link.set_light(&if b.on {
                LightRequest {
                    on: Some(true),
                    level: Some(b.level),
                    ..Default::default()
                }
            } else {
                LightRequest::off()
            });
        }
    }
    Ok(report)
}

/// A number of the profile that the tolerance window changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Param {
    HoleDia,
    HolePos,
    HoleRound,
    HoleDebris,
    MillWidth,
    MillTol,
    MillMin,
    MillSpread,
    MinMean,
    MinSharp,
}

impl Param {
    pub const ALL: [Param; 10] = [
        Param::HoleDia,
        Param::HolePos,
        Param::HoleRound,
        Param::HoleDebris,
        Param::MillWidth,
        Param::MillTol,
        Param::MillMin,
        Param::MillSpread,
        Param::MinMean,
        Param::MinSharp,
    ];

    pub fn label(self) -> Text {
        match self {
            Param::HoleDia => Text::InspTuneHoleDia,
            Param::HolePos => Text::InspTuneHolePos,
            Param::HoleRound => Text::InspTuneRound,
            Param::HoleDebris => Text::InspTuneDebris,
            Param::MillWidth => Text::InspTuneMillWidth,
            Param::MillTol => Text::InspTuneMillTol,
            Param::MillMin => Text::InspTuneMillMin,
            Param::MillSpread => Text::InspTuneSpread,
            Param::MinMean => Text::InspTuneMinMean,
            Param::MinSharp => Text::InspTuneMinSharp,
        }
    }

    /// (least, most, step) of the value.
    fn range(self) -> (f64, f64, f64) {
        match self {
            Param::HoleDia => (0.005, 1.0, 0.01),
            Param::HolePos => (0.01, 2.0, 0.01),
            Param::HoleRound => (0.5, 1.0, 0.01),
            Param::HoleDebris => (0.0, 1.0, 0.02),
            Param::MillWidth => (0.05, 2.0, 0.01),
            Param::MillTol => (0.05, 1.0, 0.05),
            Param::MillMin => (0.02, 1.0, 0.01),
            Param::MillSpread => (0.1, 2.0, 0.05),
            Param::MinMean => (5.0, 120.0, 5.0),
            Param::MinSharp => (1.0, 400.0, 5.0),
        }
    }

    pub fn get(self, p: &Profile) -> f64 {
        match self {
            Param::HoleDia => p.hole_dia_tol_mm,
            Param::HolePos => p.hole_pos_tol_mm,
            Param::HoleRound => p.hole_min_roundness,
            Param::HoleDebris => p.hole_max_debris,
            Param::MillWidth => p.mill_width_mm,
            Param::MillTol => p.mill_width_tol,
            Param::MillMin => p.mill_min_width_mm,
            Param::MillSpread => p.mill_max_spread,
            Param::MinMean => p.min_mean,
            Param::MinSharp => p.min_sharpness,
        }
    }

    fn put(self, p: &mut Profile, v: f64) {
        match self {
            Param::HoleDia => p.hole_dia_tol_mm = v,
            Param::HolePos => p.hole_pos_tol_mm = v,
            Param::HoleRound => p.hole_min_roundness = v,
            Param::HoleDebris => p.hole_max_debris = v,
            Param::MillWidth => p.mill_width_mm = v,
            Param::MillTol => p.mill_width_tol = v,
            Param::MillMin => p.mill_min_width_mm = v,
            Param::MillSpread => p.mill_max_spread = v,
            Param::MinMean => p.min_mean = v,
            Param::MinSharp => p.min_sharpness = v,
        }
    }

    /// One step up (`dir` > 0) or down, kept inside the range, and not so that the least width of a channel goes over the channel itself.
    pub fn nudge(self, p: &mut Profile, dir: i32) {
        let (lo, hi, step) = self.range();
        let v = (self.get(p) + step * dir.signum() as f64).clamp(lo, hi);
        // 0.1 + 0.01 is not 0.11 in a float: the value is kept to the digits of the step
        let v = (v / step).round() * step;
        self.put(p, v.clamp(lo, hi));
        if p.mill_min_width_mm >= p.mill_max_channel_mm {
            p.mill_min_width_mm = p.mill_max_channel_mm - 0.01;
        }
    }

    /// The value as the window writes it.
    pub fn show(self, p: &Profile) -> String {
        let v = self.get(p);
        match self {
            Param::MinMean | Param::MinSharp => format!("{v:.0}"),
            Param::HoleRound | Param::HoleDebris | Param::MillTol | Param::MillSpread => {
                format!("{v:.2}")
            }
            _ => format!("{v:.2} mm"),
        }
    }
}

/// The profile kept in a file (JSON); a file that is not there, or that cannot be read, gives the standard one with the guessed scale.
pub fn load_profile(path: &Path) -> Profile {
    let mut p = std::fs::read_to_string(path)
        .ok()
        .and_then(|t| Profile::from_json(&t).ok())
        .map(|(p, _)| p)
        .unwrap_or_default();
    if !p.calibrated() {
        p.px_per_mm = DEFAULT_PX_PER_MM;
    }
    p
}

pub fn save_profile(path: &Path, p: &Profile) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(
        path,
        serde_json::to_string_pretty(&p.to_json()).unwrap_or_default(),
    )
}

/// The preset that the window button `n` (0, 1, 2) stands for, keeping the scale of the picture and which checks are on.
pub fn preset_for(n: usize, current: &Profile) -> Profile {
    let name = Profile::presets()[n.min(2)];
    let mut p = Profile::preset(name).unwrap_or_default();
    p.px_per_mm = current.px_per_mm;
    p.check_holes = current.check_holes;
    p.check_milling = current.check_milling;
    p
}

/// The words for a finding of the analyser.
pub fn finding_text(code: &str) -> Option<Text> {
    Some(match code {
        "hole_missing" => Text::InspFindHoleMissing,
        "hole_wrong_size" => Text::InspFindHoleSize,
        "hole_off_centre" => Text::InspFindHoleOff,
        "hole_not_round" => Text::InspFindHoleRound,
        "hole_debris" => Text::InspFindHoleDebris,
        "hole_out_of_view" => Text::InspFindHoleView,
        "mill_no_channels" => Text::InspFindMillNone,
        "mill_too_narrow" => Text::InspFindMillNarrow,
        "mill_too_wide" => Text::InspFindMillWide,
        "mill_pinched" => Text::InspFindMillPinch,
        "mill_uneven" => Text::InspFindMillUneven,
        "image_too_dark" => Text::InspFindDark,
        "image_too_bright" => Text::InspFindBright,
        "image_low_contrast" => Text::InspFindContrast,
        "image_blurry" => Text::InspFindBlurry,
        "not_calibrated" => Text::InspFindScale,
        "nothing_to_check" => Text::InspFindNothing,
        "no_expected_holes" => Text::InspFindNoHoles,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::sync::{Arc, Mutex};

    #[test]
    fn the_holes_in_view_are_relative_to_the_one_in_the_centre_and_wholly_inside() {
        let job = [
            (10.0, 10.0, 0.8),
            (12.0, 10.5, 1.0),
            (10.0, 8.0, 0.8),
            (40.0, 40.0, 0.8),
            (16.9, 10.0, 0.8),
        ];
        let v = holes_in_view(&job, 0, (14.0, 10.0));
        assert_eq!(
            v[0],
            ExpectedHole {
                x_mm: 0.0,
                y_mm: 0.0,
                dia_mm: 0.8
            }
        );
        assert!(v.contains(&ExpectedHole {
            x_mm: 2.0,
            y_mm: 0.5,
            dia_mm: 1.0
        }));
        assert!(v.contains(&ExpectedHole {
            x_mm: 0.0,
            y_mm: -2.0,
            dia_mm: 0.8
        }));
        // too far, and half out of the picture, are left out
        assert_eq!(v.len(), 3, "{v:?}");
        // the centre may be any hole, and a number past the end gives nothing
        assert_eq!(holes_in_view(&job, 1, (14.0, 10.0))[0].dia_mm, 1.0);
        assert!(holes_in_view(&job, 9, (14.0, 10.0)).is_empty());
    }

    #[test]
    fn the_scale_comes_from_the_hole_in_the_centre() {
        // the analyser saw 0.60 mm at 40 px/mm, which is 24 px; the job says 0.80 mm
        let img = Gray::new(8, 8, 128);
        let mut report = inspect(&img, &[], &Profile::default());
        report.holes.push(eh3d_inspect::HoleResult {
            index: 0,
            expected: ExpectedHole {
                x_mm: 0.0,
                y_mm: 0.0,
                dia_mm: 0.8,
            },
            found: true,
            cx_px: 4.0,
            cy_px: 4.0,
            dia_mm: 0.6,
            offset_mm: 0.0,
            roundness: 1.0,
            debris: 0.0,
            issues: vec![],
        });
        let p = Profile {
            px_per_mm: 40.0,
            ..Profile::default()
        };
        assert!((calibrated_scale(&report, &p).unwrap() - 30.0).abs() < 1e-9);
        report.holes[0].found = false;
        assert!(calibrated_scale(&report, &p).is_none());
    }

    #[test]
    fn the_numbers_move_by_their_step_and_stay_inside_their_range() {
        let mut p = Profile::default();
        let before = p.hole_dia_tol_mm;
        Param::HoleDia.nudge(&mut p, 1);
        assert!(
            (p.hole_dia_tol_mm - (before + 0.01)).abs() < 1e-9,
            "{}",
            p.hole_dia_tol_mm
        );
        for _ in 0..500 {
            Param::HoleRound.nudge(&mut p, 1);
        }
        assert_eq!(p.hole_min_roundness, 1.0);
        for _ in 0..500 {
            Param::HoleRound.nudge(&mut p, -1);
        }
        assert_eq!(p.hole_min_roundness, 0.5);
        // the digits do not drift
        let mut q = Profile::default();
        for _ in 0..7 {
            Param::MillWidth.nudge(&mut q, 1);
        }
        assert_eq!(Param::MillWidth.show(&q), "0.27 mm");
        assert_eq!(Param::MinMean.show(&Profile::default()), "45");
        assert_eq!(Param::HoleRound.show(&Profile::default()), "0.85");
        for param in Param::ALL {
            assert!(param.get(&Profile::default()).is_finite());
        }
        // the narrowest channel cannot be pushed above the widest
        let mut r = Profile::default();
        for _ in 0..300 {
            Param::MillMin.nudge(&mut r, 1);
        }
        assert!(r.mill_min_width_mm <= r.mill_max_channel_mm);
    }

    #[test]
    fn a_profile_is_kept_in_a_file_and_the_scale_is_guessed_when_there_is_none() {
        let dir = std::env::temp_dir().join(format!("eh3d-gui-inspect-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let file = dir.join("sub").join("profile.json");
        assert_eq!(load_profile(&file).px_per_mm, DEFAULT_PX_PER_MM);
        let mut p = preset_for(1, &load_profile(&file));
        p.px_per_mm = 77.0;
        save_profile(&file, &p).unwrap();
        assert_eq!(load_profile(&file), p);
        // a damaged file is not a crash
        std::fs::write(&file, "{ nope").unwrap();
        assert_eq!(load_profile(&file).name, "standard");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_preset_keeps_the_scale_and_the_checks_that_are_on() {
        let cur = Profile {
            px_per_mm: 91.0,
            check_milling: false,
            ..Profile::default()
        };
        let fine = preset_for(1, &cur);
        assert_eq!(
            (fine.name.as_str(), fine.px_per_mm, fine.check_milling),
            ("fine_pitch", 91.0, false)
        );
        assert!(fine.hole_pos_tol_mm < Profile::default().hole_pos_tol_mm);
        assert_eq!(preset_for(9, &cur).name, "prototype");
    }

    #[test]
    fn every_finding_of_the_analyser_has_words() {
        for code in [
            "hole_missing",
            "hole_wrong_size",
            "hole_off_centre",
            "hole_not_round",
            "hole_debris",
            "hole_out_of_view",
            "mill_no_channels",
            "mill_too_narrow",
            "mill_too_wide",
            "mill_pinched",
            "mill_uneven",
            "image_too_dark",
            "image_too_bright",
            "image_low_contrast",
            "image_blurry",
            "not_calibrated",
            "nothing_to_check",
            "no_expected_holes",
        ] {
            assert!(finding_text(code).is_some(), "{code}");
        }
        assert!(finding_text("something_new").is_none());
    }

    /// A board as the camera sees it, as a JPEG: grey with a dark hole of 0.8 mm in the centre at 50 pixels to the millimetre, or (when it is not there) a big dark spot
    /// far from the centre, so that the picture still has contrast and the analyser judges it.
    fn board_jpeg(with_hole: bool) -> Vec<u8> {
        let (w, h) = (320usize, 240usize);
        let mut rgb = vec![170u8; w * h * 3];
        let (cx, cy, r) = if with_hole {
            (160.0, 120.0, 20.0)
        } else {
            (270.0, 120.0, 30.0)
        };
        for y in 0..h {
            for x in 0..w {
                if (x as f64 - cx).hypot(y as f64 - cy) <= r {
                    let i = (y * w + x) * 3;
                    rgb[i..i + 3].copy_from_slice(&[25, 25, 25]);
                }
            }
        }
        let mut out = Vec::new();
        jpeg_encoder::Encoder::new(&mut out, 95)
            .encode(&rgb, w as u16, h as u16, jpeg_encoder::ColorType::Rgb)
            .unwrap();
        out
    }

    /// A node with the ring and a camera; it keeps the first line of each request.
    fn fake_node(picture: Vec<u8>) -> (u16, Arc<Mutex<Vec<String>>>) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = seen.clone();
        std::thread::spawn(move || {
            for conn in listener.incoming() {
                let Ok(mut c) = conn else { continue };
                let mut buf = [0u8; 2048];
                let n = c.read(&mut buf).unwrap_or(0);
                let line = String::from_utf8_lossy(&buf[..n])
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string();
                log.lock().unwrap().push(line.clone());
                let reply: Vec<u8> = if line.starts_with("GET /api/snapshot") {
                    let mut r = b"HTTP/1.0 200 OK\r\nContent-Type: image/jpeg\r\n\r\n".to_vec();
                    r.extend_from_slice(&picture);
                    r
                } else {
                    br#"HTTP/1.0 200 OK
Content-Type: application/json

{"light":true,"on":false,"level":60,"max_level":80,"tone":"white","sectors":255,"leds":8,"auto_off_s":-1,"color":"ffffff","cue":"none"}"#
                        .iter()
                        .flat_map(|&b| if b == b'\n' { vec![b'\r', b'\n'] } else { vec![b] })
                        .collect()
                };
                let _ = c.write_all(&reply);
            }
        });
        (port, seen)
    }

    #[test]
    fn the_check_lights_the_ring_looks_judges_and_shows_the_verdict() {
        let (port, seen) = fake_node(board_jpeg(true));
        let link = Link::new("127.0.0.1", port, "headtoken12");
        let p = Profile {
            px_per_mm: 50.0,
            check_milling: false,
            ..Profile::default()
        };
        let hole = [ExpectedHole {
            x_mm: 0.0,
            y_mm: 0.0,
            dia_mm: 0.8,
        }];
        let r = run_check(&link, &p, &hole, Some(60)).unwrap();
        assert_eq!(r.verdict, Verdict::Ok, "{:?}", r.findings);
        {
            let lines = seen.lock().unwrap();
            assert!(lines.iter().any(|l| l.starts_with("POST /api/light?on=1&level=60&tone=white&sectors=all ")), "{lines:?}");
            assert!(lines
                .iter()
                .any(|l| l.starts_with("POST /api/light?cue=ok ")));
            // the light was off before: it is put off again
            assert!(lines.iter().any(|l| l.starts_with("POST /api/light?on=0 ")));
        }
        // a hole that is not there: not ok, and red
        let (port, seen) = fake_node(board_jpeg(false));
        let link = Link::new("127.0.0.1", port, "headtoken12");
        let r = run_check(&link, &p, &hole, None).unwrap();
        assert_eq!(r.verdict, Verdict::NotOk);
        let lines = seen.lock().unwrap();
        assert!(lines
            .iter()
            .any(|l| l.starts_with("POST /api/light?cue=nok ")));
        // no light was asked for: the ring was not touched for the picture
        assert!(!lines.iter().any(|l| l.contains("tone=white")));
    }

    #[test]
    fn a_node_that_does_not_answer_is_an_error_not_a_hang() {
        let free = std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let link = Link::new("127.0.0.1", free, "t");
        assert!(run_check(&link, &Profile::default(), &[], None).is_err());
    }
}
