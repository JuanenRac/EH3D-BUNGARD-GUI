# EH3D-BUNGARD-GUI: requirements

This register is the single source of the state of each capability: `STATUS.md` and the README files cite it and never state a state that it does not. A state is one of `planned`, `prototype`, `implemented` and `validated`. `validated` means that a person has run the capability on a real machine and the evidence of the manifest says `bench` or `field`; nothing here is validated, because no job has been driven from this window on a real machine; it has run on the simulator of the engines of the drilling and routing programs only.

The register has 58 requirements: 42 implemented, 9 prototype, 7 planned and 0 validated. By progress: 42 done, 9 partial, 2 pending and 5 blocked. High coverage is not proof of correctness, so each row says how it is checked: the test that names it, and what it shows.

## How to read the register

* **ID:** `BGUI-NNN`, never reused and never renumbered; a new capability takes the next number.
* **State:** `implemented` is code that is complete and tested on a computer, `prototype` is part of the capability, `planned` is not written.
* **Progress:** `done`, `partial`, `pending` (can be written without hardware) or `blocked` (waits for hardware or for another program of the suite).
* **Evidence:** `simulator` when the test runs against the simulator of the machine, which the engines of the drilling and routing programs drive with the same bytes as a real machine, `host-test` when it checks a calculation, a reader or a screen-less piece, `none` when nothing is written yet.
* **Test:** the acceptance criterion: `file::name` is the test `name` of `tests/file.rs`; a name with a slash, such as `src/gui/controls::name`, is the path from the root without the extension (a test inside the source file). The test must exist. For a capability that is not written the cell says `planned:` and the name of the test that will show it; the day that test exists, the row must change.
* **Limits:** what is not done or not proven; a row that is `partial` or `blocked` always says it.

The check is `tests/reqs.rs`: it reads this table, finds each test and fails when a state, a test or a count does not agree.

## Settings, command line and launcher

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| BGUI-001 | What the person chose is remembered and comes back. | implemented | done | host-test | `src/launcher::what_is_remembered_comes_back` | - |
| BGUI-002 | Saving the settings leaves no side file and reports a failure. | implemented | done | host-test | `src/launcher::saving_the_settings_leaves_no_side_file_and_reports_a_failure` | - |
| BGUI-003 | A wrong value in the settings keeps the default. | implemented | done | host-test | `more::a_wrong_value_in_the_settings_keeps_the_default` | - |
| BGUI-004 | The keys of the window are rewritten and every other line of the machine file stays as it was. | implemented | done | host-test | `more::the_keys_of_the_window_are_rewritten_and_every_other_line_stays_as_it_was` | - |
| BGUI-005 | A file name with a line break cannot add a setting to the file. | implemented | done | host-test | `more::a_file_name_with_a_line_break_cannot_add_a_setting_to_the_file` | - |
| BGUI-006 | The options of the command line are read, a wrong one is said, and the help names every option. | implemented | done | host-test | `src/cli::help_and_version_are_read_and_the_help_names_every_option` | - |
| BGUI-007 | The two commands of the server are off unless they are given. | implemented | done | host-test | `src/cli::the_two_commands_of_the_server_are_options_that_are_off_unless_they_are_given` | - |
| BGUI-008 | The three panes always fill the room and keep their least width. | implemented | done | host-test | `more::whatever_is_asked_the_three_panes_fill_the_room_and_keep_their_least_width` | - |

## Files, views and panel

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| BGUI-009 | The holes of a drilling file come with their diameters in millimetres. | implemented | done | host-test | `src/jobs::the_holes_come_with_their_diameters_in_mm` | Only small files written for the tests and the examples; no collection of real files. |
| BGUI-010 | The paths of an HP-GL file are in millimetres. | implemented | done | host-test | `src/jobs::the_paths_of_an_hpgl_file_are_in_mm` | - |
| BGUI-011 | The outline of a Gerber board opens as a routing job and a layer of copper does not. | implemented | done | host-test | `src/gui/controls::the_outline_of_a_gerber_board_opens_as_a_routing_job_and_a_layer_of_copper_does_not` | - |
| BGUI-012 | The circles of a plotter file are cut into chords by the chord angle of the configuration. | implemented | done | host-test | `src/gui/controls::a_circle_of_a_plotter_file_is_cut_by_the_chord_angle_of_the_configuration` | - |
| BGUI-013 | Several drilling files make one job with their tools numbered by diameter. | implemented | done | host-test | `src/gui/opener::several_drilling_files_make_one_job_with_their_tools_numbered_by_diameter` | - |
| BGUI-014 | A job that does not fit on the table is refused before it starts. | implemented | done | host-test | `src/gui/controls::a_job_that_does_not_fit_on_the_table_is_refused_before_it_starts` | - |
| BGUI-015 | After moving the board and making a panel the job drills where the views draw. | implemented | done | host-test | `src/gui/extra::after_moving_the_board_and_making_a_panel_the_job_drills_where_the_views_draw` | - |
| BGUI-016 | A panel that does not fit the table is warned about and not drawn over the edge in silence. | implemented | done | host-test | `src/gui/extra::a_panel_that_does_not_fit_the_table_is_warned_about_and_not_drawn_over_the_edge_in_silence` | - |
| BGUI-017 | The holes and the paths of a board stay on one another through every turn. | implemented | done | host-test | `src/gui/view2d::the_holes_and_the_paths_of_a_board_stay_on_one_another_through_every_turn_and_after_a_move` | - |
| BGUI-018 | The design cannot be changed while the machine works, and every edit can be undone and redone. | implemented | done | host-test | `src/gui/edit::while_the_machine_works_the_design_cannot_be_changed_and_it_can_again_when_it_stops` | - |
| BGUI-019 | The published sizes of the machines are there, with the manual and the automatic change. | implemented | done | host-test | `src/machines::every_machine_is_a_model_with_the_manual_and_with_the_automatic_change` | The sizes come from the published data of the maker, not from a measure. |
| BGUI-020 | The 3D machine starts with the start of the machine, then home, then the first point. | implemented | done | simulator | `src/gui/machine3d::a_job_starts_with_the_start_of_the_machine_then_home_then_the_first_point` | - |

