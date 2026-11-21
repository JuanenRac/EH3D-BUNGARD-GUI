# EH3D-BUNGARD-GUI: status

## Current state

EH3D-BUNGARD-GUI works on a computer and on the simulator: the launcher, the views, the opening and editing of jobs, the panel, the tool changer, the resume of a job, the jog, the probe, the report, the server window and the update are covered by tests, and the window has been looked at with pictures of the simulated job. It has never driven a real machine; the pause holds the job between two moves, the 3D view shows no height of the tool paths, and the window needs its three sibling folders next to it.

## Capabilities

The official state of each capability is in `REQUIREMENTS.md`. The register has 58 requirements: 42 implemented, 9 prototype, 7 planned and 0 validated. By progress: 42 done, 9 partial, 2 pending and 5 blocked. This page never states a state that `REQUIREMENTS.md` does not.

## Open questions

* Whether the pause should stop in the middle of a move, which depends on what the control unit allows.
* Whether the whole machine file should be edited from the window or only the keys it offers.

## What it needs from other repositories

* `EH3D-DRILLPRO` and `EH3D-ROUTEPRO`: the protocol, the engines and the machine file, as sibling folders at the versions named in the workflows.
* A Bungard CCD machine and a person at it, for any job that is not simulated.
* `EH3D-SERVER` and `EH3D-NODE-BUNGARD`, for the server and the node connections; those paths have only been exercised against the simulator.

## Known technical debt and omitted tests

* No job has been driven from this window on a real machine; the simulator is the only machine the tests talk to.
* The drawing of the window is tried with pictures of the simulated job; the tests cover its logic, its texts and its layout rules, not how it looks on every screen.
* Some tests are gated by the operating system or by a feature, so the count of tests is that of the CI and not of every computer.
