# Coder stacking limits and Unified Theory cosmic actions — release verification

Native **0.10.37**, champion mod **0.2.37**, native dependency **>=0.10.37**, all release dates **2026-10-11**. Base: main after PR #27 merged. Branch: `codex/science-cosmic-actions-coder-limits`. Review PR: https://github.com/PowerViber/tfm2_mods/pull/28.

The user authorized the preceding complete plan and improved design. The attached `coder-verification.md` is the required verification reference; its old branches and version examples are historical. No typing tables, language recipes, base execution clocks or cooldown tables were changed.

## Resulting behavior

**Coder:** the previous 50% output cut remains. A fresh compilation gets one first execution; subsequent executions or loading from storage add a 90% damage reduction. Loaded, reused and inherited ancestry use one penalty. Kubernetes allocates 80% to the primary and 20% to its replica from the same immutable offensive baseline. Meta children divide a 20% allowance; nested shares multiply rather than reset. Every owned direct, delayed, percentage, wall, tether, drone and damaging sign-flipped-heal route shares one raw-damage allowance per Coder and target: 30% of current maximum HP over a rolling 60 ticks. Excess is discarded at impact; zero allowance does not call engine damage or HP mutation. Training/sudo/transaction/deployment combine to at most +40% offensive power. Support retains its existing calculations. Fleets replace their oldest member at four drones ordinarily or six for Top 10 and release associated RAM/process ownership. Damage-aware mastery judgment replaces weak reused offensive daemons, preserving the existing variety requirement without slowing coding. Logs add `reused`, `cast_share_pct`, `requested_raw` and `discarded_raw`.

Coder's packet darts, travelling connector/chip indicators and one fleet animation use eight-frame sprites and the existing effect token bucket. These are bounded visual indicators; Coder's code execution scheduling remains unchanged.

**Unified Theory:** the match HUD becomes one 12x12 scientist/preparation orb and one 42x5 available/reserved charge bar, separated by three pixels: total 57x12. Charge is shown in 10-CU segments; hatched segments are reserved. The old notebook strips and allocation textures are retired; detailed equations remain in Science Lab. The human sprite, accepted forward gaze and planted-foot transformations stay at the existing compact champion scale. Mastery evolves the instruments and hand-adjacent experiments. #1 has a restrained universe window, impossible proof or living seed, with a larger eight-frame 0.8-second combination completion animation. There is one ambient loop and one completion channel per scientist.

All 75 skills have separate activation and contact art at four mastery stages (300 each). There are 23 directional delivery types and genuine persistent fields. Instruments wait 12 ticks before release/contact, then visibly travel before field deployment; Acid/Oxidation apply their debuffs on contact. Required setup stages preserve reservations while their owned instruments travel. Moving artwork plays successive two-frame pairs every four ticks, preserving all eight original sprites and exact nonoverlapping deadlines. Eight combined packets/deliveries, bounded/coalesced impact art, and the existing six effect-start allowance per caster/update prevent unbounded animation accumulation.

**Crash hardening:** Salt Crystallization previously called `spawn_unit` with an unsupported custom name. `docs/playtest-notes.md` round 8 records an actual earlier host crash from that same nonexistent-base-animation pattern. It now spawns the known base `bombardier`, binds/blocks its actions and covers it with local opaque crystal artwork. The artwork covers 1,424 regular-active frame/direction cases from the bundled base and Valorant-shadow references, including anchor margins. Replacement, expiry, failed spawn, owner death/disappearance and stale target IDs are tested. Expired per-target windows are pruned; splitting retains original payloads; pending deliveries retain experiment budgets. Pre-action commit diagnostics improve failure attribution.

This fixes a concrete unsafe lookup; the user's crash cause remains unconfirmed because their failing match/version/log was not supplied. Rust/host fixtures cannot validate real renderer behavior. The crystal coverage proof is static and does not establish that the host displays the spawned unit's buff correctly.

## Animated design previews

