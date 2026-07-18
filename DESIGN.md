# EPOCH — Design Spec (source of truth)

Every gameplay value lives in this file. `index.html`'s DATA section mirrors it 1:1; any change here is applied there and re-verified by `sim.js` (all 5 criteria must PASS). Wave table and palettes below are generated from the engine by `gendesign.js`.

## Board
Grid 14×9, 70px cells. Path waypoints (0,1)→(12,1)→(12,3)→(1,3)→(1,5)→(12,5)→(12,7)→(0,7): 53 cells, walk length 52. Towers on any free non-path cell.

## Maps (levels — chosen on the start screen)
| # | name | path length | notes |
|---|---|---|---|
| 1 | Serpent | 52 | Normal. The balance-verified reference map |
| 2 | Spiral | 51 | inward coil; deep central tower pockets |
| 3 | Gauntlet | 31 | Hard — ~40% less exposure time |

All balance criteria are verified on Serpent; criterion 6 checks every map is playable by a naive auto-placement bot to wave ≥18.

## Rules & economy
- Start 80g, 20 lives. Leak −1 life, boss −5. Gold from kills only (no interest).
- Waves: manual start or auto 3s after clear. Speed 1×/3×. Spawn gap 0.8s (rush 0.28s).
- Early call: starting the next wave while one is still alive pays +round(2+0.5w)g (max one extra wave active).
- Endless Mode: offered on victory — waves continue past 50, archetype cycle repeats with compounding formulas, until defeat.
- Sell: refund 70% of total invested.
- Upgrade: levels 1–3, each costs 60% of base; per level dmg ×1.5, range ×1.06, projectile speed ×1.10.
- Branch: at L3 pick one of two (permanent), costs 100% of base.
- Merge: two towers, identical type+level+branch, 8-adjacent → survivor gains +1 level (L4/L5 are merge-only), invested gold sums.
- Age = ⌊cleared/5⌋ (Stone…Space); tower i unlocks at age i. Palette+music shift on age-up.

## Formulas — constants C = {hp:14, g:1.12, cnt:0.8, spd:0.008, bty:0.38}
- HP(w)=14·1.12^(w−1) · count(w)=8+⌊0.8w⌋ · spd(w)=1+0.008w cells/s · bounty(w)=max(1,round(1+0.38w))
- Rush (w=5,15,25,35,45): hp×0.55, count×2, bounty×0.5 (rounded, min 1)
- Boss (w=10,20,30,40,50; i=1…5): hp=HP(w)×26, n=1, spd 0.55, bounty 20+10i, armor [0,4,6,8,10][i], regen 0.2%·hp (i≥3), hit cap 2%·hp (i≥4)

## Enemy archetypes (block of 5 waves = [new, prev, new, new, rush|boss])
| archetype | modifiers |
|---|---|
| Grunt | — |
| Runner | hp ×0.6, spd ×1.7 |
| Armored | armor 3 (w<13) else 4 |
| Regen | regen 1.2% hp/s |
| Splitter | hp ×0.75; on death 2 children @40% hp at d−0.6/−1.2, bounty 1, no re-split |
| Shielded | hit cap 5% hp |
| Elite | armor 7 (w<37) else 8, regen 0.8% |
| Tank | hp ×2.2, spd ×0.55 |
| Dasher | spd ×1.05; dash ×2.4 for 0.6s every 2.5s |
| Void | hp ×1.04, spd ×1.05, armor 10 (w<47) else 11, regen 0.8%, dash |

