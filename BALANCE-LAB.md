# EPOCH Balance Lab — design brief

A proposal for turning balance from hand-tuning into search: a fast deterministic
core, a realistic parameterized player, and two coupled optimizers that evolve
players to break the game and evolve the game to resist. Written to be handed to
another agent to build. Nothing here is built yet.

## Why

Today `sim.js` runs about seven hand-scripted builds on one core and checks six
pass/fail rules. That catches gross breakage. It does not catch what a player
will actually do. The "Warhead Silo is OP" report is the tell: a human found a
degenerate strategy the sim never tried, because the sim only plays the builds I
thought to write. Balance is also tuned by eye right now — change a constant,
rerun, squint at the numbers.

The goal is to make balance an optimization problem with two searches that feed
each other:

1. **Attacker** — search the space of *player strategies* for the strongest or
   most degenerate way to play. If one strategy dominates every other, the game
   is unbalanced, and the attacker's job is to prove it.
2. **Defender** — search the space of *game constants* (tower stats, wave
   formulas, fusion and doctrine values) so the design targets hold against the
   best attacker.

Run them against each other and you get a minimax loop: evolve players to break
the game, evolve the game to survive the players, repeat until no single
strategy runs away and the difficulty curve matches intent.

## Throughput, measured

Measured on this repo: `node sim.js` runs about six full-game-equivalents (four
scripted builds plus a placement bot on three maps) in 1.6 seconds on one core,
so roughly 4 full games per second per core, including the per-build engine boot.
From there:

- 2 billion full games in JS: about 5,000 core-days, or 41 days pinned on 128
  cores. Not happening.
- The same engine rewritten in Rust (structure-of-arrays over enemies and
  projectiles, f32 SIMD, no garbage collector, seeded PRNG) is a conservative
  100–300× per core, so several hundred to a couple thousand games per second per
  core. 2 billion full games drops to tens of core-days, which is hours on a
  128-core box or a small cluster.

But you almost never need 2 billion *full* games. A tiered evaluation (below)
spends most of the budget on cheap partial games and saves full runs for
finalists, cutting effective cost by one to two orders of magnitude. The honest
target is not a round number of games. It is a core fast enough that full games
are cheap, plus an optimizer that spends compute where it changes the answer.

## Architecture

### 1. Core sim — `epoch-core` (Rust)

A faithful, deterministic port of `update(state, dt)` and the DATA tables.

- **Deterministic.** Seeded xoshiro256++, fixed 1/30 dt, fixed operation order.
  Same seed plus same params plus same policy gives the same result, bit for bit,
  on any machine. The JS loop already avoids `Date.now`/`Math.random`, so this
  maps cleanly.
- **SoA hot data.** Enemy and projectile fields live in parallel arrays so the
  per-tick loops vectorize and stay in cache. Towers are few (<80), so
  array-of-structs is fine for them.
- **No allocation in the loop.** Pre-sized arenas and free lists for enemies and
  projectiles.
- **Params as a struct.** Every tunable — the formula constants, tower stats,
  branch/fusion/doctrine tables — loads at start into one `Params` value, so the
  optimizer sweeps parameters without recompiling.

### 2. Golden cross-validation — the anti-drift contract

The Rust core has to match the JS game exactly, or the lab optimizes a different
game than the one people play. This is the single most important rail.

- Add a trace protocol to the JS engine: given `(seed, params, action-list)`,
  dump per-wave `{lives, gold, kills, per-tower damage, end wave}` as JSON.
- Rust runs the same inputs and must match within epsilon (or exactly, with
  fixed-point).
- A CI job runs a few thousand golden cases both ways and fails on any
  divergence. `sim.js` and `analysis.js` stay the source of truth; the Rust core
  is the fast twin, never the authority.

### 3. Player policy — realistic, parameterized

The attacker must play like a person, not run a fixed script. Model a policy as a
decision function evaluated at wave starts and gold thresholds, encoded as a
fixed-length float vector so optimizers can operate on it directly.

- **Build genome.** Weighted priorities over actions: which tower to buy, whether
  to upgrade vs. buy vs. merge vs. fuse, which branch, which doctrine to draft,
  when to spend powerups, when to early-call.
- **Placement.** Each cell scored by evolved weights over features: path length
  within range, chokepoint coverage, adjacency to support towers for infusions
  and fusions, corner control. This is the "realistic placement" part.
- **Timing.** Actions gated on affordability plus a reaction-delay model, with an
  optional actions-per-minute cap so "optimal" is not superhuman.

Calibrate the placement and timing priors against real play if possible: log a
handful of human games from the browser build and fit the priors to them.

### 4. The two optimizers

