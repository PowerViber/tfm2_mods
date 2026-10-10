# The Unified Theory — round 111 verification

Historical implementation report for 0.10.24. Current 0.10.25 mastery artwork and release checks: [round 112 verification](unified-theory-art-verification.md).

Native **0.10.24**, champion mod **0.2.24**, dependency **>=0.10.24**. Release metadata dates: **2026-10-10**. Base: main after PR #22 merged (`b1b5d5e`).

Implemented all 75 native skills, 50 experiment templates, symbolic notebook activation, CU reservation/imbalance, material/momentum, athlete mastery, owned packet/field physics, original scientist art, Science Lab and diagnostics. [Design/player guide](unified-theory-guide.md) · [Complete catalogue](unified-theory-catalogue.md) · [Visual preview](unified-theory-lab.png).

## Scope and verification source

The user approved the new champion. The attached `coder-verification.md` supplies regression requirements for changes that touch the shared native DLL, manager, metadata and localization; its instructions do not expand implementation scope. Coder combat logic, typing, execution, cooldowns, artwork and vector fixtures keep their main-branch behavior. Its README row now states the existing 100 functions and 13 languages correctly.

The new locale entry comes from `Claude outputs/unified_theory/data.py`. The Coder locale entry and champion data are byte-identical to main. Generated data and assets are changed through their generators. Native charge vectors are generated only by `SCIENCE_VECTORS=write cargo test --offline --release science_vectors`, then normally verified read-only.

## Setup and evidence

Used `. /workspace/.tfm2-env/env.sh`: Rust 1.99.0, Node 24.19.0, Python 3.12.14/Pillow. Windows linker: `/workspace/.tfm2-env/mingw/usr/bin/x86_64-w64-mingw32-gcc-posix`. Four build jobs. Browser checks use Chromium and Playwright Core. Start `node editor/server.js --no-open` first; the new browser verifier accepts `SCIENCE_LAB_URL` and `CHROMIUM_PATH`. Playwright Core can be supplied through `NODE_PATH` as below.

All final commands below succeeded, with **zero compiler warnings**. Native: **159 passed, 0 failed, 1 ignored**. Manager: **13 passed**. The ignored native test is the existing optional `scribble::real_data::real_pending` fixture.

## New checks

- Invoke every one of the 75 actual native effect handlers with valid preparation.
- Commit every stage of all 50 templates when their prerequisites are live.
- Start Grand Experiment from no preparation, pay its prerequisites, recover and reserve 96 CU, and successfully reach Decay Chain without filler-cast starvation.
- Exercise shared damage budgets across four enemies and the 35%-maximum-HP raw emission window.
- Conserve split payload and experiment identity; normalize tiny integer vectors correctly.
- Spend a failed current stage while refunding later reservations; refund everything uncommitted on target loss.
- Keep ordinary token timing identical at all ranks; repeat native fixture runs for deterministic results.
- Preserve notebook/resources/RNG across cloned simulations; use deterministic caster-based buff names.
- Share stun/forced-movement control limits, recognize only compatible chemical tags, and salvage each consumed reaction source once.
- Rotate bounded diagnostics into two complete readable files.
- Resolve all **767 views** over seven VFX sheets plus the original body; every atlas <=2048. Cover notebook/field refresh lifetimes and byte-identical regeneration.
- Reproduce **375 native charge vectors** exactly in Science Lab. Check 75 cards, search/role filters, charge imbalance, reservation/cancel, all 50 recipes, ten trials per each of eight ranks, desktop/mobile layout, no page errors or horizontal overflow.
- Pass the Coder checklist: **558 native lab vectors**, **5471 views/17 sheets**, native damage regressions, 100-row picker/rewrite, language filters, four rank arenas and ten-run comparison.

Science Lab is exact for charge/resource arithmetic. Its trajectory animation and mastery study are schematic, with no claim of full combat-engine or native-planner parity.

## Final command outputs

