//! The place of each tool in the bar of the tool changer, without a window. The tools of a job can be in the bar in any order (the bar may be loaded as the person finds
//! it handy), so each tool of the job has the place the person gave it: the window asks this module and keeps only the drawing and the buttons.
//!
//! A tool is told by its program (drill or router) and its number; its size (the diameter of a drill, in micrometres; zero when it is not known, as for a cutter) is what
//! lets the places be remembered for the next job that uses the same tools, which may number them differently.

use crate::launcher::Program;

/// The places of the bar that can hold a tool: 1 to 14 (the place 0 is the test pin, 15 the detector of a broken drill).
pub const MAX_TOOL_SLOT: usize = 14;

/// The version of the text the places are saved in; a text of another version is not read.
const VERSION: &str = "v1";

/// How many remembered places are kept for a machine.
const BOOK_MAX: usize = 200;

/// Where each tool of the job is: the program, the tool number and its place.
pub type Assignment = Vec<(Program, u8, usize)>;

/// A tool of the job: its program, its number and its size (see the top of this file).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JobTool {
    pub prog: Program,
    pub tool: u8,
    pub diameter_um: u32,
}

/// What stops a job from starting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// The tool has no place.
    Unassigned(Program, u8),
    /// The place of the tool is not defined in the machine file (the place).
    Undefined(Program, u8, usize),
    /// Two or more tools are in the same place (the place and the tools).
    Shared(usize, Vec<(Program, u8)>),
}

/// The place of a tool, if it has one.
pub fn slot_of(a: &Assignment, prog: Program, tool: u8) -> Option<usize> {
    a.iter()
        .find(|(p, t, _)| *p == prog && *t == tool)
        .map(|(_, _, s)| *s)
}

/// Gives a tool a place, or takes its place away (`None`).
pub fn set(a: &mut Assignment, prog: Program, tool: u8, slot: Option<usize>) {
    a.retain(|(p, t, _)| !(*p == prog && *t == tool));
    if let Some(s) = slot {
        a.push((prog, tool, s));
    }
}

/// The places a tool can be given: the ones the machine file defines, from 1 to [`MAX_TOOL_SLOT`], in order.
pub fn selectable(defined: &[usize]) -> Vec<usize> {
    let mut v: Vec<usize> = defined
        .iter()
        .copied()
        .filter(|s| (1..=MAX_TOOL_SLOT).contains(s))
        .collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// What is wrong with the places of the tools of the job: the tools with no place, the places the machine file does not define and the places that two tools share.
/// Empty: the job can start.
pub fn check(tools: &[JobTool], a: &Assignment, defined: &[usize]) -> Vec<Problem> {
    let ok = selectable(defined);
    let mut out = Vec::new();
    let mut by_slot: Vec<(usize, Vec<(Program, u8)>)> = Vec::new();
    for t in tools {
        match slot_of(a, t.prog, t.tool) {
            None => out.push(Problem::Unassigned(t.prog, t.tool)),
            Some(s) => {
                if !ok.contains(&s) {
                    out.push(Problem::Undefined(t.prog, t.tool, s));
                }
                match by_slot.iter_mut().find(|(k, _)| *k == s) {
                    Some((_, who)) => who.push((t.prog, t.tool)),
                    None => by_slot.push((s, vec![(t.prog, t.tool)])),
                }
            }
        }
    }
    out.extend(
        by_slot
            .into_iter()
            .filter(|(_, who)| who.len() > 1)
            .map(|(s, who)| Problem::Shared(s, who)),
    );
    out
}

/// Gives a place to each tool that has none, in order: the drills before the cutters, smaller before bigger, and the places that are free from the lowest. A tool that
/// already has a place keeps it; when the places run out the tools that are left stay without one.
pub fn fill_in_order(tools: &[JobTool], a: &mut Assignment, defined: &[usize]) {
    let mut free: Vec<usize> = selectable(defined)
        .into_iter()
        .filter(|s| !tools.iter().any(|t| slot_of(a, t.prog, t.tool) == Some(*s)))
        .collect();
    free.reverse();
    let mut todo: Vec<&JobTool> = tools
        .iter()
        .filter(|t| slot_of(a, t.prog, t.tool).is_none())
        .collect();
    todo.sort_by_key(|t| (t.prog == Program::Route, t.diameter_um, t.tool));
    for t in todo {
        let Some(s) = free.pop() else { break };
        set(a, t.prog, t.tool, Some(s));
    }
}

/// The places of the tools of one program, as the planner takes them: the tool number and its place.
pub fn engine_slots(a: &Assignment, prog: Program) -> Vec<(u8, usize)> {
    a.iter()
        .filter(|(p, _, _)| *p == prog)
        .map(|(_, t, s)| (*t, *s))
        .collect()
}

/// A place remembered for a tool: by its kind, its size and its number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Entry {
    prog: Program,
    diameter_um: u32,
    tool: u8,
    slot: usize,
}

