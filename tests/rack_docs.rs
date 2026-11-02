//! The README documents every key of the tool changer with the value the code uses when the machine file has none.

use eh3d_bungard_formats::config::{RackSetup, RACK_KEYS};

fn readme(name: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name))
        .expect("the README is there")
}

#[test]
fn every_readme_lists_the_rack_keys_with_their_defaults() {
    let d = RackSetup::default();
    let values = [
        d.tool_length_mm,
        d.ring_to_tip_mm,
        d.clearance_mm,
        d.approach_mm,
        d.reach_x_mm,
    ];
    for name in [
        "README.md",
        "README_spa.md",
        "README_ita.md",
        "README_deu.md",
        "README_fra.md",
        "README_por.md",
        "README_zho.md",
        "README_jpn.md",
    ] {
        let text = readme(name);
        for (key, value) in RACK_KEYS.iter().zip(values) {
            // the value follows the key in brackets, in both kinds of bracket
            let plain = format!("{value}");
            let wanted = [format!("`{key}`（{plain}"), format!("`{key}` ({plain}")];
            assert!(
                wanted.iter().any(|w| text.contains(w.as_str())),
                "{name} does not give {key} with its default {plain}"
            );
        }
    }
}

#[test]
fn a_file_without_rack_lines_gives_the_documented_defaults() {
    let none: Vec<String> = vec!["1.0 2.0 3.0".to_string(), "text".to_string()];
    assert_eq!(RackSetup::from_lines(none.iter()), RackSetup::default());
}