## Running the job

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| BGUI-021 | Drilling and routing run one after the other. | prototype | partial | simulator | `src/gui/controls::the_drilling_and_the_routing_that_are_on_run_one_after_the_other` | Run on the simulator only; no whole job has run on a real machine. |
| BGUI-022 | The emergency button stops the job and blocks the start until it is released. | prototype | partial | simulator | `src/gui/controls::the_emergency_stop_stops_the_job_and_blocks_the_start_until_it_is_released` | The button of the window is not the stop of the machine; it has been tried on the simulator only. |
| BGUI-023 | Start goes out while the job runs and comes back with pause or stop. | implemented | done | simulator | `src/gui/controls::start_goes_out_while_the_job_runs_and_comes_back_with_pause_or_stop` | - |
| BGUI-024 | The pause toggles only while a program runs. | prototype | partial | simulator | `src/jobflow::the_pause_toggles_only_while_a_program_runs` | The pause holds the job between two moves, not in the middle of one. |
| BGUI-025 | Whatever the tools and the rack, every tool is loaded once in the order of the job. | implemented | done | host-test | `src/jobflow::whatever_the_tools_and_the_rack_every_tool_is_loaded_once_in_the_order_of_the_job` | Checked with generated cases, not a continuous fuzzer. |
| BGUI-026 | With the changer the operator is never asked for a tool and the head goes to the bar. | prototype | partial | simulator | `src/gui/rack::with_the_changer_the_operator_is_never_asked_for_a_tool_and_the_head_goes_to_the_bar` | The changer has not been driven on a real machine. |
| BGUI-027 | A job whose tools have no place does not start, says which, and opens the window of the places. | implemented | done | host-test | `src/gui/slotswin::a_job_with_tools_that_have_no_place_does_not_start_and_says_which_and_opens_the_window` | - |
| BGUI-028 | A tool with no place, two in one place and a place the machine does not define are told. | implemented | done | host-test | `src/slots::a_tool_with_no_place_two_in_one_and_a_place_the_machine_does_not_define_are_told` | - |
| BGUI-029 | A job cut at any hole goes on from the saved state and drills nothing twice. | prototype | partial | simulator | `job_cuts::a_job_cut_at_any_hole_goes_on_from_the_saved_state_and_drills_nothing_twice` | Cut and resumed on the simulator with injected faults; not on a real machine. |
| BGUI-030 | A job whose files or settings changed is not resumed. | implemented | done | host-test | `job_cuts::a_job_whose_files_or_settings_changed_is_not_resumed` | - |
| BGUI-031 | A write that was cut leaves the last good state. | implemented | done | host-test | `job_cuts::a_write_that_was_cut_leaves_the_last_good_state` | - |
| BGUI-032 | A damaged state file never panics and what is accepted reads back the same. | implemented | done | host-test | `job_cuts::a_damaged_file_never_panics_and_what_is_accepted_reads_back_the_same` | Seeded generation, not a continuous fuzzer. |

