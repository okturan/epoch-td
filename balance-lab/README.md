# EPOCH Balance Lab

The lab is a Rust simulation and evolutionary-search companion to the browser
game. `index.html`, `sim.js`, and `analysis.js` remain authoritative.

## Commands

```sh
# Full 2,000-case cross-validation against the JavaScript oracle
npm run lab:oracle-full

# Cross-validate 64 action lists produced by the evolved policy
npm run lab:policy-golden

# High-throughput MAP-Elites attacker; writes out/report.json
cargo run --release --manifest-path balance-lab/Cargo.toml -- attack 20 256 1

# Tiered outer parameter search; writes a report and reviewable JS patch
cargo run --release --manifest-path balance-lab/Cargo.toml -- defend 6 24 48 7

# More than two million paired perk-off/perk-on comparisons (multi-hour)
npm run lab:sensitivity

# Small structural gate used by CI
npm run lab:sensitivity-smoke

# Turn an attacker or defender report into a dependency-free dashboard
cargo run --release --manifest-path balance-lab/Cargo.toml -- dashboard

# Validate the proposed patch without changing the game
npm run lab:apply-params

# Apply an approved patch to index.html
npm run lab:apply-params -- --write

# Multicore throughput check
cargo run --release --manifest-path balance-lab/Cargo.toml -- bench

# Parallel coordinator protocol: JSON jobs on stdin, summaries on stdout
cargo run --release --manifest-path balance-lab/Cargo.toml -- batch < jobs.json

# Fit attacker priors from exported browser sessions
cargo run --release --manifest-path balance-lab/Cargo.toml -- calibrate traces.json

# Seed 25% of the initial attacker population from those priors
cargo run --release --manifest-path balance-lab/Cargo.toml -- attack 20 256 1 balance-lab/out/calibration.json
cargo run --release --manifest-path balance-lab/Cargo.toml -- defend 6 24 48 7 balance-lab/out/calibration.json
```

For a repeatable browser-runtime smoke corpus (not a substitute for human
telemetry), run `npm run lab:capture-traces`, then calibrate
`balance-lab/out/playtraces.json`. The three sessions execute legal strategies
inside Chromium and exercise action recording, reactions, powers, doctrines,
and early calls.

After generating a dashboard, `npm run lab:inspect-dashboard` checks the rendered
archive, opens its top replay in Chromium, advances the simulation, and writes
ignored dashboard/replay screenshots beside the report.

`oracle.js` accepts one case or an array of cases on stdin. A case contains a
map, seed, wave limit, optional parameter overrides, and an action list. It runs
the real code extracted from `index.html` and returns per-wave lives, gold,
kills, elapsed time, and per-tower damage.

The policy genome covers tower priorities, upgrades, branches, merges, all six
fusions, doctrines, timed powerups, and early calls. Placement scores coverage,
corners, center, edges, and support adjacency. Reaction delay and an APM cap keep
the policy from issuing impossible bursts. Interesting elites retain their exact
seed and action list in the report.

The sensitivity command builds a stratified corpus across maps, placement and
tower policies, enemy speed/health/count, tower range, reaction timing, and
doctrine activation immediately before and on rush waves. Each observation is
an exact common-random-number pair: identical seed, parameters, map, and action
queue, with one doctrine forced off versus on. It reports paired deltas and 95%
confidence intervals for survival wave, lives, gold, leaks, leak damage, clear
time, effective and overkill damage, projectile latency, and wasted shots.

After the broad pass, a genetic search targets the largest outcome or mechanical
effect for each doctrine separately, then replays finalists through wave 50.
Projectile-speed results are additionally stratified by homing, splash, enemy
speed, range, map geometry, tower composition, reaction timing, and rush
proximity. The default executes 2,036,352 paired comparisons and 5,009,056 total
simulation runs. Its output is `out/sensitivity-report.json`. Classifications
mean "observed under this finite corpus and adversarial search"; they are not a
proof over every possible state and do not automatically modify or delete a
doctrine.

The defender runs a cheap stat-range screen, wave-25 partial games, and 60-wave
endless finalists. A diagonal CMA-ES searches the continuous constants, tower,
branch, fusion, doctrine, and powerup values. Each generation refines a
warm-started attacker pool against the incumbent, then reruns that attacker for
each full-game Params finalist. Its loss covers
first-loss distribution, exploitability, tower-use entropy, and clear-time curve
roughness.
Before it accepts a result, it runs the candidate through the six criteria in
`sim.js` and the isolation guard in `analysis.js`. A failed candidate is dropped
and the shipped baseline is retained. `out/params.json` is the patch understood
by the JavaScript game; `out/candidate.json` keeps the full Rust value for
reproduction. Applying a patch is deliberately a separate, explicit step.

## Cross-validation status

The full contract is green: 2,000 deterministic cases and more than 32,000 wave
traces match the JS oracle, including non-default parameter patches, per-tower
damage, and action legality. A second CI suite cross-validates 64 action lists
generated by the policy itself; this covers optimizer-only sequences such as
fusion construction. Every attacker elite contains its exact seed and actions.
The dashboard's **watch** link runs that strategy in the browser at the same
1/30 contract timestep.

## What is implemented

- P0: the JSON oracle runs the real core extracted from `index.html`.
- P1: the deterministic Rust twin, Rayon execution, a coordinator-friendly batch
  protocol, structure-of-arrays hot data, reusable arenas, unit tests, and the 2,000-case JS/Rust
  contract are in place.
- P2: the policy models buying, placement, upgrades, branches, merges, fusions,
  doctrines, powerups, early calls, reaction delay, and an APM cap. MAP-Elites
  keeps diverse niches, per-niche CMA-ES refines them, and the CI search
  independently rediscovers a Warhead Silo strategy.
- P3: the defender uses tiered partial/full/endless evaluation, parameter CMA-ES,
  and a warm-started minimax attacker. It emits before/after metrics, validates
  source-of-truth gates, and produces a complete reviewable patch for
  `index.html`.
- P4: attacker and defender dashboards, CI gates, exact seed/action archives,
  and one-click browser replay are working.
- P5 remains stretch work. There is no GPU or cluster coordinator.

The exact core deliberately remains `f64`: an `f32` authoritative path would not
preserve JavaScript's binary64 arithmetic, so it would violate the parity
contract the design calls non-negotiable. Enemy
and projectile hot fields use structure-of-arrays storage; their arenas and
scratch buffers are pre-sized and reused without per-tick allocation.
On the current 48-case benchmark it ran about 764 games/s across 10 Rayon threads
versus 24.2 games/s in the single-process JS oracle: 31.6x wall-clock and 3.2x
per thread. The workload and machine matter, and this is nowhere near the brief's
100–300x-per-core estimate. `out/bench-report.json` records the raw timings and
commit so future changes can be compared without rewriting this paragraph.

No human playtrace corpus is checked into the repo, so the default priors are not
presented as empirically calibrated. A local browser session records actions in
memory; run `copy(exportEpochPlaytrace())` in DevTools to export it. Put one or
more exported objects in a JSON array and pass it to `calibrate`. Nothing is sent
over the network. The resulting file fits reaction delay, APM, action weights,
and support-adjacency preference and can seed the attacker command shown above.