## Towers (dmg/rate/range at L0; branches at L3, cost = base)
| # | age | name | cost | dmg | rate/s | rng | mechanic | b1 | b2 |
|---|---|---|---|---|---|---|---|---|---|
| 0 | Stone | Rock Hurler | 20 | 6 | 1.0 | 2.5 | single-target projectile | Boulders: dmg×2 | Skip Shot: splash .5, dmg×.7, proj spd×1.4 |
| 1 | Bronze | Catapult | 45 | 13 | 0.5 | 3.0 | splash .9 | Siege Load: dmg×1.8 | Wide Net: splash 1.4 |
| 2 | Iron | Ballista | 70 | 22 | 0.7 | 4.5 | armor pierce | Overdraw: dmg×1.8 | Twin Bolts: rate×2, dmg×.7 |
| 3 | Medieval | Brazier | 100 | 3 | 2.0 | 2.0 | burn: 4/s·3s per stack, ×5 stacks, power scales with dmg mult | Inferno: burn×2 | Pyre: rng×1.5 |
| 4 | Gunpowder | Cannon | 140 | 55 | 0.35 | 3.0 | splash .5 + knockback 0.7 cells (boss ×0.15) | Doom Shell: dmg×1.8 | Concussive: knock 1.4, splash .8 |
| 5 | Industrial | Gatling | 190 | 8 | 3→9 | 2.5 | ramp +1.0/s while locked, resets on switch | Hot Barrels: ramp cap 12 (→15/s) | AP Rounds: pierce, dmg×.75 |
| 6 | Modern | Missile | 250 | 105 | 0.6 | 5.5 | homing v5; targets highest (hp − in-flight dmg); splash .7 | Warheads: dmg×1.8 | MIRV: splash 1.3, dmg×.75 |
| 7 | Atomic | Reactor | 330 | 12/s | aura | 2.2 | constant aura + Irradiated: +20% dmg taken, 0.5s linger | Meltdown: dmg×2 | Wide Field: rng×1.45 |
| 8 | Information | Drone Hub | 420 | — | field | 2.5 | slow ×0.6 in range; +25%·1.5^lvl fire rate to 8 neighbors | Overclock: buff base 50% | Stasis: slow ×0.45 |
| 9 | Space | Orbital Laser | 600 | 40/s | beam | ∞ | beam anywhere; +40/s per second on same target; sticky highest-max-hp | Overcharge: dmg×1.5 | Rapid Focus: ramp×2 |

## Doctrines (roguelite draft)
After every boss wave clears (10/20/30/40, and every boss in Endless), the game pauses and offers a choice of one from three doctrines — permanent run modifiers, each a single field the loop reads. Offers are deterministic: seed = wave·7 + map·3, skipping owned; the sim's reference builds always take slot 0, so all criteria are verified with doctrines in play.
| doctrine | effect |
|---|---|
| Napalm Doctrine | burn stacks to 7 (was 5) |
| Bounty Reform | +1 gold on every kill |
| Rapid Logistics | powerup cooldowns −30% |
| Standing Army | all towers +8% range |
| War Economy | selling refunds 85% (was 70%) |
| Scrap Drive | upgrades cost 50% of base (was 60%) |
| Shock Doctrine | knockback +50%, ×2 on bosses |
| Iron Curtain | +3 lives immediately |
| Overcharge Rails | projectiles +30% speed |
| Siege Corps | splash radius +20% |
| Fission Ammo | irradiation lingers 1.0s (was 0.5s) |
| Cold Snap | slow fields 50% (was 40%) |

## Secret fusions (hidden in-game — discovered through adjacency)
Two adjacent towers matching a recipe (both at/above the minimum level) can fuse — via the gold card row or by dragging one onto the other. The survivor takes the hybrid identity (base type swaps where noted), keeps the higher level, combines invested gold, gains the +10% merge bonus, and its specialization is replaced by the fusion overlay. The in-game help only hints ("rumors…"); discovery is the reward. Fusion multipliers are tuned by analysis.js, an isolation-arena harness that measures every tower, branch, and fusion effective DPS per cell and per gold, with a balance guard that fails if any fusion becomes a runaway (>1.85x its base best branch) or a trap (<0.95x its base).
| recipe | min lvl | result | overlay |
|---|---|---|---|
| Catapult + Brazier | 2 | ✦ Meteor Thrower (catapult) | dmg ×2 · splash 1.4 · projectiles apply burn ×2 |
| Ballista + Cannon | 2 | ✦ Siege Piercer (cannon) | dmg ×2 · pierces armor · knock 1.1 |
| Missile + Reactor | 2 | ✦ Warhead Silo (missile) | dmg ×1.2 · every hit irradiates (a debuff sidegrade: less raw damage than the Warheads branch, but marks the target for +20% from the whole board) |
| Gatling + Drone Hub | 2 | ✦ Swarm Core (gatling) | dmg ×1.5 · ramp cap 12 (→15/s) · hits slow |
| Laser + Reactor | 3 | ✦ Sun Lance (laser) | dmg ×1.15 · ramp ×1.5 · beam irradiates (late capstone, highest single-target DPS) |
| identical L5 twins | 5 | ✦ Ascendant (same tower) | dmg ×1.6 · range ×1.15 |