**Attacker.** For fixed Params, evolve the policy to maximize a dominance
objective (max wave reached, min lives lost, min gold, or "clears 50 using the
fewest distinct towers" to surface one-tower cheese). Use **MAP-Elites** as the
centerpiece: it keeps a diverse archive of strong strategies across niches ("best
build using only towers up to age 5", "best that never merges", "best mono-tower")
rather than a single winner. The variety is the point — it answers "is anything
dominant?" directly. CMA-ES refines the continuous weights inside promising cells.

**Defender.** Outer loop over Params. The balance loss is a scalar over design
targets:

- First-loss-wave distribution for a population of mediocre policies peaks in the
  casual band (waves 28–34).
- Exploitability is bounded: the gap between the best attacker and the median
  policy stays small.
- Diversity: many MAP-Elites cells stay viable (entropy of winning tower usage
  above a threshold).
- Clear-time curve stays smooth, no spikes.
- The existing six criteria as hard constraints.

Use CMA-ES or Bayesian optimization over the constants; the attacker is re-run,
warm-started, for each Params candidate. That nesting is the minimax loop.

**Tiered evaluation** keeps it affordable:

- Tier 0 (microseconds): the isolation-arena DPS analysis (`analysis.js`, already
  built) as a surrogate that rejects obviously broken Params before any game.
- Tier 1: partial games (to wave 25, or reduced counts) for the inner loop.
- Tier 2: full 50-wave and endless games for finalists and final validation.
- Successive halving / Hyperband to pour games into promising candidates and kill
  weak ones early.

### 5. Scale and orchestration

- Expose the core as a Rayon thread-pool library (one machine, all cores) and as
  a batch CLI (params + genome in, summary stats out) so a coordinator can fan
  jobs across machines. Each job is embarrassingly parallel.
- Determinism plus seed logging means any interesting result — a broken strategy,
  a failed target — replays exactly in the browser game for a human to watch. The
  lab finds it, you press play.
- A GPU lockstep path (one warp per game) is possible later because the per-tick
  update is SIMD-friendly, but the CPU version should already hit the target.
  List it as a stretch, not phase one.

### 6. Outputs

Static HTML and SVG, matching the game's zero-dependency habit, or a notebook for
exploration.

- Throughput: games/sec, core-hours per run, tracked across commits.
- First-loss-wave histograms per policy tier.
- Per-wave clear-time curve with a variance band.
- Tower/branch/fusion pick rates and win contribution from per-tower damage.
- A MAP-Elites heatmap showing which strategy niches are viable.
- Exploitability over optimizer generations, which should trend down.
- A before/after diff for a Params change.

Wire the key scalars into CI as regression gates, the same spirit as today's six
criteria but distributional instead of pass/fail.

## The dev-cycle loop this enables

Change the game, and the attacker searches for the new degenerate strategy, the
defender retunes constants to hit targets against it, the golden test confirms
Rust still equals JS, the dashboards say whether balance improved or regressed,
and any flagged strategy is one click from playing in the browser. Every change
is judged by search instead of by eye.

## Phased plan

Each phase is useful on its own.

- **P0 — Oracle protocol (JS, ~1 day).** JSON trace dump from the engine. The
  contract everything else trusts. Ship before any Rust.
- **P1 — Rust core + golden test (~1 week).** Port DATA and `update()`, pass
  cross-validation on a few thousand seeds, benchmark. Worth it alone: the
  existing six criteria run 100–300× faster.
- **P2 — Policy + attacker (~1 week).** MAP-Elites plus CMA-ES. Validate by having
  it independently rediscover the Warhead-Silo-class cheese.
- **P3 — Defender loop + tiered eval (~1–2 weeks).** Auto-tune the constants to
  targets and regenerate the DATA tables the game ships, closing back to the
  existing "tables regenerate from formulas" doctrine.
- **P4 — Dashboards + CI gates (~few days).** Graphs, regression tracking,
  browser replay of flagged runs.
- **P5 — GPU/cluster (stretch).** Only if you want literal billions per hour.

## Caveats worth stating

- **The oracle is everything.** If Rust drifts from JS, you optimize a phantom.
  Golden tests are non-negotiable and rerun on every core change.
- **Optimizing to a metric warps the game to the metric.** Encode "no dominant
  strategy, right difficulty curve, real diversity" carefully or the optimizer
  finds a balanced-but-boring local optimum. Keep a human reading the MAP-Elites
  archive, not just the scalar loss.
- **Realistic play is a modeling choice.** The policy's placement and timing
  priors decide what "balanced for players" even means. Calibrate against human
  playtraces where you can.
- **You probably never need 2 billion full games.** Tiers and bandits get the same
  signal for far less. Chase throughput because cheap games let you search wider,
  not to hit a round number.

## Recommended stack

- Core: Rust. Rayon for multicore, f32 SoA, xoshiro PRNG. Reach for intrinsics or
  assembly only if profiling a specific inner loop demands it; hand-written asm up
  front costs iteration speed for little over autovectorized Rust.
- Optimizer: Rust (`argmin`, or a small hand-rolled CMA-ES and MAP-Elites) to stay
  in one language, or Python (`cma`, `ribs`) driving the Rust batch CLI if you
  want that ecosystem for analysis and plotting.
- Dashboards: static HTML/SVG, or matplotlib in a notebook.
- Keep `index.html`, `sim.js`, and `analysis.js` as the human-facing game and the
  oracle. The lab is a satellite, not a rewrite.
