#!/usr/bin/env node
'use strict';
const fs = require('fs');
const path = require('path');
const reportPath = path.resolve(process.argv[2] || path.join(__dirname, 'out', 'report.json'));
const report = JSON.parse(fs.readFileSync(reportPath, 'utf8'));
const failures = [];
if (report.kind !== 'attack') failures.push('not an attacker report');
if (!Array.isArray(report.archive) || report.archive.length < 20) failures.push('fewer than 20 viable MAP-Elites cells');
if (!report.archive?.some(elite => elite.fusion === 2)) failures.push('Warhead Silo strategy was not rediscovered');
if ((report.towerPicks || []).filter(value => value > 0).length < 6) failures.push('fewer than six tower types were explored');
if (!report.clearTimeCurve?.length) failures.push('clear-time curve is empty');
if (!report.history?.every(row => Number.isFinite(row.exploitability))) failures.push('invalid exploitability history');
if (failures.length) {
  console.error(`balance search gates: FAIL\n- ${failures.join('\n- ')}`);
  process.exit(1);
}
console.log(`balance search gates: PASS (${report.archive.length} cells; Warhead Silo rediscovered)`);