- [#1 combination signatures](unified-theory-peak-completions.gif)
- [Compact mastery appearances](unified-theory-ranked-cosmic.gif)
- [Compact orb and bar](unified-theory-hud.gif)
- [Top 10 comparison](unified-theory-top10.png)
- All 25 skill cards per form: [Einstein](unified-theory-skills-einstein.png), [Newton](unified-theory-skills-newton.png), [Curie](unified-theory-skills-curie.png).

## Runtime and asset evidence

Full native suite: **202 passed, 0 failed, 1 pre-existing ignored** (`scribble::real_data::real_pending`). No assertions or tests were removed, weakened or newly ignored. Includes 26 focused Coder damage tests and 53 UT integration tests, all 75 handlers, all 50 experiment recipes, real notebook combinations at long range, all six form changes, delayed ownership, allowance boundaries, replica conservation, RAM cleanup, crystal failure paths and sprite fairness under the unchanged start limit.

| Host fixture | Duration | Temporary starts | Peak live temporary effects |
|---|---:|---:|---:|
| One crowded UT | 7 s | 968 | 28 |
| Five UT in a 5v5 | 20 s | 4,566 | 125 |
| Four UT + Coder in a 5v5 | 20 s | 4,021 | 106 |

The 5v5 fixtures include deaths, direct form changes, invalid IDs and failed crystal spawning. Shorter travel intervals increased starts from 3,249/2,980 to 4,566/4,021 in the two 5v5 fixtures while keeping their peak live counts unchanged. These are host-API and lifetime measurements, not a real game's GPU/FPS benchmark.

UT assets: 4,730 views, 38 VFX sheets, every atlas <=2048, stored eight-frame distinctness, fixed origins, local crystal coverage and byte-identical regeneration verified. Decoded RGBA estimate: **125.48 MiB**, versus **184.38 MiB** before this revision (about 32% lower). This estimate assumes every referenced sheet is decoded and does not measure the engine's actual allocation. Coder: 5,500 views, 17 bounded sheets, 558 exact typing vectors. UT: 375 exact charge vectors. Editor checks preserve one Science Lab, the common Coder/Isliid shell and all mastery lists.

## Attached-checklist commands and output

Every required command below exited 0. Native Linux/Windows and manager build/test logs contain **0 warnings**. Commands use `source /workspace/.tfm2-env/env.sh`; the prepared MinGW linker is selected there. No manager source changed, so its EXE was not rebuilt.

### 1. Linux release build

```sh
cargo build --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml
```

```text
    Finished `release` profile [optimized] target(s) in 0.00s
```

### 2. Full native release tests

```sh
cargo test --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml -- --nocapture
```

```text
    Blocking waiting for file lock on build directory
   Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 1m 03s
     Running unittests src/lib.rs (native/tfm2_custom_ai/target/release/deps/tfm2_custom_ai-e5d070af18bd592e)

running 203 tests
test coder::damage_tests::a_waiting_saved_script_keeps_its_penalty_until_the_trigger_fires ... ok
test coder::damage_tests::bonuses_are_consumed_once_offense_caps_at_forty_percent_and_support_retains_full_power ... ok
test coder::damage_tests::a_kubernetes_meta_replica_cannot_reset_the_child_damage_allowance ... ok
test coder::damage_tests::diagnostic_presim_suppression_preserves_damage_and_does_not_consume_extra_boosts_or_rng ... ok
test coder::damage_tests::daemon_and_cooldown_schedules_are_unchanged ... ok
test coder::damage_tests::diagnostics_attribute_chain_packets_and_live_scaled_persistent_hits ... ok
test coder::damage_tests::compile_history_is_bounded_without_forgetting_a_live_daemon ... ok
test coder::damage_tests::diagnostics_preserve_support_and_report_nonlethal_direct_hp_and_engine_kill_credit ... ok
test coder::damage_tests::direct_hp_bug_shares_the_rolling_allowance_and_never_touches_a_dead_target ... ok
test coder::damage_tests::failed_runs_keep_bonus_and_first_execution_for_the_next_successful_cast ... ok
test coder::damage_tests::drones_replace_oldest_and_release_owned_processes_at_both_rank_caps ... ok
test coder::damage_tests::mastered_choice_values_reused_offensive_code_by_remaining_output_and_keeps_support_value ... ok
test coder::damage_tests::immediate_damage_and_burst_are_reduced_across_rigs ... ok
test coder::damage_tests::packets_keep_their_count_schedule_and_are_not_nerfed_twice ... ok
test coder::damage_tests::percentage_attacks_and_execute_boundaries_cannot_bypass_replay_nerfs ... ok
test coder::damage_tests::primary_and_kubernetes_replica_share_one_damage_baseline_without_reducing_support ... ok
test coder::damage_tests::replay_damage_retains_both_reductions_without_halving_support_and_quantum_stays_a_coin_flip ... ok
test coder::damage_tests::replays_retain_damage_scaling_in_drones_tethers_and_walls ... ok
test coder::damage_tests::replica_uses_the_primary_offensive_context_after_sudo_is_spent_and_training_changes ... ok
test coder::damage_tests::rolling_allowance_is_per_target_per_coder_and_expires_at_the_exact_sixtieth_tick ... ok
test coder::damage_tests::reused_original_daemons_and_loaded_reuse_take_the_penalty_once_and_fresh_writing_resets_it ... ok
test coder::damage_tests::saved_kubernetes_only_penalizes_its_replica_and_support_stays_full_strength ... ok
test coder::damage_tests::saving_a_fresh_compile_and_loading_a_saved_compile_have_distinct_damage ... ok
test coder::damage_tests::support_and_sign_flipped_healing_keep_their_original_strength ... ok
test coder::damage_tests::rolling_allowance_covers_immediate_percentage_packets_walls_drones_and_tethers_and_discards_excess ... ok
test coder::tests::a_saved_function_reloads_with_its_bugs_faster_off_an_ssd ... ok
test coder::tests::choosing_is_cheap ... ok
test coder::damage_tests::loaded_code_reduces_every_emitted_hit_and_preserves_support_and_delivery_times ... ok
test coder::tests::gpu_and_cloud_functions ... ok
test coder::tests::every_view_name_exists ... ok
test coder::tests::he_doesnt_spam_the_same_function ... ok
test coder::tests::heat_throttles_the_clock_and_parts_change_the_rig ... ok
test coder::tests::haskell_ships_clean_and_lua_is_quick ... ok
test coder::tests::mining_slows_his_typing ... ok
test coder::tests::one_status_line_at_a_time ... ok
test coder::tests::root_interpolates_to_number_one ... ok
test coder::tests::lab_vectors ... ok
test coder::tests::scripts_keep_him_writing ... ok
test coder::tests::slots_grow_with_rank_and_autoscale ... ok
test coder::tests::the_code_table_matches_the_functions ... ok
test coder::tests::believed_cost_is_rosier_at_low_rank ... ok
test coder::tests::the_data_center_has_its_risks ... ok
Junior firewall: py 10.7 s, C++ 18.0 s 0.81 bugs, Rust 36.1 s 0.12 bugs
test coder::tests::rust_catches_bugs_at_a_price ... ok
test coder::tests::the_rig_follows_the_rank ... ok
test coder::tests::the_shop_buys_what_the_problems_call_for ... ok
test coder::tests::the_window_shows_three_lines ... ok
test coder::tests::wall_distance ... ok
test coder::tests::writing_is_deterministic_per_seed ... ok
test coder::trace::tests::append_and_rotation_keep_two_complete_readable_logs ... ok
test coder::trace::tests::batching_keeps_every_hit_and_waits_one_second_or_force ... ok
test coder::trace::tests::event_and_byte_limits_flush_batches_and_preserve_sequence ... ok
test coder::trace::tests::event_lines_report_observed_hp_loss_and_do_not_guess_kill_credit_source ... ok
test coder::trace::tests::maps_are_bounded_and_evicted_pending_events_are_flushed ... ok
test coder::trace::tests::presim_is_skipped_but_each_watched_origin_is_recorded ... ok
test coder::trace::tests::unknown_host_logs_only_the_first_instance_for_each_seed_and_caster ... ok
test flash::tests::ring_stays_on_map ... ok
test flash::tests::toward_stops_short ... ok
test gundam::tests::banish_ends_on_landing ... ok
test gundam::tests::burn_refreshes_not_stacks ... ok
test gundam::tests::every_visual_name_exists_in_the_data ... ok
test gundam::tests::exactly_one_wing_visual_per_phase ... ok
test gundam::tests::inner_knocks_outer_slows ... ok
test gundam::tests::landing_blocks_for_ally ... ok
test gundam::tests::no_attacks_while_rising_or_flying ... ok
test gundam::tests::slice_hits_the_strip_only ... ok
test gundam::tests::stance_swings_burn_and_pull ... ok
test gundam::tests::ult_not_for_farming ... ok
test gundam::tests::ult_saves_a_low_ally_under_attack ... ok
test gundam::tests::ult_supports_a_teamfight ... ok
test isliid::tests::abandoned_plan_does_not_block_new_drawing ... ok
test isliid::tests::accuracy_and_cover_widen_with_rank ... ok
test isliid::tests::activated_strokes_lock_recall_until_planted ... ok
test isliid::tests::aura_strength_divides_across_overlapping_swords ... ok
test isliid::tests::bearer_may_still_draw_lines ... ok
test isliid::tests::big_formation_burst_budget ... ok
effect spawns a second in a busy fight: round 92 538, round 98 217
test isliid::tests::busy_fight_effect_budget ... ok
test isliid::tests::complete_catalogue_has_drawable_legs ... ok
test isliid::tests::draw_press_does_not_hijack_a_ready_plan_sword ... ok
test isliid::tests::escort_roles_fit_the_ally ... ok
test isliid::tests::escorts_go_anywhere_two_per_teammate ... ok
test isliid::tests::every_mastery_rank_considers_simple_engravings ... ok
test isliid::tests::every_sword_state_has_a_moving_aura_source ... ok
test isliid::tests::every_sword_state_has_exactly_one_visual ... ok
test isliid::tests::every_visual_name_exists_in_the_data ... ok
test isliid::tests::far_engravings_deal_less ... ok
test isliid::tests::far_strikes_cool_down_longer ... ok
test isliid::tests::far_target_plan_keeps_its_swords ... ok
test isliid::tests::fired_formations_fade_fast ... ok
test isliid::tests::fired_strokes_still_count_against_power ... ok
test isliid::tests::flight_ticks_match_stepping ... ok
test isliid::tests::fly_angle_covers_the_full_turn ... ok
test isliid::tests::flying_swords_cost_less ... ok
rank 3: peak 12 effects on one tick, 154 over 2.5 s
rank 6: peak 12 effects on one tick, 154 over 2.5 s
rank 7: peak 12 effects on one tick, 147 over 2.5 s
test isliid::tests::formation_fire_has_no_spike ... ok
test isliid::tests::fresh_strokes_show_at_once_and_cool_later ... ok
test isliid::tests::grades_use_unrounded_accuracy ... ok
test isliid::tests::idle_swords_come_home_at_every_rank ... ok
test isliid::tests::imperial_fall_follows_the_dominant_sword ... ok
test isliid::tests::lead_is_capped ... ok
test isliid::tests::live_marks_complete_every_catalogue_shape ... ok
test isliid::tests::locked_swords_are_not_handed_out ... ok
test isliid::tests::no_engraving_on_an_empty_camp ... ok
test isliid::tests::objectives_need_a_teammate ... ok
test isliid::tests::only_imperial_one_is_perfect ... ok
test isliid::tests::plan_legs_outlive_the_deadline ... ok
test isliid::tests::plan_wobble_is_shared_with_the_lab ... ok
test isliid::tests::planted_selection_uses_context_not_fixed_rotation ... ok
test isliid::tests::recall_never_takes_a_plan_sword ... ok
test isliid::tests::redirect_clamps_by_rank ... ok
test isliid::tests::redirect_moves_the_whole_shape_once ... ok
test isliid::tests::seven_swords_remain_distinct ... ok
test isliid::tests::short_flights_are_visible ... ok
rank 0: Severing Line (2 swords)
rank 1: Funnel (3 swords)
rank 2: Funnel (3 swords)
rank 3: Imperial Fortress (4 swords)
rank 4: Imperial Fortress (4 swords)
rank 5: Imperial Fortress (4 swords)
rank 6: Emperor's Blessing (5 swords)
rank 7: Emperor's Domain (7 swords)
test isliid::tests::single_target_draws_a_shape ... ok
test isliid::tests::swords_come_out_of_the_black_hole ... ok
test isliid::tests::swords_start_slow_and_speed_up ... ok
test isliid::tests::trails_follow_cardinal_and_diagonal_paths ... ok
test isliid::tests::waiting_volley_swords_stay_on_the_ring ... ok
test levi::tests::angle_quality_peaks_at_90 ... ok
test levi::tests::apex_plans_a_clear_line_down_the_corridor ... ok
test levi::tests::apex_visual_names_exist ... ok
test levi::tests::chain_gain_needs_a_new_cable ... ok
test levi::tests::flights_follow_the_walking_path_and_read_walls ... ok
test levi::tests::orders_and_presses_are_read_by_tick ... ok
test levi::tests::rank_speeds ... ok
test levi::tests::the_pair_is_one_wall_each_side_and_he_flies_between ... ok
test mod_power_tests::per_champion ... ok
test perf::tests::effect_counts_are_off_without_the_flag ... ok
test perf::tests::wrappers_forward_every_hook ... ok
test press::tests::flags_are_read_by_exact_tick ... ok
test scribble::real_data::real_pending ... ignored
test scribble::tests::locked_tier_goes_to_the_next_spell ... ok
test scribble::tests::memory_round_trip_and_merge ... ok
test scribble::tests::opener_follows_the_situation ... ok
test scribble::tests::overreach_slips ... ok
test scribble::tests::page_flip_keeps_mid_and_bases ... ok
test scribble::tests::ranks ... ok
test scribble::tests::recipes_are_unique_and_tiered ... ok
test scribble::tests::rookies_value_big_spells_by_their_odds ... ok
test scribble::tests::top_ten_needs_300_points_and_is_ordered ... ok
test steve::tests::move_rules_skip_only_when_nothing_applies ... ok
(460000, 330000) (460000, 630000) sealed 0
detour (460000, 314000)
slid: (460000, 210000) (460000, 510000) sealed 1
test steve::tests::straight_wall_and_detour ... ok
test tactics::tests::parses_lines ... ok
test unified_theory::integration::a_packet_kill_prevents_field_control_on_the_dead_snapshot_target ... ok
test unified_theory::integration::a_removed_caster_cleans_owned_units_even_without_a_death_callback ... ok
test unified_theory::integration::acid_and_oxidation_debuffs_land_with_the_projectile ... ok
test unified_theory::integration::all_six_form_changes_are_direct_and_finish_without_stacking ... ok
test unified_theory::integration::budget_and_rolling_guard_cover_multiple_enemies ... ok
test unified_theory::integration::chemical_cleanup_is_limited_to_scientific_tags ... ok
test unified_theory::integration::cloned_simulations_preserve_the_notebook_and_rng ... ok
test unified_theory::integration::coincident_split_packets_share_one_complete_eight_frame_cycle ... ok
test unified_theory::integration::compact_hud_emits_only_an_orb_and_reserved_resource_bar ... ok
test unified_theory::integration::completion_art_follows_the_subject_of_the_final_committed_stage ... ok
test unified_theory::integration::completion_art_requires_a_successful_combination_and_respects_budget ... ok
test unified_theory::integration::contact_actions_keep_their_budget_through_windup_and_can_be_evaded ... ok
test unified_theory::integration::cosmic_auras_replace_once_follow_the_morph_and_never_stack ... ok
test unified_theory::integration::crowded_world_has_no_overlapping_replays_and_expired_art_stops ... ok
test unified_theory::integration::crystals_use_a_valid_base_sprite_and_clean_up_live_entities_only ... ok
test unified_theory::integration::eight_distinct_packets_finish_their_sprite_cycles_under_the_same_start_allowance ... ok
Junior chain: asm 1.72 bugs, py 0.41, js 0.81 (12.1 s vs C++ 17.4 s)
test coder::tests::assembly_hits_hardest_and_breaks_most ... ok
test unified_theory::integration::every_skill_has_an_executable_native_handler ... ok
Script Kiddie chain in C++:  52.0 s, 2.17 bugs shipped, 4.54 syntax errors
test unified_theory::integration::every_template_commits_all_stages_when_its_prerequisites_are_live ... ok
test unified_theory::integration::every_thrown_field_is_deployed_only_after_instrument_arrival ... ok
rollback (Rust), no review: Claude 14.2 s 0.10 bugs 0.64 syntax; ChatGPT 16.6 s 0.04 0.70; Gemini 20.4 s 0.11 2.20
Intern        chain in C++:  27.2 s, 1.39 bugs shipped, 2.66 syntax errors
5 UT 5v5 host stress: 4566 temporary starts/20 s, peak 125 live, 264 post-initial damage contacts; death, direct form changes, invalid IDs and spawn failure exercised
test unified_theory::integration::five_scientists_bound_live_effects_through_impacts_deaths_and_form_changes ... ok
Junior        chain in C++:  17.4 s, 0.83 bugs shipped, 1.50 syntax errors
Developer     chain in C++:  12.2 s, 0.46 bugs shipped, 0.75 syntax errors
Senior        chain in C++:   8.4 s, 0.21 bugs shipped, 0.33 syntax errors
Staff         chain in C++:   6.5 s, 0.07 bugs shipped, 0.12 syntax errors
Architect     chain in C++:   5.1 s, 0.02 bugs shipped, 0.03 syntax errors
Root #1       chain in C++:   3.4 s, 0.00 bugs
test unified_theory::integration::fixtures_are_reproducible_for_all_ranks ... ok
Cosmic crowded scene: 968 starts over 7 s, peak 28 live temporary animations, max 6 starts/update; persistent loops added once
test unified_theory::integration::full_cosmic_scene_bounds_live_animations_not_only_spawns ... ok
test unified_theory::integration::grand_experiment_can_charge_after_real_prerequisites ... ok
test unified_theory::integration::masteries_keep_common_token_speed ... ok
test unified_theory::integration::mastery_art_does_not_change_combat_resources_or_rng ... ok
test coder::tests::the_copilots_have_their_profiles ... ok
test unified_theory::integration::notebook_critical_failure_spends_current_and_refunds_dependents ... ok
test unified_theory::integration::orbit_art_tracks_motion_without_changing_the_packet_vector ... ok
test unified_theory::integration::prerequisite_preparation_is_finite_for_every_template ... ok
test unified_theory::integration::quench_salvages_each_new_source_once ... ok
test unified_theory::integration::rapid_casts_have_one_active_animation_per_channel ... ok
test unified_theory::integration::rapid_selection_keeps_only_latest_destination_and_death_clears_morph ... ok
test unified_theory::integration::splitting_preserves_every_original_payload_when_slots_are_reserved_for_deliveries ... ok
test unified_theory::integration::status_and_buff_names_do_not_depend_on_simulation_addresses ... ok
test unified_theory::integration::stuns_and_displacements_share_the_control_window ... ok
test unified_theory::integration::target_loss_refunds_every_uncommitted_stage ... ok
test unified_theory::integration::trace_rotation_preserves_two_complete_logs ... ok
test unified_theory::integration::travelling_field_recipes_commit_through_the_real_notebook_at_long_range ... ok
test unified_theory::integration::vanished_targets_and_expired_windows_do_not_retain_or_receive_actions ... ok
test unified_theory::integration::visual_layers_are_replaced_restored_and_cleared_without_stacking ... ok
test unified_theory::integration::waiting_for_a_funded_crystal_preserves_and_then_refunds_its_dependent_charge ... ok
test unified_theory::tests::cancel_refunds_only_uncommitted ... ok
test unified_theory::tests::catalogue_integrity ... ok
test unified_theory::tests::children_conserve_payload ... ok
test unified_theory::tests::collision_segment ... ok
test unified_theory_math::tests::agreed_vectors ... ok
test unified_theory_math::tests::integer_regeneration ... ok
test unified_theory_math::tests::invalid_undercharge ... ok
test unified_theory_math::tests::reservation_conservation ... ok
test unified_theory_math::tests::rolling_cap ... ok
test unified_theory_math::tests::vector_normalization ... ok
test unified_theory_math::vectors::science_vectors ... ok
test valorant::tests::blind_scales_with_distance ... ok
test valorant::tests::disk_segment ... ok
test valorant::tests::gun_prices ... ok
test valorant::tests::half_pit_edge_through_the_centre ... ok
test valorant::tests::paranoia_trade_through_a_teammate ... ok
test version_tests::every_version_agrees ... ok
4 UT + Coder 5v5 host stress: 4021 temporary starts/20 s, peak 106 live, 417 post-initial damage contacts; death, direct form changes, invalid IDs and spawn failure exercised
test unified_theory::integration::mixed_scientists_and_coder_bound_live_effects_in_a_five_vs_five_host ... ok
rollback in Assembly: Script Kiddie 109 s 12.48 bugs, Architect 12 s 0.11 bugs
test coder::tests::the_gap_grows_with_every_rank ... ok
test unified_theory::integration::every_mastery_visual_and_direction_resolves ... ok

test result: ok. 202 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.11s
```

### 3. Coder verification

```sh
python tools/verify_coder.py --local
```

```text
Code lab: tables, languages, functions, models and hardware match coder.rs; 558 runs reproduced exactly
Verified 5500 Coder views over 17 sheets (each <= 2048), the rig layers, the top-rank effects, crests and every name coder.rs builds
```

### 4. Manager tests

```sh
cargo test --offline --release -j4 --manifest-path tools/manager/Cargo.toml
```

```text
    Finished `release` profile [optimized] target(s) in 0.00s
     Running unittests src/main.rs (tools/manager/target/release/deps/tfm2_manager-756fc6bf9fcfdf44)

running 13 tests
test core::tests::champion_data_errors_name_the_bad_tag ... ok
test core::tests::log_scan_finds_load_errors ... ok
test core::tests::mods_json_enable ... ok
test core::tests::native_version_meets_the_champions_requirement ... ok
test core::tests::perf_log_is_read ... ok
test core::tests::install_copies_and_keeps_plans ... ok
test core::tests::build_outputs_reset_before_pull ... ok
test core::tests::repo_root_found_from_exe_dir ... ok
test core::tests::stamps_are_dates ... ok
test core::tests::vdf_and_manifest_parse ... ok
test core::tests::check_reports_mismatch ... ok
test core::tests::stale_duplicates_go_to_backup ... ok
test core::tests::the_repos_champions_all_load ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 5. JavaScript checks

```sh
node --check editor/coderlab.js
node -e 'const l=require("./editor/coderlab.js"); console.log(l.selfTest()); for(const [r,p]of l.LADDER)l.simulateSkirmish(r,p,1);'
node --check editor/science-art.js
```

```text
true
```

### 6. Windows release build and shipped DLL

```sh
cargo build --offline --release --target x86_64-pc-windows-gnu -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml
cp native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll mods/tfm2_custom_ai/tfm2_custom_ai.dll
```

```text
   Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 26.94s
```

### 7. Artifact hashes

```sh
sha256sum native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll mods/tfm2_custom_ai/tfm2_custom_ai.dll "TFM2 Mod Manager.exe"
```

```text
1b8853baf0bb6bc3eabfd7f372853ca151f07d858c816a3a7e1f0c1c037216f5  native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll
1b8853baf0bb6bc3eabfd7f372853ca151f07d858c816a3a7e1f0c1c037216f5  mods/tfm2_custom_ai/tfm2_custom_ai.dll
01780422f34e0b040eb3e299ee0f6a9f6f1227462728f402aa0f093383852e91  TFM2 Mod Manager.exe
```

### 8. UT canonical/runtime artwork verification

```sh
python tools/verify_unified_theory.py
```

```text
375 exact native charge vectors; all 8 ranks deterministic
75 unique skills; 50 bounded recipes; generated files current
Compact HUD: 108 eight-frame 12px orbs and 66 independent 42x5 charge bars
Cosmic Unified Experiment: compact handheld ranks; 300 casts and 300 contacts, 44 fields, 736 directional packet loops with shared pair aliases; inert crystal replacement
Original body, three personas, compact HUD and mastery art generated
tfm2_custom_unified_theory 4542 effects 187 persona / mastery overlays
Verified 4730 views over 38 VFX sheets; every atlas <=2048; compact HUD lifetimes; regeneration byte-identical; decoded RGBA estimate 125.48 MiB.
Art: 300 casts, 300 impacts, 44 fields and 736 directional packet loops have eight distinct frames; pair aliases share their atlas pixels; lifetimes match native replay deadlines; cosmic podium previews verified.
HUD: 108 eight-frame 12x12 scientist orbs and 66 independent 42x5 available/reserved charge bars; total footprint 57x12 px; obsolete wide notebook and allocation views removed.
Compact art: all eight frames of 24 forward-gazing costumes fit 18–24 x 35–40 px; every rank emblem fits 18 x 18 px; 48 direct transformations have eight distinct frames over 24 ticks with planted feet, covered base pixels and exact endpoints; static champion cards regenerate exactly.
Cosmic ranks: eight unique costumes per scientist, eight distinct frames per costume and preparation modifier; accepted heads/gaze unchanged; all 75 skill-stage cards and the ranked animation regenerate exactly.
Cosmic auras: 33 distinct eight-frame background loops fit 96 x 112 px, including four podium constructions per scientist; stored frames retain their phases; front layers stay empty; aura frame strips/GIF regenerate exactly.
```

### 9. Coder browser

```sh
DB_EDITOR_URL=http://127.0.0.1:19366/ node tools/verify_coder_browser.cjs
```

```text
PASS: browser parity, 100 rows, language filtering, picker/rewrite, 4 rank arenas, 10-run comparison, no page errors/NaN.
[{"rank":"Script Kiddie","shipped":0.5,"distinct":0.5,"dps":0.1},{"rank":"Intern","shipped":1.5,"distinct":1.5,"dps":0.6166666666666667},{"rank":"Junior","shipped":3.6,"distinct":3.6,"dps":5.15},{"rank":"Developer","shipped":6.3,"distinct":6.3,"dps":8.389999999999999},{"rank":"Senior","shipped":8.4,"distinct":8.4,"dps":9.283333333333333},{"rank":"Staff","shipped":9.2,"distinct":9.2,"dps":15.110000000000001},{"rank":"Architect","shipped":11.8,"distinct":11.8,"dps":25.793333333333333},{"rank":"Root #10","shipped":13,"distinct":13,"dps":37.56666666666667},{"rank":"Root #1 Zero-Day","shipped":14.8,"distinct":14.8,"dps":34.6}]
```

### 10. UT charge lab browser

```sh
SCIENCE_LAB_URL=http://127.0.0.1:19366/science.html node tools/verify_unified_theory_browser.cjs
```

```text
PASS: 375 native charge vectors, 75 skill cards, search and role filters, charge imbalance, reservation/cancel, all 50 recipes, 8-rank comparison, desktop/mobile without errors or horizontal overflow.
```

### 11. UT artwork browser

```sh
SCIENCE_LAB_URL=http://127.0.0.1:19366/science.html node tools/verify_science_art_browser.cjs
```

```text
PASS: 75 actual skill animations across all 8 ranks, 33 eight-frame auras, three distinct #1 subject titles, 21 subject completions, 10 Top 10 positions, 25 travel/field animations, all six direct transformations with the destination aura, cooldown, same-form guard, paused morph, moving frames, pause/play, completion, desktop/mobile; no script or asset-loading errors.
```

### 12. Shared editor and mastery integration browser

```sh
SCIENCE_EDITOR_PORT=19365 node tools/verify_science_editor_browser.cjs
```

```text
PASS: one Science Lab with no duplicate viewer, shared Coder/Isliid styling, 75 native recipes and animations, all 50 experiments, 375 charge vectors, 51 rank/form appearances without stacking, one persistent iframe, hidden animation suspension, preserved notebook/pause, all five mastery lists, isolated scientist writes/backups and mastery generation with a pinned #1, correct pending win points, desktop/mobile layout.
```

## Manual game follow-up

After this PR is merged, close the game, use Mod Manager Update/Check, and confirm native 0.10.37 / champion 0.2.37 with no dependency warnings. Replay xartE versus Binary Tigers game 1 with comparable stats/equipment; inspect `coder_damage_log.txt` for loaded/reused flags, cast shares, requested/discarded raw damage and actual HP loss. Ordinary attacks, damage-taken marks, defenses, engine reflection and item procs follow the engine: the 30% guard limits owned raw scripted emissions, not every possible observed HP change.

Exercise UT Salt Crystallization repeatedly, including replacement, destruction, death and respawn. Confirm the opaque crystal is displayed, its actions remain blocked, and no base-animation lookup error appears. Try the long-range setup combinations, all three #1 completion signatures, compact HUD position under actual health bars, and several UT/Coders together. Check FPS and collect the game log, `unified_theory_damage_log.txt` and crash report if another closure occurs. Installation, live game renderer timing/collision/mitigation, sprite-flip behavior and actual 5v5 FPS were not available in this cloud.

Round 124 is recorded in `playtest-notes.md`. All implementation, generated assets, release metadata, DLL, previews and verification are on the single branch above and one review PR.

## Changed files

- `Claude outputs/coder/coder_data.py`
- `Claude outputs/coder/coder_vfx.py`
- `Claude outputs/coder/preview/vfx.png`
- `Claude outputs/unified_theory/art.py`
- `Claude outputs/unified_theory/compact_art.py`
- `Claude outputs/unified_theory/cosmic_art.py`
- `Claude outputs/unified_theory/data.py`
- `Claude outputs/unified_theory/hud_art.py`
- `Claude outputs/unified_theory/mastery_art.py`
- `docs/playtest-notes.md`
- `docs/science-cosmic-coder-stacking-verification.md`
- `docs/unified-theory-art-inspector.png`
- `docs/unified-theory-aura-frames.png`
- `docs/unified-theory-auras.gif`
- `docs/unified-theory-cosmic-frames.png`
- `docs/unified-theory-cosmic.gif`
- `docs/unified-theory-effects.png`
- `docs/unified-theory-facing.png`
- `docs/unified-theory-hud.gif`
- `docs/unified-theory-hud.png`
- `docs/unified-theory-lab.png`
- `docs/unified-theory-mastery.png`
- `docs/unified-theory-peak-completions.gif`
- `docs/unified-theory-peak-completions.png`
- `docs/unified-theory-peak-curie.png`
- `docs/unified-theory-peak-einstein.png`
- `docs/unified-theory-peak-newton.png`
- `docs/unified-theory-ranked-cosmic.gif`
- `docs/unified-theory-scale.png`
- `docs/unified-theory-science-lab-study.png`
- `docs/unified-theory-science-lab-ui.png`
- `docs/unified-theory-skills-curie.png`
- `docs/unified-theory-skills-einstein.png`
- `docs/unified-theory-skills-newton.png`
- `docs/unified-theory-static-cards.png`
- `docs/unified-theory-static-einstein.png`
- `docs/unified-theory-top10.png`
- `docs/unified-theory-transform-frames.png`
- `docs/unified-theory-transforms.gif`
- `editor/coderlab.js`
- `editor/science-art-preview.json`
- `editor/science-art.js`
- `editor/science-mastery-preview.png`
- `editor/science-portraits.png`
- `editor/science.html`
- `mods/tfm2_custom/champion/tfm2_custom_coder.data_champion`
- `mods/tfm2_custom/champion/tfm2_custom_unified_theory.data_champion`
- `mods/tfm2_custom/champions/tfm2_custom_unified_theory#sheet.png`
- `mods/tfm2_custom/mod.mod_info`
- `mods/tfm2_custom/text/champion.i18n`
- `mods/tfm2_custom/vfx/coder_vfx#anim.fanim`
- `mods/tfm2_custom/vfx/coder_vfx#sheet.png`
- `mods/tfm2_custom/vfx/science_allocations#anim.fanim`
- `mods/tfm2_custom/vfx/science_allocations#sheet.png`
- `mods/tfm2_custom/vfx/science_charge#anim.fanim`
- `mods/tfm2_custom/vfx/science_charge#sheet.png`
- `mods/tfm2_custom/vfx/science_completion#anim.fanim`
- `mods/tfm2_custom/vfx/science_completion#sheet.png`
- `mods/tfm2_custom/vfx/science_crystal_body#anim.fanim`
- `mods/tfm2_custom/vfx/science_crystal_body#sheet.png`
- `mods/tfm2_custom/vfx/science_equipment0#anim.fanim`
- `mods/tfm2_custom/vfx/science_equipment0#sheet.png`
- `mods/tfm2_custom/vfx/science_equipment1#anim.fanim`
- `mods/tfm2_custom/vfx/science_equipment1#sheet.png`
- `mods/tfm2_custom/vfx/science_equipment2#anim.fanim`
- `mods/tfm2_custom/vfx/science_equipment2#sheet.png`
- `mods/tfm2_custom/vfx/science_impacts_t0#anim.fanim`
- `mods/tfm2_custom/vfx/science_impacts_t0#sheet.png`
- `mods/tfm2_custom/vfx/science_impacts_t1#anim.fanim`
- `mods/tfm2_custom/vfx/science_impacts_t1#sheet.png`
- `mods/tfm2_custom/vfx/science_impacts_t2#anim.fanim`
- `mods/tfm2_custom/vfx/science_impacts_t2#sheet.png`
- `mods/tfm2_custom/vfx/science_impacts_t3#anim.fanim`
- `mods/tfm2_custom/vfx/science_impacts_t3#sheet.png`
- `mods/tfm2_custom/vfx/science_meters#anim.fanim`
- `mods/tfm2_custom/vfx/science_meters#sheet.png`
- `mods/tfm2_custom/vfx/science_modifiers#sheet.png`
- `mods/tfm2_custom/vfx/science_notebook0#anim.fanim`
- `mods/tfm2_custom/vfx/science_notebook0#sheet.png`
- `mods/tfm2_custom/vfx/science_notebook1#anim.fanim`
- `mods/tfm2_custom/vfx/science_notebook1#sheet.png`
- `mods/tfm2_custom/vfx/science_notebook2#anim.fanim`
- `mods/tfm2_custom/vfx/science_notebook2#sheet.png`
- `mods/tfm2_custom/vfx/science_orbs#anim.fanim`
- `mods/tfm2_custom/vfx/science_orbs#sheet.png`
- `mods/tfm2_custom/vfx/science_outfits#sheet.png`
- `mods/tfm2_custom/vfx/science_packets_t0#anim.fanim`
- `mods/tfm2_custom/vfx/science_packets_t0#sheet.png`
- `mods/tfm2_custom/vfx/science_packets_t1#anim.fanim`
- `mods/tfm2_custom/vfx/science_packets_t1#sheet.png`
- `mods/tfm2_custom/vfx/science_packets_t2#anim.fanim`
- `mods/tfm2_custom/vfx/science_packets_t2#sheet.png`
- `mods/tfm2_custom/vfx/science_packets_t3#anim.fanim`
- `mods/tfm2_custom/vfx/science_packets_t3#sheet.png`
- `mods/tfm2_custom/vfx/science_personas#sheet.png`
- `mods/tfm2_custom/vfx/science_skills0_t0#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills0_t0#sheet.png`
- `mods/tfm2_custom/vfx/science_skills0_t1#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills0_t1#sheet.png`
- `mods/tfm2_custom/vfx/science_skills0_t2#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills0_t2#sheet.png`
- `mods/tfm2_custom/vfx/science_skills0_t3#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills0_t3#sheet.png`
- `mods/tfm2_custom/vfx/science_skills1_t0#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills1_t0#sheet.png`
- `mods/tfm2_custom/vfx/science_skills1_t1#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills1_t1#sheet.png`
- `mods/tfm2_custom/vfx/science_skills1_t2#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills1_t2#sheet.png`
- `mods/tfm2_custom/vfx/science_skills1_t3#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills1_t3#sheet.png`
- `mods/tfm2_custom/vfx/science_skills2_t0#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills2_t0#sheet.png`
- `mods/tfm2_custom/vfx/science_skills2_t1#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills2_t1#sheet.png`
- `mods/tfm2_custom/vfx/science_skills2_t2#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills2_t2#sheet.png`
- `mods/tfm2_custom/vfx/science_skills2_t3#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills2_t3#sheet.png`
- `mods/tfm2_custom/vfx/science_transforms#sheet.png`
- `mods/tfm2_custom_ai/mod.mod_info`
- `mods/tfm2_custom_ai/tfm2_custom_ai.dll`
- `native/tfm2_custom_ai/mod.mod_info`
- `native/tfm2_custom_ai/src/coder.rs`
- `native/tfm2_custom_ai/src/coder_damage_tests.rs`
- `native/tfm2_custom_ai/src/coder_trace.rs`
- `native/tfm2_custom_ai/src/lib.rs`
- `native/tfm2_custom_ai/src/unified_theory.rs`
- `native/tfm2_custom_ai/src/unified_theory_tests.rs`
- `tools/verify_coder.py`
- `tools/verify_science_art_browser.cjs`
- `tools/verify_unified_theory.py`
