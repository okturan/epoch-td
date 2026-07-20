# Doctrine sensitivity results

Status: accepted schema-5 run, 20 July 2026.

This report measures every doctrine as an exact paired intervention. The same
generated scenario and scheduled policy/action queue start from a shared
pre-intervention state, with the doctrine forced off and forced on. Later
actions, purchases, and placements can be accepted differently after the arms'
gold and game state diverge. This is evidence about the represented corpus, not
a claim about every possible game state, and it does not change any shipped
balance value.

## Run record

| Item | Value |
|---|---:|
| Paired off/on comparisons | 2,036,352 |
| Complete pairs used for ordinary effects | 2,034,642 |
| Right-censored pairs | 1,710 (0.08397%) |
| Broad pairs | 1,200,000 |
| Genetic-search pairs | 835,584 |
| Full-horizon finalist pairs | 768 |
| Counterfactual arms | 4,072,704 |
| Policy-generation runs | 1,036,352 |
| Logical simulation outputs | 5,109,056 |
| Workers | 10 |
| Elapsed time | 3,531.068 seconds (58m 51s) |
| Logical throughput | 1,446.887 outputs/second |

The source tree was clean and stayed on
`7d5e7787650fa65e2f29a24496cc38c19422ba1b` from start to finish. The raw
77,156,044-byte report has SHA-256
`89e02a0b78d645aa7a39bb4028eccb9f94d2ca5b2577d675183ce639de7c7521`.
The release executable used locally has SHA-256
`905101ce4256ef4a67681e37f058d1f444594e6546e0f7947ee3978748a16a41`.
The executable digest and native-control timing are local audit records, not
fields cryptographically embedded in the JSON.

The official checker passed. A separate structural audit traversed 2,019,337
numeric leaves and found no non-finite values, count-closure failures,
classification mismatch, or capped pair leaking into ordinary effect evidence.
It recomputed the Wilson and tick-cap intervals; ordinary mean intervals are
finite and contain their means. All retained maximum-effect examples are
complete and below the 3,000,000-tick arm limit.

## Classification

The broad phase contains 100,000 pairs per doctrine. Conditional on both arms
completing, its intervals describe uncertainty under the declared scenario
generator and represented parameter ranges. The targeted and finalist phases
were selected adaptively, so their intervals are descriptive rather than
population estimates.

`Outcome changed` is the complete-pair rate with any survival-wave, lives, gold,
leaks, leak-damage, or clear-time delta above the 1e-9 numerical epsilon.
Utility weights those outcomes plus wasted projectiles on the lab's fixed
scale; cumulative damage, overkill, and projectile latency do not enter utility.
The per-pair direction counts call utility above +0.1 positive and below -0.1
negative.

| Doctrine | Broad outcome changed, 95% CI | Broad utility, 95% CI | Classification | Practical reading |
|---|---:|---:|---|---|
| Napalm Doctrine | 6.611% [6.459, 6.767] | +0.153 [+0.110, +0.195] | meaningful | Positive mean broad utility. |
| Bounty Reform | 22.765% [22.506, 23.026] | +1.815 [+1.789, +1.841] | meaningful | Strongly positive economy effect on average. |
| Rapid Logistics | 0.771% [0.719, 0.827] | +0.0047 [-0.0025, +0.0120] | niche-only | Rarely active broadly; predominantly harmful where search makes it matter most. |
| Standing Army | 22.556% [22.298, 22.816] | +2.186 [+1.958, +2.414] | meaningful | Broad harm is more frequent than benefit, but the benefits are larger; all full finalists are positive. |
| War Economy | 5.498% [5.358, 5.641] | +0.0193 [+0.0187, +0.0200] | meaningful | Economy-only in the broad corpus; no direct mechanical change. |
| Scrap Drive | 20.921% [20.670, 21.174] | +0.305 [+0.290, +0.321] | meaningful | Positive mean broad utility. |
| Shock Doctrine | 3.743% [3.627, 3.862] | +0.039 [+0.015, +0.063] | meaningful | Positive broadly, but the strongest finalist contexts skew harmful. |
| Iron Curtain | 22.783% [22.524, 23.044] | +4.074 [+3.928, +4.220] | meaningful | Positive mean broad utility. |
| Overcharge Rails | 21.695% [21.440, 21.951] | +3.639 [+3.352, +3.926] | meaningful | Clearly active and useful; highly strategy-dependent. |
| Siege Corps | 20.003% [19.756, 20.252] | +3.120 [+2.924, +3.315] | meaningful | Positive mean broad utility. |
| Fission Ammo | 2.386% [2.293, 2.483] | +0.325 [+0.274, +0.376] | meaningful | Lower-frequency, positive-on-average effect. |
| Cold Snap | 1.420% [1.349, 1.495] | +0.640 [+0.569, +0.710] | meaningful | Positive broadly, but maximized contexts expose severe simulated stalls and harm. |