/// The places the person gave to the tools of one machine, kept between runs so that the next job with the same tools finds them where they were put.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Book {
    entries: Vec<Entry>,
}

fn kind_char(p: Program) -> char {
    match p {
        Program::Drill => 'd',
        Program::Route => 'r',
    }
}

impl Book {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The text of the settings file: `v1;d800#1=3,r0#7=9` (kind, size in micrometres, `#`, tool number, `=`, place). Empty when nothing is remembered.
    pub fn to_text(&self) -> String {
        if self.entries.is_empty() {
            return String::new();
        }
        let items: Vec<String> = self
            .entries
            .iter()
            .map(|e| {
                format!(
                    "{}{}#{}={}",
                    kind_char(e.prog),
                    e.diameter_um,
                    e.tool,
                    e.slot
                )
            })
            .collect();
        format!("{VERSION};{}", items.join(","))
    }

    /// Reads the text of the settings file. A text of another version, or without a version, is not read at all; of a text of this version what is not valid is left
    /// out (a place must be from 1 to 99).
    pub fn parse(text: &str) -> Book {
        let Some(body) = text
            .trim()
            .strip_prefix(VERSION)
            .and_then(|r| r.strip_prefix(';'))
        else {
            return Book::default();
        };
        let entries = body
            .split(',')
            .filter_map(|tok| {
                let tok = tok.trim();
                let prog = match tok.chars().next()? {
                    'd' => Program::Drill,
                    'r' => Program::Route,
                    _ => return None,
                };
                let (left, slot) = tok[1..].split_once('=')?;
                let (diam, tool) = left.split_once('#')?;
                let slot: usize = slot.trim().parse().ok().filter(|s| (1..=99).contains(s))?;
                Some(Entry {
                    prog,
                    diameter_um: diam.trim().parse().ok()?,
                    tool: tool.trim().parse().ok()?,
                    slot,
                })
            })
            .take(BOOK_MAX)
            .collect();
        Book { entries }
    }

    /// Remembers the places of the tools of the job that have one. A place that was remembered for a tool of another size is forgotten (the tool that is in the place
    /// now is this one).
    pub fn remember(&mut self, tools: &[JobTool], a: &Assignment) {
        for t in tools {
            let Some(slot) = slot_of(a, t.prog, t.tool) else {
                continue;
            };
            self.entries.retain(|e| {
                let same_tool =
                    e.prog == t.prog && e.diameter_um == t.diameter_um && e.tool == t.tool;
                let same_size =
                    e.prog == t.prog && e.diameter_um == t.diameter_um && t.diameter_um > 0;
                !(same_tool || (e.slot == slot && !same_size))
            });
            self.entries.push(Entry {
                prog: t.prog,
                diameter_um: t.diameter_um,
                tool: t.tool,
                slot,
            });
        }
        let extra = self.entries.len().saturating_sub(BOOK_MAX);
        self.entries.drain(..extra);
    }

