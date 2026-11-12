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
}
