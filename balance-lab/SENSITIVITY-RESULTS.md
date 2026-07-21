# Doctrine sensitivity results

Status: accepted schema-6 run, 21 July 2026.

## Result

The release gates pass. The run classifies ten doctrines as meaningful and two
as niche-only:

- niche-only: Rapid Logistics and Shock Doctrine;
- meaningful: Napalm Doctrine, Bounty Reform, Standing Army, War Economy,
  Scrap Drive, Iron Curtain, Overcharge Rails, Siege Corps, Fission Ammo, and
  Cold Snap;
- none are redundant, mechanically inactive, or harmful under the broad-corpus
  classification rule.

This is a materially better-balanced build, but it is not a claim that every
perk helps every fixed action queue. Automated power use and discontinuous
target selection still produce adverse corners. Schema 6 searches for those
corners separately and records them instead of allowing a benefit optimizer to
hide them.

## Run identity

| field | value |
|---|---:|
| schema | 6 |
| seed | 20260719 |
| source revision | `98a876e7e6ad854e220e433d4b47b3d86e823445` |
| source dirty at start/end | false / false |
| source stable throughout | true |
| report SHA-256 | `e1ad796173af0a2ab7513e58aafde4c9c3be5954d8a6ef480581875bd9ab1b0a` |
| elapsed | 14,708.019 seconds (4h 5m 8s) |
| threads | 10 |
| logical simulations/second | 688.55 |

The generated JSON is `balance-lab/out/sensitivity-report.json`. It is ignored
because it is a large local artifact; the digest above identifies the accepted
file exactly.

## Scale and gates

| count | value |
|---|---:|
| broad pairs | 1,200,000 |
| targeted pairs | 2,506,752 |
| wave-50 finalist pairs | 2,304 |
| total paired comparisons | 3,709,056 |
| complete paired comparisons | 3,709,056 |
| censored paired comparisons | 0 |
| counterfactual arms | 7,418,112 |
| policy-generation runs | 2,709,056 |
| total logical simulations | 10,127,168 |

Every release check passed:

- more than three million exact pairs;
- at least 99.5% complete pairs (actual: 100%);
- zero pathological-stall pairs;
- common random numbers and exact off/on interventions;
- separate benefit, harm, and mechanical searches;
- populated mechanic-relevant strata for every doctrine.

## What changed in the game

Cold Snap keeps its full 50% chill for 12 seconds of continuous field exposure,
then eases to the normal 40% over the next 12 seconds. This preserves its useful
opening control while preventing a regenerating enemy from remaining in a
slow-field equilibrium indefinitely.

Shock Doctrine now applies its bonus only after an enemy passes halfway along
the path. Its extra displacement has a per-enemy budget that recovers over time.
Boss math was also corrected: the old implementation multiplied 1.5 by 0.3 and
produced three times baseline boss knockback, despite the description saying
twice baseline. The new boss factor is 0.2, so the advertised total is exact.

Rapid Logistics was not weakened. Its availability cannot directly hurt a
player who chooses not to press a power. The negative simulations come from the
fixed automatic policy using an action when the shorter cooldown makes it
available. The report therefore treats Rapid's downside as policy-mediated and
keeps it visible in the harm search.

## Broad-corpus results

Each row contains 100,000 paired scenarios. Outcome-change intervals are Wilson
95% intervals. Utility intervals describe the declared broad generator; they do
not apply to the adaptively selected genetic-search samples.

| doctrine | outcome changed | 95% CI | mean utility | 95% CI | class |
|---|---:|---:|---:|---:|---|
| Napalm Doctrine | 6.609% | 6.457–6.765% | +0.155 | +0.112–+0.197 | meaningful |
| Bounty Reform | 22.766% | 22.507–23.027% | +1.815 | +1.789–+1.841 | meaningful |
| Rapid Logistics | 0.768% | 0.716–0.824% | +0.005 | -0.002–+0.012 | niche-only |
| Standing Army | 22.555% | 22.297–22.815% | +2.191 | +1.963–+2.419 | meaningful |
| War Economy | 5.498% | 5.358–5.641% | +0.019 | +0.019–+0.020 | meaningful |
| Scrap Drive | 20.921% | 20.670–21.174% | +0.305 | +0.290–+0.321 | meaningful |
| Shock Doctrine | 0.874% | 0.818–0.934% | +0.017 | +0.001–+0.033 | niche-only |
| Iron Curtain | 22.784% | 22.525–23.045% | +4.067 | +3.921–+4.212 | meaningful |
| Overcharge Rails | 21.699% | 21.445–21.956% | +3.647 | +3.358–+3.936 | meaningful |
| Siege Corps | 20.003% | 19.756–20.252% | +3.118 | +2.923–+3.313 | meaningful |
| Fission Ammo | 2.379% | 2.286–2.475% | +0.325 | +0.275–+0.376 | meaningful |
| Cold Snap | 1.421% | 1.349–1.496% | +0.649 | +0.580–+0.719 | meaningful |