The formal result is eleven `meaningful` doctrines and one `niche-only`
doctrine. None is broadly classified `redundant`, `mechanically inactive`, or
`harmful`. “Meaningful” does not mean uniformly beneficial: the paired search
found important harmful regions for Rapid Logistics, Shock Doctrine, and Cold
Snap.

The formal `harmful` rule is broad-only: the upper 95% bound on mean utility
must be below -0.1. A `meaningful` label is therefore not a safety guarantee
against targeted or strategy-specific harm. Standing Army illustrates the
difference: broad threshold-negative cases outnumber positive cases, 12.841%
to 8.673%, but the benefits are larger, leaving positive mean utility; all 64
full finalists are positive.

## Overcharge Rails

Overcharge Rails multiplies projectile velocity by 1.3. Ordinary projectiles
start at speed 12, while homing projectiles start at speed 5 before level and
branch modifiers. The intuition that many ordinary shots are already fast is
partly right, but it does not make the doctrine redundant.

| Phase | Complete / all | Outcome changed | Utility | Mean impact-latency delta |
|---|---:|---:|---:|---:|
| Broad | 99,996 / 100,000 | 21.695% | +3.639 | -7.024 ms |
| Targeted | 69,621 / 69,632 | 85.316% | +161.35 | -36.880 ms |
| Full finalist | 64 / 64 | 100.000% | +2,037.18 | -47.380 ms |

The broad latency interval is -7.111 to -6.936 ms. This is a more direct mechanic
measure than cumulative totals, but it is the difference between each arm's
mean impact latency and the arms can contain different impact mixes. Broad
cumulative damage, overkill, and wasted-shot totals can grow simply because one
arm survives longer.

Empty policies account for 64,901 broad cases and are all exact zero, as they
should be. Conditional on policies classified non-empty by their recorded tower
counts, the outcome-change rate is 61.815% [61.306, 62.322]. Even non-empty
policies without splash change outcome in 13.644% [12.999, 14.315] of cases,
with utility +3.653 and mean latency -4.028 ms. That is lower-frequency than
mixed projectile strategies, but well above redundancy.

The following rates are conditional on non-empty policies:

| Dimension | Observed broad outcome-change rates |
|---|---|
| Range multiplier | 44.74% at 0.75x, rising to 76.53% at 1.30x |
| Reaction policy | 79.65% fast, 71.63% medium, 49.95% slow |
| Activation timing | 64.12% on-rush, 58.34% immediately before |
| Map | 69.59% map 0, 66.17% map 1, 49.55% map 2 |
| Enemy speed | 60.37-62.94% across 0.65x-1.40x, with no monotonic trend |

Composition is the largest observed separator. Non-homing plus splash changes
outcome in 80.381% of its cases; homing plus splash changes 99.791%. Those are
conditional descriptions, not isolated causal estimates. Every homing case in
this corpus also has splash, and both features are strongly confounded with
mixed tower compositions. Within focused mixes, no-splash and splash rates are
similar (81.31% and 83.80%); within mono compositions, the observed rate rises
from 7.03% to 14.12% with splash.