### Native release build

```sh
cargo build --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml
```

```text
Finished `release` profile [optimized] target(s) in 0.00s
```

### Native release tests

```sh
cargo test --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml
```

```text
Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 19.01s
     Running unittests src/lib.rs (native/tfm2_custom_ai/target/release/deps/tfm2_custom_ai-e5d070af18bd592e)

running 160 tests
test coder::damage_tests::diagnostic_presim_suppression_preserves_damage_and_does_not_consume_extra_boosts_or_rng ... ok
test coder::damage_tests::diagnostics_preserve_support_and_report_nonlethal_direct_hp_and_engine_kill_credit ... ok
test coder::damage_tests::daemon_and_cooldown_schedules_are_unchanged ... ok
test coder::damage_tests::diagnostics_attribute_chain_packets_and_live_scaled_persistent_hits ... ok
test coder::damage_tests::percentage_attacks_and_execute_boundaries_cannot_bypass_replay_nerfs ... ok
test coder::damage_tests::immediate_damage_and_burst_are_reduced_across_rigs ... ok
test coder::damage_tests::packets_keep_their_count_schedule_and_are_not_nerfed_twice ... ok
test coder::damage_tests::support_and_sign_flipped_healing_keep_their_original_strength ... ok
test coder::tests::a_saved_function_reloads_with_its_bugs_faster_off_an_ssd ... ok
test coder::damage_tests::replay_damage_is_halved_without_halving_support_and_quantum_stays_a_coin_flip ... ok
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
test coder::tests::rust_catches_bugs_at_a_price ... ok
test coder::tests::the_data_center_has_its_risks ... ok
test coder::tests::assembly_hits_hardest_and_breaks_most ... ok
test coder::tests::the_rig_follows_the_rank ... ok
test coder::tests::the_shop_buys_what_the_problems_call_for ... ok
test coder::tests::the_window_shows_three_lines ... ok
test coder::tests::believed_cost_is_rosier_at_low_rank ... ok
test coder::tests::wall_distance ... ok
test coder::tests::writing_is_deterministic_per_seed ... ok
test coder::trace::tests::batching_keeps_every_hit_and_waits_one_second_or_force ... ok
test coder::trace::tests::append_and_rotation_keep_two_complete_readable_logs ... ok
test coder::trace::tests::event_lines_report_observed_hp_loss_and_do_not_guess_kill_credit_source ... ok
test coder::trace::tests::event_and_byte_limits_flush_batches_and_preserve_sequence ... ok
test coder::trace::tests::presim_is_skipped_but_each_watched_origin_is_recorded ... ok
test coder::trace::tests::unknown_host_logs_only_the_first_instance_for_each_seed_and_caster ... ok
test coder::trace::tests::maps_are_bounded_and_evicted_pending_events_are_flushed ... ok
test flash::tests::ring_stays_on_map ... ok
test flash::tests::toward_stops_short ... ok
test gundam::tests::banish_ends_on_landing ... ok
test gundam::tests::burn_refreshes_not_stacks ... ok
test gundam::tests::exactly_one_wing_visual_per_phase ... ok
test gundam::tests::inner_knocks_outer_slows ... ok
test gundam::tests::landing_blocks_for_ally ... ok
test gundam::tests::no_attacks_while_rising_or_flying ... ok
test gundam::tests::every_visual_name_exists_in_the_data ... ok
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
test isliid::tests::complete_catalogue_has_drawable_legs ... ok
test isliid::tests::draw_press_does_not_hijack_a_ready_plan_sword ... ok
test isliid::tests::escort_roles_fit_the_ally ... ok
test isliid::tests::escorts_go_anywhere_two_per_teammate ... ok
test isliid::tests::every_mastery_rank_considers_simple_engravings ... ok
test isliid::tests::busy_fight_effect_budget ... ok
test isliid::tests::every_sword_state_has_a_moving_aura_source ... ok
test isliid::tests::every_sword_state_has_exactly_one_visual ... ok
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
test isliid::tests::every_visual_name_exists_in_the_data ... ok
test mod_power_tests::per_champion ... ok
test perf::tests::wrappers_forward_every_hook ... ok
test press::tests::flags_are_read_by_exact_tick ... ok
test scribble::real_data::real_pending ... ignored
test scribble::tests::locked_tier_goes_to_the_next_spell ... ok
test scribble::tests::memory_round_trip_and_merge ... ok
test scribble::tests::opener_follows_the_situation ... ok
test scribble::tests::overreach_slips ... ok
test scribble::tests::page_flip_keeps_mid_and_bases ... ok
test perf::tests::effect_counts_are_off_without_the_flag ... ok
test scribble::tests::ranks ... ok
test scribble::tests::recipes_are_unique_and_tiered ... ok
test scribble::tests::rookies_value_big_spells_by_their_odds ... ok
test scribble::tests::top_ten_needs_300_points_and_is_ordered ... ok
test steve::tests::straight_wall_and_detour ... ok
test tactics::tests::parses_lines ... ok
test unified_theory::integration::budget_and_rolling_guard_cover_multiple_enemies ... ok
test unified_theory::integration::chemical_cleanup_is_limited_to_scientific_tags ... ok
test unified_theory::integration::cloned_simulations_preserve_the_notebook_and_rng ... ok
test unified_theory::integration::every_skill_has_an_executable_native_handler ... ok
test steve::tests::move_rules_skip_only_when_nothing_applies ... ok
test unified_theory::integration::every_template_commits_all_stages_when_its_prerequisites_are_live ... ok
test unified_theory::integration::grand_experiment_can_charge_after_real_prerequisites ... ok
test unified_theory::integration::masteries_keep_common_token_speed ... ok
test unified_theory::integration::notebook_critical_failure_spends_current_and_refunds_dependents ... ok
test unified_theory::integration::prerequisite_preparation_is_finite_for_every_template ... ok
test unified_theory::integration::quench_salvages_each_new_source_once ... ok
test unified_theory::integration::status_and_buff_names_do_not_depend_on_simulation_addresses ... ok
test unified_theory::integration::stuns_and_displacements_share_the_control_window ... ok
test unified_theory::integration::target_loss_refunds_every_uncommitted_stage ... ok
test unified_theory::integration::trace_rotation_preserves_two_complete_logs ... ok
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
test unified_theory::integration::fixtures_are_reproducible_for_all_ranks ... ok
test coder::tests::the_copilots_have_their_profiles ... ok
test coder::tests::the_gap_grows_with_every_rank ... ok

test result: ok. 159 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.53s
```

