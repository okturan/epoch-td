#!/usr/bin/env node
'use strict';

// Stable JSON protocol for the browser engine.  This is deliberately a thin
// adapter: index.html remains the authority and this file never reimplements
// combat rules.
const fs = require('fs');
const path = require('path');

const html = fs.readFileSync(path.join(__dirname, 'index.html'), 'utf8');
const core = html.split('<script>')[1].split('// ---- browser ----')[0];
const boot = new Function(core + `;return {
  mkState, update, startWave, place, upgrade, sell, towerAt, branch, merge,
  secretMerge, secretPair, pickRelic, power, setMap, TOWERS, WAVES, C
}`);

function patchObject(target, patch) {
  if (!patch) return;
  for (const [key, value] of Object.entries(patch)) {
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      if (!target[key] || typeof target[key] !== 'object') target[key] = {};
      patchObject(target[key], value);
    } else target[key] = value;
  }
}

function applyAction(G, S, action) {
  const { op, x, y } = action;
  if (op === 'place') return G.place(S, action.tower, x, y);
  const tower = G.towerAt(S, x, y);
  if (op === 'upgrade') return !!tower && G.upgrade(S, tower);
  if (op === 'branch') return !!tower && G.branch(S, tower, action.branch);
  if (op === 'merge') {
    const mate = action.with ? G.towerAt(S, action.with.x, action.with.y) : undefined;
    return !!tower && G.merge(S, tower, mate);
  }
  if (op === 'fuse') {
    const mate = G.towerAt(S, action.with.x, action.with.y);
    const recipe = tower && mate ? G.secretPair(tower, mate) : null;
    return recipe != null && G.secretMerge(S, tower, mate, recipe);
  }
  if (op === 'sell') { if (!tower) return false; G.sell(S, tower); return true; }
  if (op === 'power') return G.power(S, action.power, x || 0, y || 0);
  throw new Error(`unknown action op: ${op}`);
}

function snapshot(S, waveSeconds) {
  return {
    wave: S.wave,
    seconds: +waveSeconds.toFixed(6),
    lives: S.lives,
    gold: +S.gold.toFixed(6),
    kills: S.kills,
    won: !!S.won,
    over: !!S.over,
    towerDamage: S.towers.map(t => ({
      x: t.x, y: t.y, tower: t.i, level: t.lvl, branch: t.br,
      fusion: t.sec == null ? null : t.sec,
      damage: +(t.dd || 0).toFixed(6),
    })).sort((a, b) => a.y - b.y || a.x - b.x),
  };
}

function run(input) {
  const G = boot();
  G.setMap(input.map || 0);
  patchObject(G.C, input.params && input.params.constants);
  if (input.params && input.params.towers) {
    for (const [i, towerPatch] of Object.entries(input.params.towers))
      patchObject(G.TOWERS[Number(i)], towerPatch);
  }
  // Formula changes invalidate the eager first 50 rows.
  if (input.params && input.params.constants)
    for (let i = 0; i < G.WAVES.length; i++) G.WAVES[i] = undefined;

  const S = G.mkState();
  if (input.initial) patchObject(S, input.initial);
  const actions = (input.actions || []).map((a, order) => ({ ...a, order }))
    .sort((a, b) => (a.wave || 0) - (b.wave || 0) || a.order - b.order);
  const pending = actions.slice(), rejected = [], trace = [];
  const maxWave = input.maxWave == null ? 50 : input.maxWave;
  const tickLimit = input.tickLimit || 3000000;
  let ticks = 0;

  while (!S.over && S.wave < maxWave && ticks < tickLimit) {
    if (S.pick) G.pickRelic(S, Math.abs(Number(input.seed || 0) + S.wave) % S.pick.length);
    while (pending.length && (pending[0].wave || 0) <= S.wave) {
      const a = pending.shift();
      if (!applyAction(G, S, a)) rejected.push({ ...a, reason: 'illegal-or-unaffordable' });
    }
    if (S.phase !== 'wave') G.startWave(S);
    const started = S.t;
    while (S.phase === 'wave' && !S.over && ticks < tickLimit) {
      G.update(S, 1 / 30); S.ev.length = 0; ticks++;
    }
    trace.push(snapshot(S, S.t - started));
  }
  if (ticks >= tickLimit) throw new Error(`tick limit ${tickLimit} exceeded`);
  return {
    protocol: 1,
    seed: String(input.seed || 0), map: input.map || 0,
    trace, rejected, pending: pending.length,
    result: snapshot(S, trace.reduce((sum, row) => sum + row.seconds, 0)),
  };
}

function main() {
  const arg = process.argv[2];
  const raw = arg ? fs.readFileSync(arg, 'utf8') : fs.readFileSync(0, 'utf8');
  const input = JSON.parse(raw);
  const result = Array.isArray(input) ? input.map(run) : run(input);
  process.stdout.write(JSON.stringify(result) + '\n');
}

if (require.main === module) {
  try { main(); } catch (error) { console.error(error.stack || error); process.exit(1); }
}
module.exports = { run };