## Jog, probe and spindle

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| BGUI-033 | The head stops at the ends of the travel and the last part is moved. | prototype | partial | simulator | `src/jog::the_head_stops_at_the_ends_of_the_travel_and_the_last_part_is_moved` | The travel is the configured table; no limit switch has been reached on a real machine. |
| BGUI-034 | A job or the emergency stop blocks every jog order. | implemented | done | simulator | `src/jog::a_job_or_the_emergency_stop_blocks_every_order` | - |
| BGUI-035 | Going to a place raises Z first, then goes across, then goes down. | implemented | done | simulator | `src/jog::going_to_a_place_raises_z_first_then_goes_across_then_goes_down` | - |
| BGUI-036 | The probe measures the table point by point and the map says how it is tilted. | prototype | partial | simulator | `src/gui/controls::the_probe_measures_the_table_point_by_point_and_the_map_says_how_it_is_tilted` | Tried with a simulated table; no real table has been probed. |
| BGUI-037 | A complete map corrects the jobs only when the person asks. | implemented | done | simulator | `src/gui/controls::a_map_that_is_complete_corrects_the_jobs_only_when_the_person_asks` | - |
| BGUI-038 | A bit that asks for far more turns than the spindle has is not compatible. | implemented | done | host-test | `src/spindle::a_bit_that_asks_for_far_more_turns_than_the_spindle_has_is_not_compatible` | The limits of the spindles are those of the data sheets. |
| BGUI-039 | Every material keeps inside the limits of the spindle. | implemented | done | host-test | `src/defaults::every_material_keeps_inside_the_limits_of_the_spindle` | - |

## Server, node and updates

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| BGUI-040 | A request that timed out is not sent twice but a read is tried again. | prototype | partial | host-test | `src/server_wire::a_post_that_timed_out_is_not_sent_twice_but_a_get_is_tried_again` | Against a faked server, not a running one. |
| BGUI-041 | A request that never got an answer is an error in words after its retries. | implemented | done | host-test | `src/server_wire::a_request_that_never_got_an_answer_is_an_error_in_words_after_its_retries` | - |
| BGUI-042 | A file without a server, or with a value out of range, is refused with the reason. | implemented | done | host-test | `src/servercmd::a_file_without_a_server_or_with_a_value_out_of_its_range_is_refused_with_the_reason` | - |
| BGUI-043 | A node that does not answer is an error, not a hang. | implemented | done | host-test | `src/inspection::a_node_that_does_not_answer_is_an_error_not_a_hang` | - |
| BGUI-044 | A package whose checksum does not agree is never installed. | implemented | done | host-test | `src/updates::a_package_whose_checksum_does_not_agree_is_never_installed` | - |
| BGUI-045 | A published signature that does not verify stops the update. | implemented | done | host-test | `src/updates::a_published_signature_that_does_not_verify_stops_the_update` | - |
| BGUI-046 | The update window offers only what can be done and never installs alone. | implemented | done | host-test | `src/gui/controls::the_update_window_offers_only_what_can_be_done_and_never_installs_alone` | - |
| BGUI-047 | The secrets never get into the log, and a log that cannot be written does not panic. | implemented | done | host-test | `src/applog::the_secrets_never_get_in` | - |

## Languages and report

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| BGUI-048 | No text of a button, a switch or a list is cut in any language. | implemented | done | host-test | `src/gui/controls::no_text_of_a_button_a_switch_or_a_list_is_cut_in_any_language` | - |
| BGUI-049 | Every language has every word of the report and only those. | implemented | done | host-test | `src/report::every_language_has_every_word_and_only_those` | - |
| BGUI-050 | The time of the report is the sum of the moves, the strokes, the cuts and the tool changes. | prototype | partial | host-test | `src/report::the_time_is_the_sum_of_the_moves_the_strokes_the_cuts_and_the_tool_changes` | An estimate; it has not been compared with a machine. |
| BGUI-051 | Every voice has its four recordings and its licence. | implemented | done | host-test | `voices::every_voice_is_named_with_its_licence_in_the_files_that_say_where_it_comes_from` | - |

## Not written yet

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| BGUI-052 | A whole job is driven from the window on a real Bungard machine. | planned | blocked | none | `planned:real_job::a_whole_job_is_driven_on_a_real_machine` | Needs a machine, a board, a tool and a person ready to stop the job. |
| BGUI-053 | The pause stops in the middle of a move. | planned | blocked | none | `planned:pause_mid_move::a_pause_stops_in_the_middle_of_a_move` | Depends on what the control unit allows; it needs a machine. |
| BGUI-054 | The 3D view shows the height of the tool paths. | planned | pending | none | `planned:path_height::the_3d_view_shows_the_height_of_the_tool_paths` | Not written. |
| BGUI-055 | The whole configuration is loaded and saved from the window. | planned | pending | none | `planned:whole_config::the_whole_machine_file_is_loaded_and_saved` | Not written; only the keys the window offers are saved. |
| BGUI-056 | The CCD Premium and the CCD XL are driven. | planned | blocked | none | `planned:other_models::the_premium_and_the_xl_are_driven` | Needs their protocol and a machine. |
| BGUI-057 | The job goes through a running server and a running node to a machine. | planned | blocked | none | `planned:real_link::a_job_goes_through_a_running_server_and_node` | Needs the server, the node and a machine. |
| BGUI-058 | A release package is published and the window updates itself from it. | planned | blocked | none | `planned:real_release::the_window_updates_itself_from_a_published_release` | Needs a published release. |