### Science assets and exact charge parity

```sh
python tools/verify_unified_theory.py
```

```text
375 exact native charge vectors; all 8 ranks deterministic
75 unique skills; 50 bounded recipes; generated files current
Original body, three personas, notebooks, meter and 20 animated scientific effects generated
tfm2_custom_unified_theory 755 effects 11 persona / mastery overlays
Verified 767 views over 7 VFX sheets; original body <=2048; notebook lifetimes; regeneration byte-identical.
```

### Science Lab desktop/mobile browser

```sh
NODE_PATH=/workspace/.tfm2-env/browser/node_modules node tools/verify_unified_theory_browser.cjs
```

```text
PASS: 375 native charge vectors, 75 skill cards, search and role filters, charge imbalance, reservation/cancel, all 50 recipes, 8-rank comparison, desktop/mobile without errors or horizontal overflow.
```

### Coder verifier

```sh
python tools/verify_coder.py --local
```

```text
Code lab: tables, languages, functions, models and hardware match coder.rs; 558 runs reproduced exactly
Verified 5471 Coder views over 17 sheets (each <= 2048), the rig layers, the top-rank effects, crests and every name coder.rs builds
```

### Coder JavaScript checklist

```sh
node -c editor/coderlab.js
node -e "const l=require('./editor/coderlab.js'); console.log(l.selfTest()); for(const [r,p] of l.LADDER) l.simulateSkirmish(r,p,1);"
```

