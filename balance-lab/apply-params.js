#!/usr/bin/env node
'use strict';

// Review is the default. Pass --write only after approving the defender report.
const fs = require('fs');
const path = require('path');

const root = path.join(__dirname, '..');
const write = process.argv.includes('--write');
const inputArg = process.argv.find(arg => arg.endsWith('.json'));
const inputPath = path.resolve(inputArg || path.join(__dirname, 'out', 'params.json'));
const gamePath = path.join(root, 'index.html');
const patch = JSON.parse(fs.readFileSync(inputPath, 'utf8'));

if (!patch.constants || !Array.isArray(patch.towers) || patch.towers.length !== 10 ||
    !Array.isArray(patch.fusions) || patch.fusions.length !== 6 || !patch.rules)
  throw new Error('expected a complete constants/towers/fusions/rules params patch');

for (const key of ['hp', 'g', 'cnt', 'spd', 'bty']) {
  if (!Number.isFinite(patch.constants[key])) throw new Error(`invalid constant ${key}`);
}

let html = fs.readFileSync(gamePath, 'utf8');
const original = html;
const number = value => Number(value).toString();
html = html.replace(/const C=\{[^\n]+\};/, `const C={hp:${number(patch.constants.hp)},g:${number(patch.constants.g)},cnt:${number(patch.constants.cnt)},spd:${number(patch.constants.spd)},bty:${number(patch.constants.bty)}};`);

const core = original.split('<script>')[1].split('// ---- browser ----')[0];
const current = new Function(core + ';return {TOWERS,SECRETS,B}')();
const patchObject = (target, update) => {
  for (const [key, value] of Object.entries(update || {})) {
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      if (!target[key] || typeof target[key] !== 'object') target[key] = {};
      patchObject(target[key], value);
    } else target[key] = value;
  }
};
patch.towers.forEach((tower, i) => patchObject(current.TOWERS[i], tower));
patch.fusions.forEach((fusion, i) => patchObject(current.SECRETS[i], fusion));
patchObject(current.B, patch.rules);

const replaceDeclaration = (source, name, value) => {
  const start = source.indexOf(`const ${name}=`);
  const end = source.indexOf(';', start);
  if (start < 0 || end < 0) throw new Error(`could not locate ${name} declaration`);
  return source.slice(0, start) + `const ${name}=${JSON.stringify(value)};` + source.slice(end + 1);
};
html = replaceDeclaration(html, 'TOWERS', current.TOWERS);
html = replaceDeclaration(html, 'SECRETS', current.SECRETS);
html = replaceDeclaration(html, 'B', current.B);

const changed = html !== original;
if (write) {
  fs.writeFileSync(gamePath, html);
  console.log(`${changed ? 'applied' : 'already current'}: ${inputPath} -> ${gamePath}`);
} else {
  console.log(`validated: ${inputPath}; ${changed ? 'would update index.html' : 'index.html already matches'} (pass --write to apply)`);
}
