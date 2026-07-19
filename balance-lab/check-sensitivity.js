"use strict";

const fs = require("fs");
const path = process.argv[2] || "balance-lab/out/sensitivity-report.json";
const minimumPairs = Number(process.argv[3] || 1);
const report = JSON.parse(fs.readFileSync(path, "utf8"));
const categories = new Set([
  "meaningful",
  "niche-only",
  "redundant",
  "harmful",
  "mechanically inactive",
]);
const metricNames = [
  "survivalWave",
  "lives",
  "gold",
  "leaks",
  "leakDamage",
  "clearSeconds",
  "effectiveDamage",
  "overkillDamage",
  "projectileLatencySeconds",
  "wastedProjectiles",
];

function check(condition, message) {
  if (!condition) throw new Error(message);
}

function finiteNumbers(value, at = "report") {
  if (typeof value === "number") {
    check(Number.isFinite(value), `${at} is not finite`);
  } else if (Array.isArray(value)) {
    value.forEach((entry, index) => finiteNumbers(entry, `${at}[${index}]`));
  } else if (value && typeof value === "object") {
    Object.entries(value).forEach(([key, entry]) => finiteNumbers(entry, `${at}.${key}`));
  }
}

check(report.kind === "perk-sensitivity", "wrong report kind");
check(report.perks?.length === 12, "expected all 12 perks");
check(report.counts.pairedComparisons >= minimumPairs, "paired corpus is below the requested minimum");
check(report.counts.counterfactualRuns === report.counts.pairedComparisons * 2, "every pair must contain off and on runs");
check(
  report.counts.totalSimulationRuns ===
    report.counts.counterfactualRuns + report.counts.policyGenerationRuns,
  "total run accounting does not close",
);

for (const perk of report.perks) {
  check(categories.has(perk.classification), `${perk.name} has an invalid classification`);
  for (const phase of ["broad", "targeted", "fullWaveValidation"]) {
    const stats = perk[phase];
    check(stats.pairs > 0, `${perk.name}.${phase} has no paired samples`);
    check(
      metricNames.every((metric) => stats.metrics[metric]?.ci95?.length === 2),
      `${perk.name}.${phase} is missing metric confidence intervals`,
    );
    check(stats.outcomeChangeRateCi95?.length === 2, `${perk.name}.${phase} is missing outcome-rate CI`);
    check(stats.mechanicalChangeRateCi95?.length === 2, `${perk.name}.${phase} is missing mechanical-rate CI`);
  }
}

const interactionKeys = Object.keys(report.projectileSpeedInteractions || {});
for (const prefix of ["homing:", "splash:", "enemySpeed:", "range:", "map:", "rushProximity:"]) {
  check(interactionKeys.some((key) => key.startsWith(prefix)), `missing projectile-speed interaction ${prefix}`);
}
finiteNumbers(report);
console.log(
  `sensitivity report valid: ${report.counts.pairedComparisons} paired comparisons, ${report.counts.totalSimulationRuns} total simulations`,
);
