# Walking Unified Experiment — round 112 verification

Native **0.10.25**, champion mod **0.2.25**, dependency **>=0.10.25**; both native metadata copies match `VERSION`. Release dates: **2026-10-10**. This follow-up extends the unmerged Unified Theory branch after `5d0f5cd`; PR #23 remains the single implementation PR.

The scientist now has seven distinct ordinary emblems and ten numbered Top 10 variants, eight outfits per scientist, evolving instruments/laboratories, separate podium equipment, successful-combination assembly and #1's projected shadow. All 75 skills have four cast-art stages; fields and sixteen-direction packets evolve by the same mastery tiers. Split/catalyst/decay preparation has persistent sigils. The Science Lab inspector animates the actual runtime pixels.

[Every rank](unified-theory-mastery.png) · [Top 10](unified-theory-top10.png) · [Skill stages](unified-theory-effects.png) · [Inspector](unified-theory-art-inspector.png) · [Player guide](unified-theory-guide.md)

## What changed

- `Claude outputs/unified_theory/mastery_art.py`: reproducible badges, torso outfits, moving equipment/panels, four-tier casts/fields, directional packet art, modifier sigils, completions, echoes and original-pixel previews.
- `art.py` / `data.py`: integrate the new generator; replace ground discs; classify front/back/body/badge layers; render completion behind the face. Generated champion JSON and atlases are never edited manually.
- `native/tfm2_custom_ai/src/unified_theory.rs`: retain mastery's existing Top 10 position for artwork, choose visual tiers, manage only the needed persistent layers, keep a bounded cosmetic queue, and use actual motion to orient orbit graphics. Combat arithmetic, notebook timing, cooldowns and RNG decisions retain their previous behavior.
- `unified_theory_tests.rs`: host captures for effects/layer lifecycle plus five meaningful visual regression tests.
- `editor/science-art.js`, `science.html`, `science.css`, `server.js`: original-asset inspector, all skills/ranks/positions, pause/play and completion; source atlases work without an installed game.
- `tools/verify_unified_theory.py` / `verify_science_art_browser.cjs`: asset uniqueness, evolving tiers, every directional tag, reproducible generation and functional browser coverage.
- Generated runtime assets, previews, player guide, release metadata, `VERSION`, shipped native DLL and round 112 playtest notes.

The manager source and EXE were unchanged by this follow-up, so its EXE was not rebuilt. Its existing SHA256 is `01780422f34e0b040eb3e299ee0f6a9f6f1227462728f402aa0f093383852e91`.

## Verification scope

The user's request authorizes the scientist artwork implementation. The attached `coder-verification.md` is used for shared-DLL, editor and release regressions; its historical branch/version examples and Coder-specific regeneration steps do not expand this task into a Coder redesign. Read the supplied breakdown's file map and rounds 101–108. Coder generator inputs, runtime art, champion data, sources, typing vectors, Code Lab, locale entry and manager source/EXE match the preceding commit. Neither Coder nor scientific charge vectors were rewritten.

Used `. /workspace/.tfm2-env/env.sh`: Rust 1.99.0, Python 3.12/Pillow, Node 24, installed Chromium/Playwright Core and the existing MinGW linker. Four build jobs; offline Cargo builds/tests. No new environment installation or configuration was needed.

Results: **164 native tests passed, 0 failed, 1 existing ignored** (`scribble::real_data::real_pending`); **13 manager tests passed**. All Linux/Windows builds and test compilations had **0 warnings**. `every_version_agrees` passed. **2144 views over 35 VFX sheets**, each <=2048; all regenerated assets and original-pixel previews reproduce identical bytes. The 300 cast variants have unique signatures and every skill evolves through four stages. All 375 native charge vectors still match the lab. Coder's 558 runs and 5471 views/17 sheets pass unchanged.

New native tests verify:

- All runtime visual names, ranks, Top 10 positions, modifier masks, fields, completions and packet directions resolve.
- Podium positions leave combat output, timing, charge/resources, cooldowns and RNG identical at every rank; VFX stay within six requests/update and the pending queue stays bounded.
- Existing loops are retained, changed layers are replaced, a missing layer is restored individually, gameplay buffs survive visual swaps, and death clears visual layers and pending effects.
- Only successful multi-stage combinations emit completion art; single casts and failed commitments do not. Busy frames keep queued art for the next available frame.
- Orbit artwork uses actual motion without changing the combat packet vector.

Browser checks cover all 75 skills across eight ranks, all ten leaderboard positions, moving frames, pause/play, completion, desktop/mobile layout and no script/asset-loading errors. Existing scientific charge controls, all 50 recipes and comparisons pass; the Coder picker/rewrite, 100-row table, language filters, four rank arenas and ten-run comparison pass.