## Merge economics
Merging never destroys gold: the survivor's invested total absorbs the consumed twin's, so sell value carries 100% of both. Each merge grants +10% damage (compounding across L4/L5/fusions). Two separate towers still out-damage their merged self per gold (~×6.75 vs ×5.57 at L4 pair-equivalent) — concentration buys a freed cell, range, armor efficiency and buff density, so stacking stays a punished temptation (criterion 4).

## Damage pipeline
Projectile hit: max(1, dmg−armor) [pierce ignores armor] → min(hit cap) → ×1.2 if irradiated → hp.
Burn / aura / beam tick: bypass armor and cap, ×1.2 if irradiated. Regen caps at max hp.
Projectile speed 12 cells/s (missile 5) ×1.1^lvl. Knockback clamps at path start.

## Targeting
Fixed per tower: First-in-path (max path distance) for all projectile towers. Missile: highest (hp − in-flight damage). Laser: highest max hp, sticky until death.

## Combos — adjacency infusions (8-neighbor, projectile towers only)
- Neighbor of a **Brazier** → flaming ammo: projectiles apply 1 burn stack (power 1).
- Neighbor of a **Reactor** → radioactive ammo: hits irradiate the target (0.5s).
- Neighbor of a **Drone Hub** → +25%·1.5^lvl fire rate (Overclock branch: 50% base). Best of multiple hubs, effects stack across kinds.

## Reactions
- **Meltdown**: burn ticks ×2.4 on irradiated enemies (vs ×1.2 for direct hits).
- **Shatter**: splash hits deal +40% to slowed enemies.

## Powerups (cooldown abilities, no gold cost)
| name | unlocks | cd | effect |
|---|---|---|---|
| ☄ Meteor | Bronze | 45s | click-target strike: 3× current-wave base HP to all in r 1.5 |
| ❄ Stasis | Gunpowder | 30s | all enemies 85% slowed for 2.5s |
| ⚡ Overdrive | Atomic | 40s | all towers +50% rate (beams/auras +50% dmg) for 8s |

## Presentation
Enemy silhouettes per archetype: circle grunt · triangle runner · plated circle armored · cross-marked regen · twin-blob splitter · diamond shielded · star elite · block tank · chevron dasher · ring void · spinning hexagon boss. Big boss HP bar top-center on boss waves. Per-age ground texture, spawn arrow & exit portal from path data. Victory fireworks; end screen shows RANK (S≥18 · A≥12 · B≥7 · C lives) + top-6 damage tally (powerups included). Hotkeys: 1-0 towers, Space start/early-call, Esc deselect.

## UI system
No emoji in chrome — all icons are canvas-drawn (tower glyphs in shop, powerup icons, and a 17-glyph stat icon set: damage, rate, range, dps, splash, pierce, burn, knockback, slow, buff, aura, beam, armor, regen, hit-cap, speed, hp, gold) threaded through the unit card, branch descriptions, enemy inspector and HUD. Layout is shift-free: tabular numerals, fixed-width slots for every dynamic text (stats 255px, age 135px, start 134px, powerup buttons 150px, card buttons 226px, Sell pinned right 120px), two-row HUD sized to the 980px board. Locked powerups show dimmed icons + unlock age (no padlocks).
The inspector is a 300px side rail right of the board (the app auto-scales to the viewport). Card layout is a fixed four-slot skeleton so nothing ever shifts: header (glyph, name, level, ★branch, invested/dealt) · 2-column stat grid · Slot A Upgrade (button, or 'maxed' plaque) · Slot B Specialize (two half-width choices at L3, chosen-branch plaque after, ghost hint before) · Slot C Merge (one row: direction-arrow chips, one per eligible twin — hover highlights the twin; ghost recipe when none) · Slot D Sell pinned to the bottom. Merging also works by dragging a tower onto an adjacent twin on the board (eligible twins ring gold during the drag; the drop target survives). Below the action slots a run-info footer fills the rail: drafted Doctrines listed live, ring/hotkey legend pinned above the bottom-anchored Sell. Empty and enemy states share the footer. Enemy card reuses the same zones with live-updating stats.

## Score & end screen
Score = waves·500 + kills·3 + bosses·250 + (victory: lives·100). The end screen is a scorecard: score + NEW BEST flag, run line (map, waves/∞, lives), stat tiles (kills, gold earned, bosses slain, time played), a DAMAGE DEALT bar chart (top 7, colored by era, powerups gold), and drafted doctrines. Per-map best (wave · rank · pts) persists in localStorage and shows on the map cards.