    /// Forgets the places of these tools: what is remembered for their size, and for their number when the size is the same or is not known.
    pub fn forget(&mut self, tools: &[JobTool]) {
        self.entries.retain(|e| {
            !tools.iter().any(|t| {
                e.prog == t.prog
                    && ((t.diameter_um > 0 && e.diameter_um == t.diameter_um)
                        || (e.tool == t.tool
                            && (e.diameter_um == t.diameter_um
                                || e.diameter_um == 0
                                || t.diameter_um == 0)))
            })
        });
    }

    /// The places for the tools of a job from what was remembered: by kind and size when the size is known and no other tool of the job has it, and by the number of
    /// the tool (with the same size, or when either size is not known) otherwise. Only the places the machine file defines are given.
    pub fn recall(&self, tools: &[JobTool], defined: &[usize]) -> Assignment {
        let ok = selectable(defined);
        let mut out: Assignment = Vec::new();
        for t in tools {
            let mine = self.entries.iter().filter(|e| e.prog == t.prog);
            let twins = tools
                .iter()
                .filter(|o| o.prog == t.prog && o.diameter_um == t.diameter_um)
                .count();
            let by_size = if t.diameter_um > 0 && twins == 1 {
                let same: Vec<&Entry> = mine
                    .clone()
                    .filter(|e| e.diameter_um == t.diameter_um && ok.contains(&e.slot))
                    .collect();
                same.iter()
                    .find(|e| e.tool == t.tool)
                    .or(same.first())
                    .map(|e| e.slot)
            } else {
                None
            };
            let found = by_size.or_else(|| {
                mine.filter(|e| {
                    e.tool == t.tool
                        && ok.contains(&e.slot)
                        && (e.diameter_um == t.diameter_um
                            || e.diameter_um == 0
                            || t.diameter_um == 0)
                })
                .map(|e| e.slot)
                .next_back()
            });
            if let Some(s) = found {
                set(&mut out, t.prog, t.tool, Some(s));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drill(tool: u8, um: u32) -> JobTool {
        JobTool {
            prog: Program::Drill,
            tool,
            diameter_um: um,
        }
    }

    fn cutter(tool: u8) -> JobTool {
        JobTool {
            prog: Program::Route,
            tool,
            diameter_um: 0,
        }
    }

    /// The bar of the machine: the test pin and fourteen places.
    fn bar() -> Vec<usize> {
        (0..=16).collect()
    }

    fn job() -> Vec<JobTool> {
        vec![drill(1, 800), drill(2, 1000), drill(3, 600), cutter(7)]
    }

    #[test]
    fn a_job_with_every_tool_in_a_place_of_its_own_can_start() {
        let mut a: Assignment = Vec::new();
        for (t, s) in job().iter().zip([5, 2, 9, 14]) {
            set(&mut a, t.prog, t.tool, Some(s));
        }
        assert!(check(&job(), &a, &bar()).is_empty());
        assert_eq!(slot_of(&a, Program::Route, 7), Some(14));
        // the places go to the planner by program, with the number of the tool
        assert_eq!(
            engine_slots(&a, Program::Drill),
            vec![(1, 5), (2, 2), (3, 9)]
        );
        assert_eq!(engine_slots(&a, Program::Route), vec![(7, 14)]);
    }

    #[test]
    fn a_tool_with_no_place_two_in_one_and_a_place_the_machine_does_not_define_are_told() {
        let mut a: Assignment = Vec::new();
        set(&mut a, Program::Drill, 1, Some(5));
        set(&mut a, Program::Drill, 2, Some(5));
        set(&mut a, Program::Drill, 3, Some(20));
        let p = check(&job(), &a, &bar());
        assert!(p.contains(&Problem::Unassigned(Program::Route, 7)));
        assert!(p.contains(&Problem::Undefined(Program::Drill, 3, 20)));
        assert!(p.contains(&Problem::Shared(
            5,
            vec![(Program::Drill, 1), (Program::Drill, 2)]
        )));
        assert_eq!(p.len(), 3);
        // a place that is defined in the file of the machine only from the third one on is not one the first two can have
        let mut b: Assignment = Vec::new();
        set(&mut b, Program::Drill, 1, Some(1));
        let few = [0usize, 2, 3, 4];
        assert_eq!(
            check(&[drill(1, 800)], &b, &few),
            vec![Problem::Undefined(Program::Drill, 1, 1)]
        );
        // the place 0 (the test pin) and the place 15 (the detector) are not places for a tool
        assert_eq!(selectable(&bar()), (1..=14).collect::<Vec<_>>());
        assert_eq!(selectable(&few), vec![2, 3, 4]);
    }

    #[test]
    fn a_tool_is_given_a_place_and_loses_it() {
        let mut a: Assignment = Vec::new();
        set(&mut a, Program::Drill, 1, Some(3));
        set(&mut a, Program::Drill, 1, Some(4));
        assert_eq!(a, vec![(Program::Drill, 1, 4)]);
        // the drill 1 and the cutter 1 are not the same tool
        set(&mut a, Program::Route, 1, Some(6));
        assert_eq!(slot_of(&a, Program::Drill, 1), Some(4));
        assert_eq!(slot_of(&a, Program::Route, 1), Some(6));
        set(&mut a, Program::Drill, 1, None);
        assert_eq!(slot_of(&a, Program::Drill, 1), None);
        assert_eq!(a.len(), 1);
    }

    #[test]
    fn assigning_in_order_fills_the_free_places_drills_first_and_smaller_first() {
        let mut a: Assignment = Vec::new();
        // the cutter has the place 1 already
        set(&mut a, Program::Route, 7, Some(1));
        fill_in_order(&job(), &mut a, &bar());
        // drills by size: 0.6 (tool 3), 0.8 (tool 1), 1.0 (tool 2) in the free places 2, 3, 4
        assert_eq!(slot_of(&a, Program::Drill, 3), Some(2));
        assert_eq!(slot_of(&a, Program::Drill, 1), Some(3));
        assert_eq!(slot_of(&a, Program::Drill, 2), Some(4));
        assert_eq!(slot_of(&a, Program::Route, 7), Some(1));
        assert!(check(&job(), &a, &bar()).is_empty());
        // run again: nothing moves
        let before = a.clone();
        fill_in_order(&job(), &mut a, &bar());
        assert_eq!(a, before);
        // with no assignment at all the cutters come after the drills
        let mut b: Assignment = Vec::new();
        fill_in_order(&job(), &mut b, &bar());
        assert_eq!(slot_of(&b, Program::Route, 7), Some(4));
        // when the places run out the tools that are left have none
        let mut c: Assignment = Vec::new();
        fill_in_order(&job(), &mut c, &[0, 1, 2]);
        assert_eq!(c.len(), 2);
        assert_eq!(slot_of(&c, Program::Route, 7), None);
    }

    #[test]
    fn what_was_put_in_the_bar_comes_back_through_the_text_of_the_settings() {
        let mut a: Assignment = Vec::new();
        for (t, s) in job().iter().zip([5, 2, 9, 14]) {
            set(&mut a, t.prog, t.tool, Some(s));
        }
        let mut book = Book::default();
        assert!(book.is_empty() && book.to_text().is_empty());
        book.remember(&job(), &a);
        let text = book.to_text();
        assert_eq!(text, "v1;d800#1=5,d1000#2=2,d600#3=9,r0#7=14");
        let back = Book::parse(&text);
        assert_eq!(back, book);
        // the same job finds every tool where it was put
        let mut got = back.recall(&job(), &bar());
        got.sort_by_key(|(_, t, _)| *t);
        let mut want = a.clone();
        want.sort_by_key(|(_, t, _)| *t);
        assert_eq!(got, want);
    }

    #[test]
    fn the_places_are_found_again_by_size_when_the_next_job_numbers_the_tools_differently() {
        let mut a: Assignment = Vec::new();
        for (t, s) in job().iter().zip([5, 2, 9, 14]) {
            set(&mut a, t.prog, t.tool, Some(s));
        }
        let mut book = Book::default();
        book.remember(&job(), &a);
        // the same drills with other numbers, and one more that is new
        let next = [drill(4, 600), drill(5, 800), drill(6, 1200), cutter(7)];
        let got = book.recall(&next, &bar());
        assert_eq!(slot_of(&got, Program::Drill, 4), Some(9));
        assert_eq!(slot_of(&got, Program::Drill, 5), Some(5));
        assert_eq!(slot_of(&got, Program::Drill, 6), None);
        // a cutter is found by its number
        assert_eq!(slot_of(&got, Program::Route, 7), Some(14));
        assert_eq!(
            slot_of(&book.recall(&[cutter(8)], &bar()), Program::Route, 8),
            None
        );
    }

    #[test]
    fn two_tools_of_one_size_are_told_apart_by_their_numbers_and_a_tool_of_another_size_does_not_take_a_number(
    ) {
        let twins = [drill(1, 800), drill(2, 800)];
        let mut a: Assignment = Vec::new();
        set(&mut a, Program::Drill, 1, Some(3));
        set(&mut a, Program::Drill, 2, Some(6));
        let mut book = Book::default();
        book.remember(&twins, &a);
        let got = book.recall(&twins, &bar());
        assert_eq!(slot_of(&got, Program::Drill, 1), Some(3));
        assert_eq!(slot_of(&got, Program::Drill, 2), Some(6));
        // the drill 1 of the next job is 1.0 mm and not 0.8: it is not where the 0.8 mm was put
        let got = book.recall(&[drill(1, 1000)], &bar());
        assert!(got.is_empty());
    }

    #[test]
    fn a_place_taken_by_another_size_is_forgotten_and_a_place_the_machine_lost_is_not_given() {
        let mut book = Book::default();
        let mut a: Assignment = Vec::new();
        set(&mut a, Program::Drill, 1, Some(3));
        book.remember(&[drill(1, 800)], &a);
        // now a 1.0 mm drill is in the place 3
        set(&mut a, Program::Drill, 1, Some(3));
        book.remember(&[drill(1, 1000)], &a);
        assert_eq!(book.to_text(), "v1;d1000#1=3");
        assert!(book.recall(&[drill(2, 800)], &bar()).is_empty());
        // a place that the file of the machine no longer defines
        assert!(book.recall(&[drill(1, 1000)], &[0, 1, 2]).is_empty());
    }

    #[test]
    fn a_tool_that_is_given_no_place_is_forgotten_and_the_others_are_not() {
        let mut a: Assignment = Vec::new();
        for (t, s) in job().iter().zip([5, 2, 9, 14]) {
            set(&mut a, t.prog, t.tool, Some(s));
        }
        let mut book = Book::default();
        book.remember(&job(), &a);
        book.forget(&[drill(1, 800), cutter(7)]);
        assert_eq!(book.to_text(), "v1;d1000#2=2,d600#3=9");
        // by size: another tool of the same size is not remembered either
        book.forget(&[drill(5, 600)]);
        assert_eq!(book.to_text(), "v1;d1000#2=2");
        // a drill with the number of another size is not the one that is forgotten
        book.forget(&[drill(2, 800)]);
        assert_eq!(book.to_text(), "v1;d1000#2=2");
        book.forget(&[drill(2, 1000)]);
        assert!(book.is_empty());
    }

    #[test]
    fn a_text_of_another_version_or_with_nonsense_is_not_trusted() {
        assert!(Book::parse("").is_empty());
        assert!(Book::parse("d800#1=5").is_empty());
        assert!(Book::parse("v2;d800#1=5").is_empty());
        assert!(Book::parse("v1").is_empty());
        // what is not valid is left out, what is valid stays
        let b = Book::parse("v1;d800#1=5,x1#1=2,d800#2=0,d800#3=100,dA#1=2,r0#7=14,d600#4");
        assert_eq!(b.to_text(), "v1;d800#1=5,r0#7=14");
        // spaces are forgiven
        assert_eq!(
            Book::parse("  v1; d800#1=5 , r0#7=14 ").to_text(),
            "v1;d800#1=5,r0#7=14"
        );
    }
}