The highest-fitness complete finalist gained 25 survival waves and 20 lives,
prevented 20 leaks, and reduced per-impact latency by 56.7 ms. Its extra 25
waves also added 1,125 seconds, 1.53 million damage, 109,824 overkill, and 2,313
wasted shots. Those cumulative changes are exposure-confounded, not evidence
that faster projectiles became less efficient.

The benefit is not uniform: five of 64 full finalists are negative, and the
minimum full utility is -2,789.94. The report does not retain the
minimum-utility context, so that downside tail should be reproduced and
inspected before a buff.

Conclusion: keep Overcharge Rails at its current strength pending downside-tail
inspection. It is often irrelevant to empty or simple mono-tower play, but
meaningful in active, mixed, long-range defenses. Observationally in the
represented broad corpus, range, composition, reaction speed, and timing are
larger separators than enemy speed.

## Harmful and pathological search regions

The broad classification is the primary label. The genetic search answers a
different question: “where can this doctrine have the largest effect?” Its
fitness is 1,000 times absolute outcome score plus absolute mechanical score;
it does not directly minimize utility or search for harm. These are discovered
downside regions, not an exhaustive worst-harm bound, and absence from this
section does not prove safety. The search exposed three clearest phase-level
balance risks.

### Rapid Logistics

Rapid Logistics is the only `niche-only` doctrine. The targeted phase changed
outcome in 56.46% of completed pairs, but mean utility was -7.209. In the 64
full-horizon finalists, all outcomes changed, 58 were negative, six were
positive, and mean utility was -978.593. The highest-fitness finalist lost 25
waves and 20 lives. This is not evidence of a healthy beneficial niche; it is a
rare mechanic whose most active regions are usually harmful.

The formal `niche-only` boundary is also met: its broad outcome upper bound is
0.827%, below 1%, while an eligible adversarial outcome-effect score exceeds
the activity threshold.

Recommendation: redesign or constrain it before considering a numerical buff.

### Shock Doctrine

Shock Doctrine is modestly positive in the broad corpus. Among the 64 selected
full-horizon finalists, however, 36 were negative and 28 positive. Mean utility
was -172.858 [-314.752, -30.964], and survival fell by 1.844 waves on average.
Its highest-fitness complete example lost 22 waves.

Recommendation: inspect the contexts retained in the report before shipping a
balance change; the average broad benefit masks an in-corpus downside region.

### Cold Snap

Cold Snap is positive but low-frequency in the broad corpus. Its targeted phase
has mean complete-pair utility -23.590, and 1,683 of 69,632 pairs are censored;
1,094 are on-only caps, seven are off-only, and 582 are both-capped.
Complete-pair targeted estimates are therefore conditional on avoiding that
excluded stalled region. Broad censoring is only five in 100,000, so the formal
broad label remains stable.

All 64 full finalists are negative, with mean utility -1,182.764. The
highest-fitness complete example adds 75,779 seconds—about 21 simulated
hours—without improving survival, lives, leaks, or gold. It remains below the
tick cap, so this is an uncapped simulated pathological stall rather than a
censored artifact. Across full finalists, Cold Snap improves survival by 0.719
waves and lives by 11.969 on average, but adds
27,640 seconds [24,167, 31,113]; the stall cost dominates and makes all 64
utilities negative. The maximum example also adds 16.825 million cumulative
damage, but schema 5 correctly gives cumulative damage no utility reward,
leaving utility -3,786.67.

Recommendation: add a stall guard or redesign the slow interaction before
tuning its headline strength.

Fission Ammo provides a useful contrast. It has only four broad and three
targeted both-arm caps, no cap transition, positive mean utility in all three
phases, and 63 positive versus one negative full finalist. It is a
lower-frequency, positive-on-average effect without Cold Snap's cap-transition
pathology; it is still not uniformly beneficial.

## Censoring and deterministic replay

A capped arm is not treated as a completed game. Any pair containing one is
excluded from ordinary metrics, intervals, utility, maxima, and examples. In
genetic search a censored candidate receives fixed fitness -1, below every
eligible candidate; cap direction and magnitude are not optimized. The 1,710
censored pairs close exactly as 600 both-capped, 13 off-only, and 1,097 on-only
pairs. Full finalists are all uncapped.

