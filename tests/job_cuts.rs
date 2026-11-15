//! A drilling job that is cut in the middle (the computer goes off, the window is closed, the link to the machine goes down) and the way back: the file that is left says
//! what was done, the job goes on from there and drills no hole twice, a half-written file leaves the last good one, a job whose files changed is not resumed, and a
//! damaged file is never taken for a job and never makes the window panic. The job is a twin: it counts holes and saves its state as the window does. The cuts come from a
//! seeded generator, so every run is the same. Nothing here has driven a machine.

use eh3d_bungard_gui::recovery::{fingerprint, Recovery};
use std::path::{Path, PathBuf};

/// A small deterministic generator (SplitMix64): the same seed always gives the same numbers.
struct Seeded(u64);

impl Seeded {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

/// A folder of its own for each test, removed at the end.
struct Folder(PathBuf);

impl Folder {
    fn new(name: &str) -> Folder {
        let path =
            std::env::temp_dir().join(format!("eh3d-job-cuts-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Folder(path)
    }
    fn file(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The holes of a board: quarter-millimetre positions, so that they are written and read back exactly.
fn holes(seed: u64, count: usize) -> Vec<(f64, f64)> {
    let mut rng = Seeded(seed);
    (0..count)
        .map(|_| (rng.below(800) as f64 / 4.0, rng.below(600) as f64 / 4.0))
        .collect()
}

fn base(drill: &Path, total: usize, fingerprint: &str) -> Recovery {
    Recovery {
        program: "drill".into(),
        drill: Some(drill.to_path_buf()),
        columns: 1,
        rows: 1,
        mode: "gap".into(),
        total,
        fingerprint: fingerprint.into(),
        ..Recovery::default()
    }
}

/// The job: drills the holes from `from` up to (not including) `until`, saving its state after each one as the window does. Returns the holes it drilled.
fn run(
    all: &[(f64, f64)],
    from: usize,
    until: usize,
    state: &mut Recovery,
    file: &Path,
) -> Vec<(f64, f64)> {
    let mut drilled = Vec::new();
    for (i, hole) in all.iter().enumerate().take(until).skip(from) {
        drilled.push(*hole);
        state.done = i + 1;
        state.drilled.push(*hole);
        state.save(file);
    }
    drilled
}

#[test]
fn a_job_cut_at_any_hole_goes_on_from_the_saved_state_and_drills_nothing_twice() {
    let folder = Folder::new("cuts");
    let file = folder.file("recovery.cfg");
    let board = folder.file("board.ncd");
    std::fs::write(&board, "T01\nX100Y100\n").unwrap();
    let mut rng = Seeded(2024);
    for seed in 0..25 {
        let all = holes(seed, 40);
        let cut = 1 + rng.below(39) as usize;
        Recovery::clear(&file);
        let mut state = base(&board, all.len(), "00aa00aa00aa00aa");
        let first = run(&all, 0, cut, &mut state, &file);
        // the window is gone; at the next start the file tells what was done
        let mut found = Recovery::load(&file).expect("a job was left");
        assert_eq!((found.done, found.total), (cut, 40), "seed {seed}");
        assert_eq!(found.drilled, first);
        let second = run(&all, found.done, all.len(), &mut found, &file);
        let mut everything = first;
        everything.extend(second);
        assert_eq!(
            everything, all,
            "seed {seed}: a hole was missed or drilled twice"
        );
        // the job ends and the file goes
        Recovery::clear(&file);
        assert!(Recovery::load(&file).is_none());
    }
}

#[test]
fn a_write_that_was_cut_leaves_the_last_good_state() {
    let folder = Folder::new("torn");
    let file = folder.file("recovery.cfg");
    let board = folder.file("board.ncd");
    std::fs::write(&board, "T01\n").unwrap();
    let all = holes(5, 20);
    let mut state = base(&board, 20, "1111222233334444");
    run(&all, 0, 12, &mut state, &file);
    let good = Recovery::load(&file).unwrap();
    assert_eq!(good.done, 12);
    // the next save was cut after a part of the side file was written and before it replaced the file
    let mut next = good.clone();
    next.done = 13;
    next.drilled.push(all[12]);
    let text = next.to_text();
    std::fs::write(file.with_extension("tmp"), &text[..text.len() / 2]).unwrap();
    assert_eq!(Recovery::load(&file), Some(good));
    // the leftover is cleared with the file
    Recovery::clear(&file);
    assert!(!file.with_extension("tmp").exists());
    assert!(Recovery::load(&file).is_none());
}

#[test]
fn a_job_whose_files_or_settings_changed_is_not_resumed() {
    let folder = Folder::new("changed");
    let board = folder.file("board.ncd");
    std::fs::write(&board, "T01\nX100Y100\nX200Y100\n").unwrap();
    let file = folder.file("recovery.cfg");
    let settings = "mils|special|area 0,0,300,200";
    let was = fingerprint(&[board.as_path()], settings);
    let all = holes(8, 30);
    let mut state = base(&board, 30, &was);
    run(&all, 0, 11, &mut state, &file);
    let found = Recovery::load(&file).unwrap();
    let may_resume = |r: &Recovery, now: &str| r.fingerprint == now;
    assert!(may_resume(
        &found,
        &fingerprint(&[board.as_path()], settings)
    ));
    // a setting that decides the order of the holes
    assert!(!may_resume(
        &found,
        &fingerprint(&[board.as_path()], "mils|x first|area 0,0,300,200")
    ));
    // the file of the job edited between the cut and the restart
    std::fs::write(&board, "T01\nX100Y100\nX200Y101\n").unwrap();
    assert!(!may_resume(
        &found,
        &fingerprint(&[board.as_path()], settings)
    ));
    // the way back: leave the old job and start again from the first hole
    Recovery::clear(&file);
    let mut again = base(&board, 30, &fingerprint(&[board.as_path()], settings));
    let drilled = run(&all, 0, all.len(), &mut again, &file);
    assert_eq!(drilled, all);
    assert_eq!(Recovery::load(&file).unwrap().done, 30);
}

#[test]
fn a_damaged_file_never_panics_and_what_is_accepted_reads_back_the_same() {
    let folder = Folder::new("damaged");
    let board = folder.file("board.ncd");
    std::fs::write(&board, "T01\n").unwrap();
    let all = holes(3, 15);
    let mut state = base(&board, 15, "abcdefabcdefabcd");
    state.queue.push("route".into());
    state.off.push(("drill".into(), 2));
    run(&all, 0, 15, &mut state, &folder.file("r.cfg"));
    let text = state.to_text();
    let bytes = text.as_bytes();
    let mut rng = Seeded(99);
    let mut accepted = 0;
    for _ in 0..400 {
        let mut damaged = bytes.to_vec();
        match rng.below(3) {
            0 => damaged.truncate(rng.below(bytes.len() as u64) as usize),
            1 => {
                let at = rng.below(bytes.len() as u64) as usize;
                damaged[at] = rng.below(128) as u8;
            }
            _ => {
                let at = rng.below(bytes.len() as u64) as usize;
                damaged.drain(at..(at + 7).min(bytes.len()));
            }
        }
        let text = String::from_utf8_lossy(&damaged).into_owned();
        let first = Recovery::from_text(&text);
        assert_eq!(first, Recovery::from_text(&text));
        if let Some(r) = first {
            accepted += 1;
            // what is accepted is a job: it can be saved and read again, and saving it again changes nothing
            let once =
                Recovery::from_text(&r.to_text()).expect("an accepted job is saved as a job");
            assert_eq!(Recovery::from_text(&once.to_text()), Some(once));
        }
    }
    assert!(
        accepted > 0,
        "no damaged file was accepted: the test does not try anything"
    );
}

#[test]
fn a_file_that_is_not_there_or_is_empty_is_no_job() {
    let folder = Folder::new("none");
    let file = folder.file("recovery.cfg");
    assert!(Recovery::load(&file).is_none());
    std::fs::write(&file, "").unwrap();
    assert!(Recovery::load(&file).is_none());
    std::fs::write(&file, [0xFF, 0xFE, 0x00, 0x41]).unwrap();
    assert!(Recovery::load(&file).is_none());
}
