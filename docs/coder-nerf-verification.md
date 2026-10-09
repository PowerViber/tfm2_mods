# Coder round 109 verification

Native 0.10.22; champion mod 0.2.22; dependency >=0.10.22. Base: main with PR #20 merged (641a36f).

All automated checklist checks passed. The seven added native tests use a host-API capture fixture to exercise actual native execution, queued hits, persistent/replayed damage, percentage attacks, execution boundaries, Quantum outcomes, support amounts and timing. They do not emulate game defenses or replace a real-match playtest.

## Changed files

- Claude outputs/coder/coder_data.py
- editor/coderlab.js
- mods/tfm2_custom/mod.mod_info
- mods/tfm2_custom/text/champion.i18n (generated)
- mods/tfm2_custom_ai/mod.mod_info
- mods/tfm2_custom_ai/tfm2_custom_ai.dll (rebuilt)
- native/tfm2_custom_ai/mod.mod_info
- native/tfm2_custom_ai/src/coder.rs
- native/tfm2_custom_ai/src/coder_damage_tests.rs
- native/tfm2_custom_ai/src/lib.rs
- tools/verify_coder.py
- docs/playtest-notes.md (round 109)
- docs/coder-nerf-verification.md

## Tool activation

Commands ran in the prepared cloud environment after `. /workspace/.tfm2-env/env.sh`. Rust 1.99.0, Python 3.12.14/Pillow 12.3.0, Node 24.19.0. Build parallelism: four jobs. Windows linker: /workspace/.tfm2-env/mingw/usr/bin/x86_64-w64-mingw32-gcc-posix, supplied through CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER. The Windows cross-build uses the Linux toolchain plus the Windows target.

## Checklist outputs

### 1. Native release build

```bash
cd native/tfm2_custom_ai && cargo build --offline --release -j 4
```

Exit status: 0.

```text
   Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 15.63s
```

### 2. Native release tests

```bash
cd native/tfm2_custom_ai && cargo test --offline --release -j 4
```

Exit status: 0.

```text
   Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 16.75s
     Running unittests src/lib.rs (native/tfm2_custom_ai/target/release/deps/tfm2_custom_ai-e5d070af18bd592e)

running 124 tests
test coder::damage_tests::packets_keep_their_count_schedule_and_are_not_nerfed_twice ... ok
test coder::damage_tests::percentage_attacks_and_execute_boundaries_cannot_bypass_replay_nerfs ... ok
test coder::damage_tests::daemon_and_cooldown_schedules_are_unchanged ... ok
test coder::damage_tests::immediate_damage_and_burst_are_reduced_across_rigs ... ok
test coder::damage_tests::replay_damage_is_halved_without_halving_support_and_quantum_stays_a_coin_flip ... ok
test coder::damage_tests::support_and_sign_flipped_healing_keep_their_original_strength ... ok
test coder::tests::a_saved_function_reloads_with_its_bugs_faster_off_an_ssd ... ok
test coder::damage_tests::replays_retain_damage_scaling_in_drones_tethers_and_walls ... ok
test coder::tests::every_view_name_exists ... ok
test coder::tests::gpu_and_cloud_functions ... ok
test coder::tests::choosing_is_cheap ... ok
test coder::tests::he_doesnt_spam_the_same_function ... ok
test coder::tests::heat_throttles_the_clock_and_parts_change_the_rig ... ok
test coder::tests::lab_vectors ... ok
test coder::tests::mining_slows_his_typing ... ok
test coder::tests::one_status_line_at_a_time ... ok
test coder::tests::root_interpolates_to_number_one ... ok
test coder::tests::haskell_ships_clean_and_lua_is_quick ... ok
test coder::tests::scripts_keep_him_writing ... ok
test coder::tests::slots_grow_with_rank_and_autoscale ... ok
test coder::tests::the_code_table_matches_the_functions ... ok
test coder::tests::assembly_hits_hardest_and_breaks_most ... ok
test coder::tests::the_data_center_has_its_risks ... ok
test coder::tests::believed_cost_is_rosier_at_low_rank ... ok
test coder::tests::the_rig_follows_the_rank ... ok
test coder::tests::the_shop_buys_what_the_problems_call_for ... ok
test coder::tests::the_window_shows_three_lines ... ok
test coder::tests::wall_distance ... ok
test coder::tests::writing_is_deterministic_per_seed ... ok
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
test steve::tests::straight_wall_and_detour ... ok
test tactics::tests::parses_lines ... ok
test valorant::tests::blind_scales_with_distance ... ok
test valorant::tests::disk_segment ... ok
test valorant::tests::gun_prices ... ok
test valorant::tests::half_pit_edge_through_the_centre ... ok
test valorant::tests::paranoia_trade_through_a_teammate ... ok
test version_tests::every_version_agrees ... ok
test coder::tests::rust_catches_bugs_at_a_price ... ok
test coder::tests::the_copilots_have_their_profiles ... ok
test coder::tests::the_gap_grows_with_every_rank ... ok

test result: ok. 123 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.65s
```

### 3. Native/editor/assets verifier

```bash
python tools/verify_coder.py --local
```

Exit status: 0.

```text
Code lab: tables, languages, functions, models and hardware match coder.rs; 558 runs reproduced exactly
Verified 5471 Coder views over 17 sheets (each <= 2048), the rig layers, the top-rank effects, crests and every name coder.rs builds
```