## Session & feel
Page never scrolls: the app auto-scales to the viewport (mouse math is scale-proof). P pauses. Enemies flash white on hit; each tower tracks career damage (shown on its card). Per-map best wave & rank persist in localStorage and show on the map cards.

## Player guidance
Shop-button tooltips per tower · '?' help overlay (rules, combos, powerups, ring legend) · click any enemy for live stats · next-wave preview with threat hint (armor/regen/cap/splits/dashes/boss).

## Wave table (generated)
| w | type | n | hp | spd | armor | regen | cap | bounty |
|---|---|---|---|---|---|---|---|---|
| 1 | Grunt | 8 | 14 | 1.01 | 0 | 0 | — | 1 |
| 2 | Grunt | 9 | 16 | 1.02 | 0 | 0 | — | 2 |
| 3 | Grunt | 10 | 18 | 1.02 | 0 | 0 | — | 2 |
| 4 | Grunt | 11 | 20 | 1.03 | 0 | 0 | — | 3 |
| 5 | Grunt (rush) | 24 | 12 | 1.04 | 0 | 0 | — | 2 |
| 6 | Runner | 12 | 15 | 1.78 | 0 | 0 | — | 3 |
| 7 | Grunt | 13 | 28 | 1.06 | 0 | 0 | — | 4 |
| 8 | Runner | 14 | 19 | 1.81 | 0 | 0 | — | 4 |
| 9 | Runner | 15 | 21 | 1.82 | 0 | 0 | — | 4 |
| 10 | BOSS | 1 | 1009 | 0.55 | 0 | 0 | — | 30 |
| 11 | Armored | 16 | 43 | 1.09 | 3 | 0 | — | 5 |
| 12 | Runner | 17 | 29 | 1.86 | 0 | 0 | — | 6 |
| 13 | Armored | 18 | 55 | 1.10 | 4 | 0 | — | 6 |
| 14 | Armored | 19 | 61 | 1.11 | 4 | 0 | — | 6 |
| 15 | Armored (rush) | 40 | 38 | 1.12 | 4 | 0 | — | 4 |
| 16 | Regen | 20 | 77 | 1.13 | 0 | 1 | — | 7 |
| 17 | Armored | 21 | 86 | 1.14 | 4 | 0 | — | 7 |
| 18 | Regen | 22 | 96 | 1.14 | 0 | 1 | — | 8 |
| 19 | Regen | 23 | 108 | 1.15 | 0 | 1 | — | 8 |
| 20 | BOSS | 1 | 3135 | 0.55 | 4 | 0 | — | 40 |
| 21 | Splitter | 24 | 101 | 1.17 | 0 | 0 | — | 9 |
| 22 | Regen | 25 | 151 | 1.18 | 0 | 2 | — | 9 |
| 23 | Splitter | 26 | 127 | 1.18 | 0 | 0 | — | 10 |
| 24 | Splitter | 27 | 142 | 1.19 | 0 | 0 | — | 10 |
| 25 | Splitter (rush) | 56 | 88 | 1.20 | 0 | 0 | — | 6 |
| 26 | Shielded | 28 | 238 | 1.21 | 0 | 0 | 11 | 11 |
| 27 | Splitter | 29 | 200 | 1.22 | 0 | 0 | — | 11 |
| 28 | Shielded | 30 | 299 | 1.22 | 0 | 0 | 14 | 12 |
| 29 | Shielded | 31 | 334 | 1.23 | 0 | 0 | 16 | 12 |
| 30 | BOSS | 1 | 9737 | 0.55 | 6 | 19 | — | 50 |
| 31 | Elite | 32 | 419 | 1.25 | 7 | 3 | — | 13 |
| 32 | Shielded | 33 | 470 | 1.26 | 0 | 0 | 23 | 13 |
| 33 | Elite | 34 | 526 | 1.26 | 7 | 4 | — | 14 |
| 34 | Elite | 35 | 589 | 1.27 | 7 | 5 | — | 14 |
| 35 | Elite (rush) | 72 | 363 | 1.28 | 7 | 5 | — | 7 |
| 36 | Tank | 36 | 1626 | 0.71 | 0 | 0 | — | 15 |
| 37 | Elite | 37 | 828 | 1.30 | 8 | 7 | — | 15 |
| 38 | Tank | 38 | 2040 | 0.72 | 0 | 0 | — | 15 |
| 39 | Tank | 39 | 2285 | 0.72 | 0 | 0 | — | 16 |
| 40 | BOSS | 1 | 30242 | 0.55 | 8 | 60 | 604 | 60 |
| 41 | Dasher | 40 | 1303 | 1.39 | 0 | 0 | — | 17 |
| 42 | Tank | 41 | 3210 | 0.73 | 0 | 0 | — | 17 |
| 43 | Dasher | 42 | 1634 | 1.41 | 0 | 0 | — | 17 |
| 44 | Dasher | 43 | 1830 | 1.42 | 0 | 0 | — | 18 |
| 45 | Dasher (rush) | 88 | 1127 | 1.43 | 0 | 0 | — | 9 |
| 46 | Void | 44 | 2388 | 1.44 | 10 | 19 | — | 18 |
| 47 | Dasher | 45 | 2571 | 1.44 | 0 | 0 | — | 19 |
| 48 | Void | 46 | 2995 | 1.45 | 11 | 24 | — | 19 |
| 49 | Void | 47 | 3354 | 1.46 | 11 | 27 | — | 20 |
| 50 | BOSS | 1 | 93926 | 0.55 | 10 | 188 | 1878 | 70 |