```text
true
```

### Coder browser checklist

```sh
node /workspace/.tfm2-env/browser/verify-coder.cjs
```

```text
PASS: browser parity, 100 rows, language filtering, picker/rewrite, 4 rank arenas, 10-run comparison, no page errors/NaN.
[{"rank":"Script Kiddie","shipped":0.5,"distinct":0.5,"dps":0.20666666666666667},{"rank":"Intern","shipped":1.5,"distinct":1.5,"dps":1.7866666666666666},{"rank":"Junior","shipped":3.7,"distinct":3.7,"dps":9.47},{"rank":"Developer","shipped":6.6,"distinct":6.6,"dps":58.17666666666666},{"rank":"Senior","shipped":8.6,"distinct":8.6,"dps":53.230000000000004},{"rank":"Staff","shipped":9.3,"distinct":9.3,"dps":34.766666666666666},{"rank":"Architect","shipped":10,"distinct":10,"dps":65.46333333333334},{"rank":"Root #10","shipped":11.7,"distinct":11.7,"dps":82.28},{"rank":"Root #1 Zero-Day","shipped":14.8,"distinct":14.8,"dps":104.1}]
```

### Manager release tests

```sh
cargo test --offline --release -j4 --manifest-path tools/manager/Cargo.toml
```

```text
Compiling tfm2_manager v1.0.0 (/workspace/tfm2_mods/tools/manager)
    Finished `release` profile [optimized] target(s) in 3.11s
     Running unittests src/main.rs (tools/manager/target/release/deps/tfm2_manager-756fc6bf9fcfdf44)

running 13 tests
test core::tests::build_outputs_reset_before_pull ... ok
test core::tests::champion_data_errors_name_the_bad_tag ... ok
test core::tests::check_reports_mismatch ... ok
test core::tests::log_scan_finds_load_errors ... ok
test core::tests::install_copies_and_keeps_plans ... ok
test core::tests::perf_log_is_read ... ok
test core::tests::mods_json_enable ... ok
test core::tests::native_version_meets_the_champions_requirement ... ok
test core::tests::repo_root_found_from_exe_dir ... ok
test core::tests::stale_duplicates_go_to_backup ... ok
test core::tests::stamps_are_dates ... ok
test core::tests::vdf_and_manifest_parse ... ok
test core::tests::the_repos_champions_all_load ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

### Native Windows release

```sh
cargo build --offline --release -j4 --target x86_64-pc-windows-gnu --manifest-path native/tfm2_custom_ai/Cargo.toml
```

```text
Finished `release` profile [optimized] target(s) in 0.00s
```

### Manager Windows release

```sh
cargo build --offline --release -j4 --target x86_64-pc-windows-gnu --manifest-path tools/manager/Cargo.toml
```

```text
Compiling tfm2_manager v1.0.0 (/workspace/tfm2_mods/tools/manager)
    Finished `release` profile [optimized] target(s) in 3.94s