### 4. Manager release tests

```bash
cd tools/manager && cargo test --offline --release -j 4
```

Exit status: 0.

```text
   Compiling tfm2_manager v1.0.0 (/workspace/tfm2_mods/tools/manager)
    Finished `release` profile [optimized] target(s) in 2.55s
     Running unittests src/main.rs (tools/manager/target/release/deps/tfm2_manager-756fc6bf9fcfdf44)

running 13 tests
test core::tests::build_outputs_reset_before_pull ... ok
test core::tests::champion_data_errors_name_the_bad_tag ... ok
test core::tests::install_copies_and_keeps_plans ... ok
test core::tests::log_scan_finds_load_errors ... ok
test core::tests::mods_json_enable ... ok
test core::tests::check_reports_mismatch ... ok
test core::tests::native_version_meets_the_champions_requirement ... ok
test core::tests::perf_log_is_read ... ok
test core::tests::repo_root_found_from_exe_dir ... ok
test core::tests::stale_duplicates_go_to_backup ... ok
test core::tests::stamps_are_dates ... ok
test core::tests::vdf_and_manifest_parse ... ok
test core::tests::the_repos_champions_all_load ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

### 5. JavaScript syntax and every-rank smoke test

```bash
node -c editor/coderlab.js
node -e "const l=require('./editor/coderlab.js'); console.log(l.selfTest()); for (const [r,p] of l.LADDER) l.simulateSkirmish(r,p,1);"
```

Exit status: 0.

```text
true
```

### 6. Windows release build

```bash
cd native/tfm2_custom_ai && cargo build --offline --release --target x86_64-pc-windows-gnu -j 4
```

Exit status: 0.

```text
   Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 14.35s
```

### 7. Real Chromium editor checks

```bash
node /workspace/.tfm2-env/browser/verify-coder.cjs
```

Exit status: 0.

```text
PASS: browser parity, 100 rows, language filtering, picker/rewrite, 4 rank arenas, 10-run comparison, no page errors/NaN.
[{"rank":"Script Kiddie","shipped":0.5,"distinct":0.5,"dps":0.20666666666666667},{"rank":"Intern","shipped":1.5,"distinct":1.5,"dps":1.7866666666666666},{"rank":"Junior","shipped":3.7,"distinct":3.7,"dps":9.47},{"rank":"Developer","shipped":6.6,"distinct":6.6,"dps":58.17666666666666},{"rank":"Senior","shipped":8.6,"distinct":8.6,"dps":53.230000000000004},{"rank":"Staff","shipped":9.3,"distinct":9.3,"dps":34.766666666666666},{"rank":"Architect","shipped":10,"distinct":10,"dps":65.46333333333334},{"rank":"Root #10","shipped":11.7,"distinct":11.7,"dps":82.28},{"rank":"Root #1 Zero-Day","shipped":14.8,"distinct":14.8,"dps":104.1}]
```

## Release and integrity

Both DLL files have SHA-256:

```text
b0815c3dc096521a646d0ce1636667e0efc7ccdc9e87a32f6ffb3ea32fb063f0
```

- Build: native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll
- Shipped copy: mods/tfm2_custom_ai/tfm2_custom_ai.dll
- Copy verified byte-for-byte; packaged file is an x86-64 Windows DLL.
- All release build/test logs contain zero compiler warnings and zero errors.
- every_version_agrees passed.
- TFM2 Mod Manager.exe was not rebuilt because manager source was unchanged.
- Ran coder_data.py last from Claude outputs/coder; generated champion data was identical, while localization changed. No handwritten generated-file edits.
- Generated function code, code art, font, and coder_vectors.txt were unchanged.
- Source comparison confirmed cooldown_ticks, choose, step_typing, knobs, step_overclock, step_mining and step_shop unchanged. Native regression tests confirm daemon and packet/persistent-effect schedules.
- Visually checked functions.png, vfx2.png and themes.png. Existing code/art previews are unchanged; the browser successfully rendered the Code lab and its assets. Some displayed code snippets retain their former numeric examples; the updated description and native damage rules define the new balance. In-game text readability is still part of the playtest.
- Fixed the arena's run-count display and comparison aggregation to count run records. Chromium verified numeric counts without `[object Object]` output.
- git diff --check passed.

## Limits and follow-up

The Code lab reproduces typing exactly but remains a reduced combat model. Its new burst and finisher cases do not make every one of the 100 functions an exact match simulation. Rank comparison had rising shipped/distinct counts; DPS is not strictly monotonic between every rank.

Game deployment, load popup, real-match survivability, effect placement, performance/FPS and game/coder/perf logs were not checked: Teamfight Manager 2 and saves are absent. Playtest low ranks and Root/Zero-Day, check whether healthy opponents survive bursts and whether below-8% kill9 executions are readable.

All changes are on `codex/coder-damage-nerf`, pushed to origin, in [PR #21](https://github.com/PowerViber/tfm2_mods/pull/21). The branch's remote commit matched the local commit after pushing. No merge was performed.

GitHub API requests initially failed with a proxy CONNECT 403. An additive draft network setting for api.github.com was saved; that draft alone does not publish environment configuration. The later PR creation request succeeded, so API access did not block delivery.