## Command outputs

Commands ran from the repository root with the environment above. Browser commands use `NODE_PATH=/workspace/.tfm2-env/browser/node_modules`, `SCIENCE_LAB_URL=http://127.0.0.1:19362/science.html` and `node editor/server.js --no-open --port 19362`. The existing Coder browser script was copied to `/tmp` with only its base URL changed from port 7272 to 19362.

### Linux native release build

```sh
cargo build --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml
```

```text
   Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 14.99s
```

### Full native release tests

```sh
cargo test --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml
```

```text
   Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 19.31s
     Running unittests src/lib.rs (native/tfm2_custom_ai/target/release/deps/tfm2_custom_ai-e5d070af18bd592e)

running 165 tests
test coder::damage_tests::daemon_and_cooldown_schedules_are_unchanged ... ok
test coder::damage_tests::diagnostics_preserve_support_and_report_nonlethal_direct_hp_and_engine_kill_credit ... ok
test coder::damage_tests::diagnostic_presim_suppression_preserves_damage_and_does_not_consume_extra_boosts_or_rng ... ok
test coder::damage_tests::diagnostics_attribute_chain_packets_and_live_scaled_persistent_hits ... ok
test coder::damage_tests::percentage_attacks_and_execute_boundaries_cannot_bypass_replay_nerfs ... ok
test coder::damage_tests::immediate_damage_and_burst_are_reduced_across_rigs ... ok
test coder::damage_tests::packets_keep_their_count_schedule_and_are_not_nerfed_twice ... ok
test coder::tests::a_saved_function_reloads_with_its_bugs_faster_off_an_ssd ... ok
test coder::damage_tests::replay_damage_is_halved_without_halving_support_and_quantum_stays_a_coin_flip ... ok
test coder::damage_tests::support_and_sign_flipped_healing_keep_their_original_strength ... ok
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
test unified_theory::integration::budget_and_rolling_guard_cover_multiple_enemies ... ok
test unified_theory::integration::chemical_cleanup_is_limited_to_scientific_tags ... ok
test unified_theory::integration::cloned_simulations_preserve_the_notebook_and_rng ... ok
test unified_theory::integration::completion_art_requires_a_successful_combination_and_respects_budget ... ok
test coder::tests::rust_catches_bugs_at_a_price ... ok
test unified_theory::integration::every_skill_has_an_executable_native_handler ... ok
test unified_theory::integration::every_template_commits_all_stages_when_its_prerequisites_are_live ... ok
test unified_theory::integration::fixtures_are_reproducible_for_all_ranks ... ok
test unified_theory::integration::grand_experiment_can_charge_after_real_prerequisites ... ok
test unified_theory::integration::masteries_keep_common_token_speed ... ok
test unified_theory::integration::mastery_art_does_not_change_combat_resources_or_rng ... ok
test unified_theory::integration::notebook_critical_failure_spends_current_and_refunds_dependents ... ok
test unified_theory::integration::orbit_art_tracks_motion_without_changing_the_packet_vector ... ok
test unified_theory::integration::prerequisite_preparation_is_finite_for_every_template ... ok
test unified_theory::integration::quench_salvages_each_new_source_once ... ok
test unified_theory::integration::status_and_buff_names_do_not_depend_on_simulation_addresses ... ok
test unified_theory::integration::stuns_and_displacements_share_the_control_window ... ok
test unified_theory::integration::target_loss_refunds_every_uncommitted_stage ... ok
test unified_theory::integration::trace_rotation_preserves_two_complete_logs ... ok
test unified_theory::integration::visual_layers_are_replaced_restored_and_cleared_without_stacking ... ok
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
test coder::tests::the_copilots_have_their_profiles ... ok
test unified_theory::integration::every_mastery_visual_and_direction_resolves ... ok
test coder::tests::the_gap_grows_with_every_rank ... ok

test result: ok. 164 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.71s
```

### Scientific assets and math

```sh
python tools/verify_unified_theory.py
```

```text
375 exact native charge vectors; all 8 ranks deterministic
75 unique skills; 50 bounded recipes; generated files current
Walking Unified Experiment: 17 animated badges, 24 outfits, 3 laboratories, 300 skill casts, 896 directional packet tags, 44 fields, modifiers and podium completions
Original body, three personas, notebooks, meter and mastery art generated
tfm2_custom_unified_theory 2005 effects 138 persona / mastery overlays
Verified 2144 views over 35 VFX sheets; original body <=2048; notebook lifetimes; regeneration byte-identical.
Art: 17 animated rank/Top 10 badges, 24 outfits, 300 distinct skill casts, 44 evolving fields and 896 directional packet tags; podium equipment and previews verified.
```

### Coder regression verifier

```sh
python tools/verify_coder.py --local
```

