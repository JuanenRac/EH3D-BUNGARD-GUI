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
}