```

## Shipped Windows binaries

Build and shipped-copy SHA-256 values match:

- `mods/tfm2_custom_ai/tfm2_custom_ai.dll`: `cbabadc5c7b56c57b80d050ebb3df00cb8f91c3ae796ad83e81c4c227d3fea65`
- `TFM2 Mod Manager.exe`: `01780422f34e0b040eb3e299ee0f6a9f6f1227462728f402aa0f093383852e91`

## Remaining real-game checks

A Teamfight Manager 2 match cannot be run in this cloud workspace. Windows binaries were cross-built, not launched here. In-game validation still needs:

1. Native 0.10.24 loading without dependency warnings; champion name and description present.
2. Low/high mastery, scientist swaps, notebook readability/flicker, crests, charge meters and death/respawn.
3. Lenses, curves, orbits, reflection, wall clipping, field expiry and crystal destruction. General engine map blocking by crystal units is not assumed.
4. Alpha range, multi-target Gamma/Decay conservation, minion/champion targeting, and performance with two scientists plus a Coder.
5. Real mitigation, crit, item procs and kill attribution. Engine-added item damage and ordinary attacks are outside the raw scientific emission guard; observed HP changes are logged rather than claimed universally capped.
6. Athlete progress across matches/relaunches. Mod Manager option 4 includes `unified_theory_log.txt` and `unified_theory_damage_log.txt` alongside the existing Coder diagnostics.

The exposed API cannot mutate or delete foreign projectiles, so interception/filtering use supported protection. Length Contraction uses the circular radius multiplier. Chemical cleanup only recognizes typed scientific buffs. No unrestricted buff removal or arbitrary source-code execution is added.

## Changed files

- `Claude outputs/unified_theory/art.py`
- `Claude outputs/unified_theory/catalogue.json`
- `Claude outputs/unified_theory/data.py`
- `Claude outputs/unified_theory/generate.py`
- `README.md`
- `TFM2 Mod Manager.exe`
- `docs/playtest-notes.md`
- `docs/unified-theory-catalogue.md`
- `docs/unified-theory-guide.md`
- `docs/unified-theory-lab.png`
- `docs/unified-theory-verification.md`
- `editor/index.html`
- `editor/science-portraits.png`
- `editor/science.css`
- `editor/science.html`
- `editor/server.js`
- `editor/unified-theory-data.js`
- `editor/unified-theorylab.js`
- `mods/tfm2_custom/champion/tfm2_custom_unified_theory.data_champion`
- `mods/tfm2_custom/champions/tfm2_custom_unified_theory#anim.fanim`
- `mods/tfm2_custom/champions/tfm2_custom_unified_theory#sheet.png`
- `mods/tfm2_custom/mod.mod_info`
- `mods/tfm2_custom/text/champion.i18n`
- `mods/tfm2_custom/vfx/science_allocations#anim.fanim`
- `mods/tfm2_custom/vfx/science_allocations#sheet.png`
- `mods/tfm2_custom/vfx/science_meters#anim.fanim`
- `mods/tfm2_custom/vfx/science_meters#sheet.png`
- `mods/tfm2_custom/vfx/science_notebook0#anim.fanim`
- `mods/tfm2_custom/vfx/science_notebook0#sheet.png`
- `mods/tfm2_custom/vfx/science_notebook1#anim.fanim`
- `mods/tfm2_custom/vfx/science_notebook1#sheet.png`
- `mods/tfm2_custom/vfx/science_notebook2#anim.fanim`
- `mods/tfm2_custom/vfx/science_notebook2#sheet.png`
- `mods/tfm2_custom/vfx/science_personas#anim.fanim`
- `mods/tfm2_custom/vfx/science_personas#sheet.png`
- `mods/tfm2_custom/vfx/science_vfx#anim.fanim`
- `mods/tfm2_custom/vfx/science_vfx#sheet.png`
- `mods/tfm2_custom_ai/mod.mod_info`
- `mods/tfm2_custom_ai/tfm2_custom_ai.dll`
- `native/tfm2_custom_ai/mod.mod_info`
- `native/tfm2_custom_ai/src/lib.rs`
- `native/tfm2_custom_ai/src/steve.rs`
- `native/tfm2_custom_ai/src/unified_theory.rs`
- `native/tfm2_custom_ai/src/unified_theory_data.rs`
- `native/tfm2_custom_ai/src/unified_theory_math.rs`
- `native/tfm2_custom_ai/src/unified_theory_tests.rs`
- `native/tfm2_custom_ai/src/unified_theory_vectors.txt`
- `tools/manager/src/main.rs`
- `tools/verify_unified_theory.py`
- `tools/verify_unified_theory_browser.cjs`
