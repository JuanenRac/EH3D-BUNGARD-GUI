//! The machine of the 3D view as a machine, not as a marker that sits on the work: it has a place of its own (home, the corner of the plate) and a start like the real one.
//! When a job starts the machine powers up, goes home, travels to the first point of the work, and only then does the work begin and the head follow it; when the job is over
//! (or is stopped) the head goes back home. Moving the board in the plan view never moves the machine: where the work is on the table changes where the first point is, not
//! where the machine is. Everything here is time and positions, with no window, so it is tested on its own.

/// How long the machine takes to power up before it moves, in seconds of the simulation.
const STARTUP_S: f64 = 1.2;
/// The speed of homing and of the free moves, in mm per second (the real machines rapid at a few hundred).
const HOMING_MM_S: f64 = 110.0;
const RAPID_MM_S: f64 = 160.0;
/// The least a move takes, so that a short one is still seen.
const LEAST_MOVE_S: f64 = 0.7;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    /// Nothing runs: the head rests at home.
    Parked,
    /// The machine powers up (the head does not move yet).
    Startup,
    /// The head goes to its home position.
    Homing,
    /// The head travels from home to the first point of the work.
    Approach,
    /// The work: the head is where the job says.
    Working,
    /// The job is over: the head goes back home.
    Returning,
}

#[derive(Debug, Clone, Copy)]
pub struct Sim3d {
    phase: Phase,
    /// Seconds spent in the phase.
    t: f64,
    pos: (f64, f64),
    from: (f64, f64),
    to: (f64, f64),
    duration: f64,
    home: (f64, f64),
    /// Where the work begins: the head travels there after homing.
    first_point: (f64, f64),
}

fn distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

