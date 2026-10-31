//! The folder of the spoken notices: every voice is a complete set of four recordings, the languages that were added later are offered in the lists, and each
//! voice is named, with its licence, in the files that say where it comes from.

use eh3d_drillpro::protocol::sound::{available_voices, voice_file, Cue, VOICE_LANGS};
use std::path::{Path, PathBuf};

fn voices() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("voices")
}

/// The length in seconds of a WAV file of 16 bits and one channel, read from its header.
fn seconds(path: &Path) -> f64 {
    let b = std::fs::read(path).unwrap();
    assert!(b.len() > 44 && &b[0..4] == b"RIFF" && &b[8..12] == b"WAVE", "{path:?}");
    let channels = u16::from_le_bytes([b[22], b[23]]);
    let rate = u32::from_le_bytes([b[24], b[25], b[26], b[27]]);
    let bits = u16::from_le_bytes([b[34], b[35]]);
    assert_eq!((channels, bits), (1, 16), "{path:?} is not 16 bits and mono");
    (b.len() - 44) as f64 / (rate as f64 * 2.0)
}

#[test]
fn every_voice_has_its_four_recordings_of_a_sensible_length() {
    let mut found = 0;
    for lang in std::fs::read_dir(voices()).unwrap().flatten() {
        if !lang.path().is_dir() {
            continue;
        }
        let code = lang.file_name().to_string_lossy().to_string();
        assert!(VOICE_LANGS.contains(&code.as_str()), "{code} is not a language of the voices");
        for gender in std::fs::read_dir(lang.path()).unwrap().flatten() {
            let female = match gender.file_name().to_string_lossy().as_ref() {
                "f" => true,
                "m" => false,
                other => panic!("{code}/{other} is not f or m"),
            };
            for cue in [Cue::Finished, Cue::ToolChange, Cue::CheckOk, Cue::CheckNotOk] {
                let file = voice_file(&voices(), &code, female, cue)
                    .unwrap_or_else(|| panic!("{code}/{} lacks a recording", gender.file_name().to_string_lossy()));
                let s = seconds(&file);
                assert!((1.0..=6.0).contains(&s), "{file:?} lasts {s} seconds");
                assert!(std::fs::metadata(&file).unwrap().len() < 400_000, "{file:?} is too big");
            }
            found += 1;
        }
    }
    assert!(found >= 30, "{found} voices");
}

#[test]
fn the_voices_of_the_new_languages_are_offered() {
    let offered = available_voices(&voices());
    for (lang, female) in [("cs", false), ("pl", false), ("uk", true), ("pt-br", false), ("el", true), ("ru", false)] {
        assert!(offered.contains(&(lang, female)), "{lang} {female}: {offered:?}");
    }
    // a language without a voice is not in the list, so it falls to the tone
    assert!(!offered.iter().any(|(l, _)| ["hr", "ga", "mt", "gl", "ca", "sr", "is", "lt"].contains(l)));
}

#[test]
fn every_voice_is_named_with_its_licence_in_the_files_that_say_where_it_comes_from() {
    let readme = std::fs::read_to_string(voices().join("README.txt")).unwrap();
    let sources = std::fs::read_to_string(voices().join("SOURCES.txt")).unwrap();
    for lang in std::fs::read_dir(voices()).unwrap().flatten() {
        if !lang.path().is_dir() {
            continue;
        }
        let code = lang.file_name().to_string_lossy().to_string();
        for gender in std::fs::read_dir(lang.path()).unwrap().flatten() {
            let line = format!("{code}/{}", gender.file_name().to_string_lossy());
            assert!(readme.contains(&line), "{line} is not in README.txt");
            assert!(sources.contains(&line), "{line} is not in SOURCES.txt");
        }
    }
}
