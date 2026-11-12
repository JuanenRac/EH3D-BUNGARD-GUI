//! The order of a job that drills and routes, without a window: how the tools of both programs are shared out in the loads of the tool
//! rack and which slot holds which tool. The window asks this module and keeps only the drawing and the buttons.

use crate::launcher::Program;

/// The tools of one program that go into the rack together, in the order the job uses them.
pub type Piece = (Program, Vec<u8>);

/// The loads of the rack for the tools each program has to do (in the order of the job): every program's tools in pieces that fit the rack, and
/// consecutive pieces together while they fit. A program with no tools is left out, and a rack of no places is taken as one of a single place.
pub fn plan_loads(tools: &[Piece], places: usize) -> Vec<Vec<Piece>> {
    let cap = places.max(1);
    let pieces = tools
        .iter()
        .flat_map(|(p, ts)| ts.chunks(cap).map(move |c| (*p, c.to_vec())));
    let mut loads: Vec<Vec<Piece>> = Vec::new();
    let (mut cur, mut n): (Vec<Piece>, usize) = (Vec::new(), 0);
    for piece in pieces {
        if n + piece.1.len() > cap && !cur.is_empty() {
            loads.push(std::mem::take(&mut cur));
            n = 0;
        }
        n += piece.1.len();
        cur.push(piece);
    }
    if !cur.is_empty() {
        loads.push(cur);
    }
    loads
}

/// The slot of each tool of the pieces of a load (slot 1 holds the first tool): the slot, the program and the tool.
pub fn slot_tools(pieces: &[Piece]) -> Vec<(u8, Program, u8)> {
    pieces
        .iter()
        .flat_map(|(p, ts)| ts.iter().map(move |&t| (*p, t)))
        .enumerate()
        .map(|(i, (p, t))| ((i + 1) as u8, p, t))
        .collect()
}

/// What a request to start a job comes to, before anything moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Start {
    /// Nothing is switched on and no file is loaded: nothing to say.
    Silent,
    /// A file is loaded but the drilling and the routing are both off.
    NothingOn,
    /// Part of the work falls outside the table: the job is refused.
    Outside,
    /// The machine has the tool changer: the places of the bar and the loads of the rack come first.
    Changer,
    /// The programs start one after the other.
    Plain,
}

/// The first decision of a start: `programs_empty` (nothing switched on), `file_loaded`, `outside` (part of the work is off the table) and `changer` (the model has the tool changer).
pub fn start_decision(
    programs_empty: bool,
    file_loaded: bool,
    outside: bool,
    changer: bool,
) -> Start {
    if programs_empty {
        if file_loaded {
            Start::NothingOn
        } else {
            Start::Silent
        }
    } else if outside {
        Start::Outside
    } else if changer {
        Start::Changer
    } else {
        Start::Plain
    }
}

/// The programs a start works on: the ones that are on, or, when it goes on from a hole already done (`skip` above 0), only the program that was running.
pub fn programs_for_start(on: Vec<Program>, running: Option<Program>, skip: usize) -> Vec<Program> {
    if skip > 0 {
        running.into_iter().collect()
    } else {
        on
    }
}

/// The state of the pause after the Pause button: it toggles while a program runs and is never on when none does.
pub fn toggle_pause(running: bool, paused: bool) -> bool {
    running && !paused
}

/// What the job does when a program of it has ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Next {
    /// The next piece of the load that is in the rack (its tools come from the places after the ones that ran).
    Piece,
    /// The rack has to be loaded again for the next load.
    Reload,
    /// The next program of the queue starts (the machine has no changer, or the rack is not in use).
    Program,
    /// There is nothing more: the job is over.
    Over,
}

/// The step after a program ended: with the rack in use the pieces of the load go first, then the next load, then the end; without it the queue of programs decides.
pub fn after_program(
    rack_active: bool,
    pieces_left: usize,
    loads_left: usize,
    queue_left: usize,
) -> Next {
    if rack_active {
        if pieces_left > 0 {
            Next::Piece
        } else if loads_left > 0 {
            Next::Reload
        } else {
            Next::Over
        }
    } else if queue_left == 0 {
        Next::Over
    } else {
        Next::Program
    }
}