Each cell below is `complete / censored`:

| Doctrine | Broad | Targeted | Full finalist |
|---|---:|---:|---:|
| Napalm Doctrine | 100,000 / 0 | 69,632 / 0 | 64 / 0 |
| Bounty Reform | 100,000 / 0 | 69,632 / 0 | 64 / 0 |
| Rapid Logistics | 100,000 / 0 | 69,632 / 0 | 64 / 0 |
| Standing Army | 100,000 / 0 | 69,632 / 0 | 64 / 0 |
| War Economy | 100,000 / 0 | 69,632 / 0 | 64 / 0 |
| Scrap Drive | 100,000 / 0 | 69,632 / 0 | 64 / 0 |
| Shock Doctrine | 100,000 / 0 | 69,632 / 0 | 64 / 0 |
| Iron Curtain | 100,000 / 0 | 69,632 / 0 | 64 / 0 |
| Overcharge Rails | 99,996 / 4 | 69,621 / 11 | 64 / 0 |
| Siege Corps | 100,000 / 0 | 69,632 / 0 | 64 / 0 |
| Fission Ammo | 99,996 / 4 | 69,629 / 3 | 64 / 0 |
| Cold Snap | 99,995 / 5 | 67,949 / 1,683 | 64 / 0 |

The game core is deterministic for fixed inputs. The top-level seed drives
corpus and genome generation; a scenario seed can also deterministically choose
a fallback relic when scheduled actions leave a pick unresolved, but it is not
a stream of in-run stochastic noise. Both paired arms receive the same seed, so
the only forced difference is the doctrine intervention. Each doctrine's 64
`fullWaveValidation` records are selected deterministic finalist replays, not
independent stochastic seed holdouts. They extend nine wave-25 targeted screens
to wave 50. Overcharge Rails, Fission Ammo, and Cold Snap already target wave 50,
so their full phase is a same-horizon replay with a different deterministic
fallback seed. It adds no longer horizon, and its adaptively selected interval
remains descriptive rather than an independent population-validation estimate.

The 12-way Overcharge interaction table is exploratory. It contains 15,606
occupied cells with median sample size three; 11,564 cells have five or fewer
observations. The one-dimensional marginals above are well populated, while
individual high-dimensional cells should be used to generate hypotheses, not
precise effect estimates.

## Performance assessment

The accepted run uses the faster of the two exact CPU configurations compared in
a small control for this report. The accepted binary uses default arm64 code
generation; it has no PGO, explicit NEON/SIMD kernels, ISA dispatch, or GPU
offload.

- Rust release `opt-level=3`, thin LTO, one codegen unit, and abort-on-panic;
- ten Rayon workers, which kept the machine near ten fully occupied cores;
- structure-of-arrays enemy and projectile storage;
- pre-sized, reusable arenas and scratch buffers;
- one shared pre-intervention state per policy/scenario group, forked for
  counterfactual arms and reused across perk batches;
- reuse of the natural policy arm when it exactly matches an intervention.

A host-tuned `target-cpu=native` build, for which LLVM selected `apple-m4` on
this M5 host, produced identical simulation and effect results but measured
about 5.9% slower in one paired control comparison. It was not used for the
accepted run. Hand-written SIMD is not an automatic win for the branch-heavy,
variable-length world update. A straightforward Metal port would not preserve
the current JavaScript-binary64 parity contract, so GPU acceleration requires a
separate numerical backend. Distributed CPU execution is a different project:
it requires deterministic sharding and aggregation, especially around the
adaptive genetic search.

The next credible speed work is profile-led CPU work—such as removing repeated
ID lookups and cloning less mechanic-irrelevant state—followed by another full
oracle/parity campaign. GPU offload is not a compiler flag. The reported
1,446.887 outputs/second is logical-output throughput under prefix and baseline
reuse, not 1,446 independent full-horizon arms per second.

The practical conclusion is that this run is heavily optimized and complete at
the requested millions scale, but it is not the theoretical maximum possible
throughput. Claims of “best-in-class GPU/ISA” would be false for the current
code.