/// A move that starts and ends softly.
fn ease(x: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

impl Sim3d {
    /// A machine at rest at its home position.
    pub fn new(home: (f64, f64)) -> Sim3d {
        Sim3d {
            phase: Phase::Parked,
            t: 0.0,
            pos: home,
            from: home,
            to: home,
            duration: 0.0,
            home,
            first_point: home,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// Where the head is now, in millimetres of the table.
    pub fn pos(&self) -> (f64, f64) {
        self.pos
    }

    fn enter(&mut self, phase: Phase, to: (f64, f64), duration: f64) {
        self.phase = phase;
        self.t = 0.0;
        self.from = self.pos;
        self.to = to;
        self.duration = duration;
    }

    fn move_time(a: (f64, f64), b: (f64, f64), mm_s: f64) -> f64 {
        (distance(a, b) / mm_s).max(LEAST_MOVE_S)
    }

    /// A job starts: the machine powers up, goes home and travels to `first`, the first point of the work (the machine stays at home when the job has no point).
    pub fn begin(&mut self, first: Option<(f64, f64)>) {
        self.first_point = first.unwrap_or(self.home);
        // what comes after the start is worked out when each phase begins; the point to travel to is kept
        self.enter(Phase::Startup, self.pos, STARTUP_S);
    }

    /// The job is over or was stopped: the head returns home (at once when it is there already).
    pub fn finish(&mut self) {
        if matches!(self.phase, Phase::Parked | Phase::Returning) {
            return;
        }
        let d = Self::move_time(self.pos, self.home, RAPID_MM_S);
        self.enter(Phase::Returning, self.home, d);
    }

    /// While the job works the head is where the job says.
    pub fn follow(&mut self, p: (f64, f64)) {
        if self.phase == Phase::Working {
            self.pos = p;
        }
    }

    /// Lets `dt` seconds of the simulation pass. Returns true when the machine is ready to work (the job may advance).
    pub fn advance(&mut self, dt: f64) -> bool {
        let mut left = dt.max(0.0);
        // a phase that ends hands what is left of the time to the next one, so that a fast simulation does not stop at each change
        for _ in 0..8 {
            match self.phase {
                Phase::Parked => return false,
                Phase::Working => return true,
                _ => {}
            }
            let remaining = self.duration - self.t;
            let step = left.min(remaining.max(0.0));
            self.t += step;
            left -= step;
            let k = if self.duration > 0.0 {
                ease(self.t / self.duration)
            } else {
                1.0
            };
            self.pos = (
                self.from.0 + (self.to.0 - self.from.0) * k,
                self.from.1 + (self.to.1 - self.from.1) * k,
            );
            if self.t + 1e-9 < self.duration {
                return false;
            }
            self.pos = self.to;
            match self.phase {
                Phase::Startup => {
                    // the head goes home first
                    let d = Self::move_time(self.pos, self.home, HOMING_MM_S);
                    self.enter(Phase::Homing, self.home, d);
                }
                Phase::Homing => {
                    let d = Self::move_time(self.home, self.first_point, RAPID_MM_S);
                    let p = self.first_point;
                    self.enter(Phase::Approach, p, d);
                }
                Phase::Approach => {
                    self.phase = Phase::Working;
                    self.t = 0.0;
                }
                Phase::Returning => {
                    self.phase = Phase::Parked;
                    self.t = 0.0;
                }
                _ => {}
            }
        }
        self.phase == Phase::Working
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(m: &mut Sim3d, seconds: f64) {
        let mut t = 0.0;
        while t < seconds {
            m.advance(0.05);
            t += 0.05;
        }
    }

    #[test]
    fn a_machine_at_rest_is_at_home_and_does_not_move() {
        let mut m = Sim3d::new((0.0, 0.0));
        assert_eq!(m.phase(), Phase::Parked);
        assert!(!m.advance(10.0));
        assert_eq!(m.pos(), (0.0, 0.0));
    }

    #[test]
    fn a_job_starts_with_the_start_of_the_machine_then_home_then_the_first_point() {
        let mut m = Sim3d::new((0.0, 0.0));
        m.begin(Some((200.0, 150.0)));
        assert_eq!(m.phase(), Phase::Startup);
        // powering up: the head does not move
        m.advance(0.5);
        assert_eq!((m.phase(), m.pos()), (Phase::Startup, (0.0, 0.0)));
        // it passes through homing and the approach in order, and the head really travels
        let mut seen = vec![m.phase()];
        let mut max_x = 0.0f64;
        for _ in 0..400 {
            m.advance(0.05);
            if *seen.last().unwrap() != m.phase() {
                seen.push(m.phase());
            }
            max_x = max_x.max(m.pos().0);
            if m.phase() == Phase::Working {
                break;
            }
        }
        assert_eq!(
            seen,
            vec![
                Phase::Startup,
                Phase::Homing,
                Phase::Approach,
                Phase::Working
            ]
        );
        assert_eq!(m.pos(), (200.0, 150.0));
        assert!(max_x <= 200.0 + 1e-9);
    }

    #[test]
    fn the_job_may_advance_only_when_the_machine_is_at_the_first_point() {
        let mut m = Sim3d::new((0.0, 0.0));
        m.begin(Some((100.0, 100.0)));
        let mut ready_at = None;
        for i in 0..2000 {
            if m.advance(0.01) {
                ready_at = Some(i as f64 * 0.01);
                break;
            }
        }
        let at = ready_at.expect("it gets ready");
        // the start, the homing (a move that is seen) and the travel take real time at normal speed
        assert!(at > 2.5, "{at}");
        assert_eq!(m.pos(), (100.0, 100.0));
    }

    #[test]
    fn a_fast_simulation_does_not_stop_at_each_phase() {
        let mut m = Sim3d::new((0.0, 0.0));
        m.begin(Some((250.0, 120.0)));
        assert!(m.advance(1000.0), "all the phases in one step");
        assert_eq!((m.phase(), m.pos()), (Phase::Working, (250.0, 120.0)));
    }

    #[test]
    fn while_the_work_goes_on_the_head_follows_it() {
        let mut m = Sim3d::new((0.0, 0.0));
        m.begin(Some((10.0, 10.0)));
        m.advance(1000.0);
        m.follow((55.5, 42.0));
        assert_eq!(m.pos(), (55.5, 42.0));
        m.follow((60.0, 43.0));
        assert_eq!(m.pos(), (60.0, 43.0));
    }

    #[test]
    fn the_head_does_not_follow_anything_while_the_machine_is_not_working() {
        let mut m = Sim3d::new((0.0, 0.0));
        m.follow((90.0, 90.0));
        assert_eq!(m.pos(), (0.0, 0.0));
        m.begin(Some((10.0, 10.0)));
        m.follow((90.0, 90.0));
        assert_eq!(m.pos(), (0.0, 0.0), "powering up is not working");
    }

    #[test]
    fn at_the_end_the_head_goes_back_home_and_rests() {
        let mut m = Sim3d::new((0.0, 0.0));
        m.begin(Some((10.0, 10.0)));
        m.advance(1000.0);
        m.follow((180.0, 90.0));
        m.finish();
        assert_eq!(m.phase(), Phase::Returning);
        run(&mut m, 0.3);
        let (x, y) = m.pos();
        assert!(
            x > 0.0 && x < 180.0 && y > 0.0 && y < 90.0,
            "on its way home: {x} {y}"
        );
        run(&mut m, 5.0);
        assert_eq!((m.phase(), m.pos()), (Phase::Parked, (0.0, 0.0)));
        // finishing twice, or at rest, changes nothing
        m.finish();
        assert_eq!(m.phase(), Phase::Parked);
    }

    #[test]
    fn a_job_stopped_while_the_machine_is_still_preparing_goes_home_from_where_it_is() {
        let mut m = Sim3d::new((0.0, 0.0));
        m.begin(Some((300.0, 200.0)));
        run(&mut m, 3.5);
        m.finish();
        run(&mut m, 10.0);
        assert_eq!((m.phase(), m.pos()), (Phase::Parked, (0.0, 0.0)));
    }

    #[test]
    fn a_job_without_a_point_stays_at_home_and_works() {
        let mut m = Sim3d::new((0.0, 0.0));
        m.begin(None);
        assert!(m.advance(1000.0));
        assert_eq!(m.pos(), (0.0, 0.0));
    }

    #[test]
    fn the_moves_start_and_end_softly() {
        assert_eq!(ease(0.0), 0.0);
        assert_eq!(ease(1.0), 1.0);
        assert!(ease(0.1) < 0.1 && ease(0.9) > 0.9);
        assert_eq!(ease(-3.0), 0.0);
        assert_eq!(ease(3.0), 1.0);
    }
}
