#!/usr/bin/env node
'use strict';
const fs = require('fs');
const path = require('path');
const report = JSON.parse(fs.readFileSync(path.resolve(process.argv[2] || path.join(__dirname, 'out', 'defender-report.json')), 'utf8'));
const failures = [];
if (report.kind !== 'defender') failures.push('not a defender report');
if (report.optimizer !== 'diagonal-cma-es') failures.push('defender did not use CMA-ES');
if (report.tier1MaxWave !== 25 || report.tier2MaxWave <= 50) failures.push('partial/full/endless tiers are missing');
if (!report.gates?.passed) failures.push('source-of-truth gates failed');
if (!report.history?.every(row => Number.isFinite(row.parameterSigma) && row.attackerPool > 0)) failures.push('adaptive CMA or warm-started attacker evidence missing');
const p = report.jsPatch;
if (!p?.constants || p?.towers?.length !== 10 || p?.fusions?.length !== 6 || !p?.rules) failures.push('incomplete JS parameter patch');
if (failures.length) {
  console.error(`defender gates: FAIL\n- ${failures.join('\n- ')}`);
  process.exit(1);
}
console.log(`defender gates: PASS (${report.optimizer}; ${report.tier2MaxWave}-wave finalists; ${p.fusions.length} fusions)`);