## Ages: palette & music (generated)
| age | ground | path | accent | bpm | root Hz | osc |
|---|---|---|---|---|---|---|
| Stone | `#3b382f` | `#6b5b43` | `#d9c9a0` | 70 | 110 | triangle |
| Bronze | `#3a342a` | `#7a6134` | `#e0b458` | 78 | 123.5 | triangle |
| Iron | `#32363b` | `#5a646f` | `#b8c4d4` | 86 | 130.8 | triangle |
| Medieval | `#2f382c` | `#66593f` | `#8fbf6f` | 94 | 138.6 | square |
| Gunpowder | `#3a3134` | `#6f5947` | `#e08050` | 102 | 146.8 | square |
| Industrial | `#2d2d31` | `#524d46` | `#c8b898` | 110 | 164.8 | square |
| Modern | `#263039` | `#47586a` | `#55c8e8` | 118 | 174.6 | sawtooth |
| Atomic | `#293327` | `#4c5d45` | `#7fe06f` | 126 | 185 | sawtooth |
| Information | `#232336` | `#454578` | `#9090ff` | 134 | 207.7 | sawtooth |
| Space | `#131320` | `#39395d` | `#c870ff` | 140 | 220 | sawtooth |

Music: 16-step loop at age bpm (8ths). Bass root/2 at step 0, ×0.749 at 8 (7 steps long). Riff [0,·,2,·,4,2,5,·,4,·,2,4,0,·,7,5] over minor-pent [0,3,5,7,10,12,15,17] from root×2, age osc; echoed +1 oct through 0.22s delay (fb 0.35) at age ≥6. Percussion: noise 50ms on 0/4/8/12 (accent on 0/8), 15ms hats on evens at age ≥5. Age-up: 6-note rising pent arpeggio.
SFX: shoot square 160+70·tower Hz 60ms · hit noise 35ms · die chirp 320→70 · boss-die saw 240→40 0.7s · leak saw 95&61 Hz 0.3s · boss spawn saw 70→26 1.1s · victory 220→440 / defeat 160→41 · place thunk · sell coin-down · upgrade rise · branch chime · merge sweep 220→880 · select tick · deny buzz · wave-clear jingle · early-call coin · meteor boom · stasis sweep-down · overdrive arpeggio · infusion chime.
Boss waves mute the melody into a low sawtooth pulse (root/4) until the boss dies.

## Verification — sim.js reference builds, all criteria PASS at these values
1. Intended (full counter-play comp incl. branches & infusions) clears wave 50 with ≥5 lives → **20** (doctrines drafted)
2. Budget (cheap comp, light upgrades) dies waves 28–34 → **31**
3. No non-boss wave exceeds 90s at 1× → worst ~65s (bosses walk 94.5s, die at 27–60s)
4. Greedy (few towers, max upgrades + merged L4s) dies on a rush wave → **25**
5. Intended with lasers swapped for equal-cost extra missiles must fail (composition check) → dies **48**