```text
Code lab: tables, languages, functions, models and hardware match coder.rs; 558 runs reproduced exactly
Verified 5471 Coder views over 17 sheets (each <= 2048), the rig layers, the top-rank effects, crests and every name coder.rs builds
```

### Manager tests

```sh
cargo test --offline --release -j4 --manifest-path tools/manager/Cargo.toml
```

```text
    Finished `release` profile [optimized] target(s) in 0.01s
     Running unittests src/main.rs (tools/manager/target/release/deps/tfm2_manager-756fc6bf9fcfdf44)

running 13 tests
test core::tests::build_outputs_reset_before_pull ... ok
test core::tests::champion_data_errors_name_the_bad_tag ... ok
test core::tests::check_reports_mismatch ... ok
test core::tests::log_scan_finds_load_errors ... ok
test core::tests::mods_json_enable ... ok
test core::tests::native_version_meets_the_champions_requirement ... ok
test core::tests::perf_log_is_read ... ok
test core::tests::install_copies_and_keeps_plans ... ok
test core::tests::stale_duplicates_go_to_backup ... ok
test core::tests::stamps_are_dates ... ok
test core::tests::vdf_and_manifest_parse ... ok
test core::tests::the_repos_champions_all_load ... ok
test core::tests::repo_root_found_from_exe_dir ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

### Coder JS syntax and rank smoke test

```sh
node -c editor/coderlab.js
node -e "const l=require('./editor/coderlab.js'); console.log(l.selfTest()); for (const [r,p] of l.LADDER) l.simulateSkirmish(r,p,1);"
```

```text
true
```

### Windows native DLL

```sh
cargo build --offline --release --target x86_64-pc-windows-gnu -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml
```

```text
   Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 16.46s
```

### Built and shipped DLL hashes

```sh
sha256sum native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll mods/tfm2_custom_ai/tfm2_custom_ai.dll
```

```text
95a00ddc13271757cc5810cdd468d88207f999c53bd865d2d6f2ff7d78ec2024  native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll
95a00ddc13271757cc5810cdd468d88207f999c53bd865d2d6f2ff7d78ec2024  mods/tfm2_custom_ai/tfm2_custom_ai.dll
```

### Scientific browser regression

```sh
node tools/verify_unified_theory_browser.cjs
```

```text
PASS: 375 native charge vectors, 75 skill cards, search and role filters, charge imbalance, reservation/cancel, all 50 recipes, 8-rank comparison, desktop/mobile without errors or horizontal overflow.
```

### Actual-art browser regression

```sh
node tools/verify_science_art_browser.cjs
```

```text
PASS: 75 actual skill animations across all 8 ranks, 10 Top 10 positions, moving frames, pause/play, completion animation, desktop/mobile layout; no script or asset-loading errors.
```

### Coder browser regression

```sh
node /tmp/science-art-coder-browser.cjs
```

```text
PASS: browser parity, 100 rows, language filtering, picker/rewrite, 4 rank arenas, 10-run comparison, no page errors/NaN.
[{"rank":"Script Kiddie","shipped":0.5,"distinct":0.5,"dps":0.20666666666666667},{"rank":"Intern","shipped":1.5,"distinct":1.5,"dps":1.7866666666666666},{"rank":"Junior","shipped":3.7,"distinct":3.7,"dps":9.47},{"rank":"Developer","shipped":6.6,"distinct":6.6,"dps":58.17666666666666},{"rank":"Senior","shipped":8.6,"distinct":8.6,"dps":53.230000000000004},{"rank":"Staff","shipped":9.3,"distinct":9.3,"dps":34.766666666666666},{"rank":"Architect","shipped":10,"distinct":10,"dps":65.46333333333334},{"rank":"Root #10","shipped":11.7,"distinct":11.7,"dps":82.28},{"rank":"Root #1 Zero-Day","shipped":14.8,"distinct":14.8,"dps":104.1}]
```

Additional syntax checks passed: `node -c editor/server.js`, `node -c editor/science-art.js`, `node -c tools/verify_science_art_browser.cjs`, Python generator/verifier compilation, and `git diff --check`.

## In-game checks still required

No Windows game match is available in cloud. Install native 0.10.25/champions 0.2.25 through Mod Manager, confirm no load/dependency errors, and test low mastery plus Unified Mind #1. Check badge placement in both facings, body/equipment layering during movement/attacks/death, respawn cleanup, scientist switching, actual Top 10 position, all podium signatures, multi-stage completion/shadow, field refresh and packet orientation through curves/reflection/orbit. Run a crowded match with two scientists and a Coder to assess notebook flicker, readability and FPS. Artwork and browser/native checks do not establish in-game rendering performance or combat-engine/item balance.