The broad negative-utility rates range from 0% for War Economy to 12.832% for
Standing Army. A negative paired result does not by itself make a perk harmful;
the harmful classification requires the upper 95% confidence bound on broad
mean utility to be below -0.1. No doctrine meets that condition.

## Relevant-context results

Corpus-wide inactivity can dilute a perk that requires a particular tower or
action. Schema 6 therefore reports a mechanic-relevant stratum for every perk.
Selected rows:

| doctrine | relevant pairs | outcome changed | mechanical change | mean utility |
|---|---:|---:|---:|---:|
| Rapid Logistics | 17,058 | 4.502% | 10.406% | +0.030 |
| Shock Doctrine | 8,573 | 10.195% | 15.864% | +0.195 |
| Overcharge Rails | 35,099 | 61.822% | 65.480% | +10.391 |
| Fission Ammo | 2,595 | 91.676% | 96.069% | +12.534 |
| Cold Snap | 1,465 | 96.997% | 97.133% | +44.317 |

This is why Overcharge Rails is not redundant even though base projectiles are
fast. It changes outcomes in 61.8% of policies that actually use projectile
towers.

## Projectile-speed interactions

Overcharge Rails was stratified across homing, splash, enemy speed, tower range,
map, activation timing, rush timing, composition, focus tower, reaction delay,
and sell strategy.

- Homing present: 2,390 pairs, 99.791% outcome change, mean utility +19.977.
- Splash present: 24,640 pairs, 82.273% outcome change, mean utility +13.251.
- Splash absent: 75,360 pairs, 1.894% outcome change, mean utility +0.507.
- Broad compositions: 11,776 pairs, 97.274% outcome change, mean utility
  +17.247.
- Range factor 0.75: 15.637% outcome change, mean utility +1.445.
- Range factor 1.30: 26.847% outcome change, mean utility +5.489.
- Map outcome-change rates are 24.423%, 23.418%, and 17.256% for maps 0, 1,
  and 2 respectively.

Enemy-speed strata all remain active: their outcome-change rates range from
21.286% to 22.094%. The important interactions are homing, splash, composition,
range, and geometry—not simply whether enemies are faster.

## Adversarial tails

The genetic searches maximize three different objectives per doctrine. Each
objective uses 34 generations of 2,048 policies, followed by 64 independent-seed
wave-50 finalists. These selected samples are descriptive, not confidence
intervals.

| doctrine | best benefit finalist | best harm finalist | interpretation |
|---|---:|---:|---|
| Rapid Logistics | +25 survival waves, +17 lives | -25 waves, -17 lives | shorter cooldown changes which scheduled powers execute |
| Shock Doctrine | +8 waves | -8 waves | niche defensive effect; deterministic targeting can still diverge |
| Overcharge Rails | +25 waves, +14 lives | -24 waves, -2 lives | highly active, with both strong benefit and targeting-dependent harm |
| Cold Snap | +9 waves, +6 lives | -5 waves, -2 lives | useful field control without the former unbounded stall |

The mechanical objective can find a worse outcome than the harm objective
because it is optimizing telemetry magnitude, not utility. Shock's selected
mechanical finalist loses 18 waves, for example. This is retained as an explicit
tail, not used as broad-population evidence.

## Regression replays

The exact old Cold Snap failure used the same scenario on both sides. Before the
fix, the perk-on arm took 82,669.3 simulated seconds and 2,480,079 ticks, versus
6,889.8 seconds and 206,695 ticks off, with no survival, lives, leaks, or gold
improvement. With the new fatigue rule, the same scenario completes in 2,112.0
seconds and 63,360 ticks on, saves 4 lives, and leaks 4 fewer enemies. The
pathological-stall count across the accepted 3.7-million-pair run is zero.

The exact old Shock Doctrine failure fell from wave 47 off to wave 25 on. With
the halfway-path and bonus-budget rules, both arms reach wave 47 in that replay;
the perk-on arm has a small positive utility delta.

## Method and limits

Every observation is perk-on minus perk-off under the same seed, map,
parameters, policy genome, placement/action queue, and pre-intervention state.
The broad corpus varies all three maps, five enemy-speed factors, five tower
range factors, three health factors, three count factors, five activation waves,
ten focus towers, placements, branches, merges, fusions, power timing, early
calls, reaction delay, APM, and sell timing/target.

Metrics include survival wave, lives, gold, leaks, leak damage, clear time,
effective damage, overkill, projectile latency, wasted shots, damage per combat
second, overkill per kill, wasted-shot rate, knockback per combat second, slow
coverage, and power uses. Raw totals and normalized rates are both retained.

The simulator is deterministic, so common-random-number pairing removes seed
noise but does not remove policy discontinuities. A one-tick targeting change
can send two arms down different trajectories. Broad confidence intervals are
valid only for the declared generator and parameter ranges. Genetic-search and
finalist intervals are not inferential because selection is adaptive. This is a
large finite corpus, not a proof over every possible game state.