/// How the chuck is handed on to the next piece of a load: the first place the next program takes its tools from (after the ones of the piece that ran) and the tool the
/// chuck holds now (the last tool of that piece).
pub fn hand_over(first: usize, ran: Option<&Piece>) -> (usize, Option<(Program, u8)>) {
    (
        first + ran.map_or(0, |(_, t)| t.len()),
        ran.and_then(|(p, t)| t.last().map(|&t| (*p, t))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drilling_and_routing_share_one_load_when_the_rack_holds_them_all() {
        let tools = [(Program::Drill, vec![1, 2, 3]), (Program::Route, vec![7])];
        let loads = plan_loads(&tools, 15);
        assert_eq!(loads, vec![tools.to_vec()]);
    }

    #[test]
    fn a_small_rack_is_loaded_in_turns_and_the_routing_goes_with_the_last_drills() {
        let tools = [(Program::Drill, vec![1, 2, 3]), (Program::Route, vec![7])];
        let loads = plan_loads(&tools, 2);
        assert_eq!(
            loads,
            vec![
                vec![(Program::Drill, vec![1, 2])],
                vec![(Program::Drill, vec![3]), (Program::Route, vec![7])],
            ]
        );
        // every tool is in a load once, in the order of the job
        let all: Vec<u8> = loads
            .iter()
            .flatten()
            .flat_map(|(_, ts)| ts.iter().copied())
            .collect();
        assert_eq!(all, vec![1, 2, 3, 7]);
    }

    #[test]
    fn a_program_without_tools_is_left_out_and_a_rack_of_no_places_holds_one_tool() {
        let tools = [(Program::Drill, vec![]), (Program::Route, vec![7])];
        assert_eq!(plan_loads(&tools, 5), vec![vec![(Program::Route, vec![7])]]);
        assert!(plan_loads(&[], 5).is_empty());
        let two = [(Program::Drill, vec![1, 2])];
        assert_eq!(plan_loads(&two, 0).len(), 2);
    }

    #[test]
    fn the_slots_are_numbered_from_one_in_the_order_of_the_job() {
        let load = [(Program::Drill, vec![4, 5]), (Program::Route, vec![9])];
        assert_eq!(
            slot_tools(&load),
            vec![
                (1, Program::Drill, 4),
                (2, Program::Drill, 5),
                (3, Program::Route, 9)
            ]
        );
        assert!(slot_tools(&[]).is_empty());
    }

    #[test]
    fn after_a_program_the_pieces_go_first_then_the_next_load_then_the_end() {
        assert_eq!(after_program(true, 2, 1, 5), Next::Piece);
        assert_eq!(after_program(true, 0, 1, 5), Next::Reload);
        assert_eq!(after_program(true, 0, 0, 5), Next::Over);
        assert_eq!(after_program(false, 0, 0, 1), Next::Program);
        assert_eq!(after_program(false, 9, 9, 0), Next::Over);
    }

    #[test]
    fn the_chuck_is_handed_on_with_the_last_tool_of_the_piece_that_ran() {
        let ran: Piece = (Program::Drill, vec![4, 5]);
        assert_eq!(hand_over(3, Some(&ran)), (5, Some((Program::Drill, 5))));
        assert_eq!(hand_over(3, None), (3, None));
        let empty: Piece = (Program::Route, vec![]);
        assert_eq!(hand_over(0, Some(&empty)), (0, None));
    }

    #[test]
    fn a_start_is_silent_refused_or_goes_to_the_changer_or_on() {
        assert_eq!(start_decision(true, false, false, true), Start::Silent);
        assert_eq!(start_decision(true, true, false, true), Start::NothingOn);
        assert_eq!(start_decision(false, true, true, true), Start::Outside);
        assert_eq!(start_decision(false, true, false, true), Start::Changer);
        assert_eq!(start_decision(false, true, false, false), Start::Plain);
    }

    #[test]
    fn going_on_from_a_hole_works_only_on_the_program_that_was_running() {
        let on = vec![Program::Drill, Program::Route];
        assert_eq!(programs_for_start(on.clone(), Some(Program::Route), 0), on);
        assert_eq!(
            programs_for_start(on.clone(), Some(Program::Route), 5),
            vec![Program::Route]
        );
        assert!(programs_for_start(on, None, 5).is_empty());
    }

    #[test]
    fn the_pause_toggles_only_while_a_program_runs() {
        assert!(toggle_pause(true, false));
        assert!(!toggle_pause(true, true));
        assert!(!toggle_pause(false, false));
        assert!(!toggle_pause(false, true));
    }
}
