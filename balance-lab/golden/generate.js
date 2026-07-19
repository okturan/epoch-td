#!/usr/bin/env node
'use strict';
const fs = require('fs');
const path = require('path');
const root = path.join(__dirname, '..', '..');
const sim = fs.readFileSync(path.join(root, 'sim.js'), 'utf8');
const intended = eval(sim.match(/const INTENDED=(\[[\s\S]*?\]);/)[1]);
const budget = eval(sim.match(/const BUDGET=(\[[\s\S]*?\]);/)[1]);
const greedy = eval(sim.match(/const GREEDY=(\[[\s\S]*?\]);/)[1]);

function actions(build) {
  return build.map(([wave, op, x, y]) => {
    if (typeof op === 'number') return { wave, op:'place', tower:op, x, y };
    if (op === 'u') return { wave, op:'upgrade', x, y };
    if (op === 'b1' || op === 'b2') return { wave, op:'branch', branch:Number(op[1]), x, y };
    if (op === 's') return { wave, op:'sell', x, y };
    if (op === 'm') return { wave, op:'merge', x, y };
    throw new Error(`unsupported sim action ${op}`);
  });
}

const count = Number(process.argv[2] || 2000);
const builds = [intended, budget, greedy];
const cases = [];
for (let i=0; i<count; i++) {
  const maxWave = i < 9 ? 50 : 1 + (i * 17) % 50;
  const test = { seed: i * 7919 + 17, map: i % 3, maxWave,
    actions: actions(builds[i % builds.length]).filter(a => a.wave < maxWave) };
  if (i % 5 === 4) {
    const tower = (Math.floor(i / 5) * 7) % 10, towers = Array.from({length:10}, () => ({}));
    towers[tower] = {d:[6,13,22,3,55,8,105,12,0,40][tower] * (0.92 + (i % 4) * 0.04)};
    if (![7,8,9].includes(tower)) towers[tower].r = [1,.5,.7,2,.35,3,.6,0,0,0][tower] * (0.96 + (i % 3) * 0.04);
    const branchDamage=[2,1.8,1.8,null,1.8,null,1.8,2,null,1.5][tower];
    if (branchDamage) towers[tower].b1={dm:branchDamage * (0.98 + (i % 2) * .04)};
    test.params = {constants:{hp:14 * (0.96 + (i % 3) * 0.04),g:1.115 + (i % 3) * .005},towers};
  }
  // Exercise every parameter surface through the exact browser oracle, not
  // only the base tower/curve fields.  These cases can afford and construct a
  // real fusion, draft doctrine-sensitive rules, and fire timed powers.
  if (i % 25 === 24) {
    test.map = 0;
    test.maxWave = Math.max(6, maxWave);
    test.initial = {gold:100000};
    test.actions = [
      {wave:0,op:'place',tower:1,x:6,y:0},
      {wave:0,op:'place',tower:3,x:7,y:0},
      {wave:0,op:'upgrade',x:6,y:0},{wave:0,op:'upgrade',x:6,y:0},
      {wave:0,op:'upgrade',x:7,y:0},{wave:0,op:'upgrade',x:7,y:0},
      {wave:0,op:'fuse',x:6,y:0,with:{x:7,y:0}},
      {wave:1,tick:5,op:'power',power:0,x:6,y:1},
      {wave:2,tick:7,op:'power',power:1},
      {wave:3,tick:9,op:'power',power:2},
    ];
    const fusions = Array.from({length:6}, () => ({}));
    fusions[0] = {f:{dm:1.85 + (i % 3) * .1,sp:1.25,bu:1,bum:2.15}};
    test.params = {
      constants:{hp:14.25,g:1.119,cnt:.805,spd:.0285,bty:3.9},
      towers:test.params && test.params.towers || Array.from({length:10}, () => ({})),
      fusions,
      rules:{upgrade_cost:.57,sell_refund:.72,doctrine_upgrade_cost:.47,
        doctrine_sell_refund:.87,doctrine_bounty:1.2,doctrine_range:1.09,
        doctrine_cooldown:.68,doctrine_knockback:1.55,
        doctrine_boss_knockback:.32,doctrine_lives:4,
        doctrine_projectile_speed:1.27,doctrine_splash:1.23,
        doctrine_irradiate:1.1,doctrine_slow:.48,burn_cap:5.25,
        doctrine_burn_cap:7.5,power_cooldowns:[41,27,36],power_ages:[0,0,0],
        meteor_damage:3.4,meteor_radius:1.65,stasis_duration:2.7,
        stasis_factor:.12,overdrive_duration:8.4,overdrive_multiplier:1.58}
    };
  }
  cases.push(test);
}
process.stdout.write(JSON.stringify(cases));
