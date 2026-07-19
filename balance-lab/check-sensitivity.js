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

function checkExample(example, at, expectedWave) {
  check(example && typeof example === "object", `${at} is missing`);
  check(/^(0|[1-9]\d*)$/.test(example.scenarioSeed), `${at}.scenarioSeed is not an exact decimal string`);
  check(example.screenSeed === example.scenarioSeed, `${at} seed fields disagree`);
  check(example.screenWave === expectedWave, `${at}.screenWave is wrong`);
  check(
    Array.isArray(example.genome) &&
      example.genome.length === report.config.corpusDimensions.policyGenomeDimensions,
    `${at} is missing its exact policy genome`,
  );
  check(example.context && typeof example.context === "object", `${at}.context is missing`);
  for (const key of ["homing", "splash", "sellStrategy", "sellScheduled"]) {
    check(typeof example.context[key] === "boolean", `${at}.context.${key} is invalid`);
  }
  check(typeof example.context.composition === "string", `${at}.context.composition is invalid`);
  check(example.context.screenWave === expectedWave, `${at}.context.screenWave is wrong`);
  check(
    metricNames.every((metric) => Object.hasOwn(example.effect?.metrics || {}, metric)),
    `${at}.effect is missing metrics`,
  );
}

check(report.kind === "perk-sensitivity", "wrong report kind");
check(report.schema === 3, "wrong sensitivity schema");
check(/^(0|[1-9]\d*)$/.test(report.seed), "report seed is not an exact decimal string");
check(report.config?.gaElitesReevaluated === false, "GA must not count unchanged elites as independent samples");
check(report.execution?.countUnit === "logical-complete-simulation-output", "wrong sensitivity count unit");
check(report.execution?.sharedPreInterventionPrefixes === true, "paired prefixes must be shared");
check(report.execution?.broadPrefixesSharedAcrossPerks === true, "broad prefixes must be shared across perks");
check(report.execution?.policyCheckpointsReused === true, "policy checkpoints must be reused");
check(report.execution?.naturalPolicyBaselineArmsReused === true, "identical natural policy arms must be reused");
check(report.execution?.compactSensitivityResults === true, "sensitivity runs must skip unused trace payloads");
check(typeof report.inferenceCaveat === "string" && report.inferenceCaveat.length > 0, "inference caveat is missing");
check(report.perks?.length === 12, "expected all 12 perks");

for (const key of [
  "broadScenarios",
  "gaGenerations",
  "gaPopulation",
  "fullWaveFinalistsPerPerk",
  "screenMaxWave",
  "lateScreenWave",
]) {
  check(
    Number.isSafeInteger(report.config?.[key]) && report.config[key] > 0,
    `config.${key} must be a positive safe integer`,
  );
}
check(
  report.config.lateScreenWave === Math.max(report.config.screenMaxWave, 50),
  "late-screen wave must be max(screenMaxWave, 50)",
);

const expectedBroadPairs = report.config.broadScenarios * report.perks.length;
const expectedTargetedPairs =
  report.config.gaGenerations * report.config.gaPopulation * report.perks.length;
const expectedFullWavePairs = report.config.fullWaveFinalistsPerPerk * report.perks.length;
check(report.counts.broadPairs === expectedBroadPairs, "broad-pair count does not match config");
check(report.counts.targetedPairs === expectedTargetedPairs, "targeted-pair count does not match config");
check(report.counts.fullWavePairs === expectedFullWavePairs, "full-wave-pair count does not match config");
check(
  report.counts.pairedComparisons ===
    expectedBroadPairs + expectedTargetedPairs + expectedFullWavePairs,
  "paired-comparison phase accounting does not close",
);
check(report.counts.pairedComparisons >= minimumPairs, "paired corpus is below the requested minimum");
check(
  report.counts.counterfactualLogicalArms === report.counts.pairedComparisons * 2,
  "every pair must contain logical off and on arms",
);
check(
  report.counts.totalLogicalSimulations ===
    report.counts.counterfactualLogicalArms + report.counts.policyGenerationRuns,
  "total logical-simulation accounting does not close",
);
check(
  report.counts.counterfactualPrefixGroups === report.counts.policyGenerationRuns,
  "every policy-generation run must own exactly one counterfactual prefix group",
);

for (const perk of report.perks) {
  check(categories.has(perk.classification), `${perk.name} has an invalid classification`);
  const targetedWave = report.config.lateScreenPerks.includes(perk.name)
    ? report.config.lateScreenWave
    : report.config.screenMaxWave;
  const expectedPairsByPhase = {
    broad: report.config.broadScenarios,
    targeted: report.config.gaGenerations * report.config.gaPopulation,
    fullWaveValidation: report.config.fullWaveFinalistsPerPerk,
  };
  const expectedWaveByPhase = {
    broad: targetedWave,
    targeted: targetedWave,
    fullWaveValidation: report.config.lateScreenWave,
  };
  for (const [phase, expectedPairs] of Object.entries(expectedPairsByPhase)) {
    const stats = perk[phase];
    check(stats.pairs === expectedPairs, `${perk.name}.${phase} pair count does not match config`);
    check(
      metricNames.every((metric) => stats.metrics[metric]?.ci95?.length === 2),
      `${perk.name}.${phase} is missing metric confidence intervals`,
    );
    check(stats.outcomeChangeRateCi95?.length === 2, `${perk.name}.${phase} is missing outcome-rate CI`);
    check(stats.mechanicalChangeRateCi95?.length === 2, `${perk.name}.${phase} is missing mechanical-rate CI`);
    if (stats.maxOutcomeScore > 0) {
      check(stats.maxOutcomeContext && typeof stats.maxOutcomeContext === "object", `${perk.name}.${phase} lost its maximum-outcome context`);
      check(stats.maxOutcomeContext.screenWave === expectedWaveByPhase[phase], `${perk.name}.${phase} maximum-outcome context has the wrong screen wave`);
    }
    if (stats.maxMechanicalScore > 0) {
      check(stats.maxMechanicalContext && typeof stats.maxMechanicalContext === "object", `${perk.name}.${phase} lost its maximum-mechanical context`);
      check(stats.maxMechanicalContext.screenWave === expectedWaveByPhase[phase], `${perk.name}.${phase} maximum-mechanical context has the wrong screen wave`);
    }
  }
  checkExample(perk.bestAdversarialExample, `${perk.name}.bestAdversarialExample`, targetedWave);
  checkExample(perk.bestFullWaveExample, `${perk.name}.bestFullWaveExample`, report.config.lateScreenWave);
}

const interactionKeys = Object.keys(report.projectileSpeedInteractions || {});
for (const prefix of ["homing:", "splash:", "enemySpeed:", "range:", "map:", "rushProximity:"]) {
  check(interactionKeys.some((key) => key.startsWith(prefix)), `missing projectile-speed interaction ${prefix}`);
}
if (minimumPairs >= 2_000_000) {
  for (const key of ["homing:false", "homing:true", "splash:false", "splash:true"]) {
    check(report.projectileSpeedInteractions[key]?.pairs > 0, `missing populated projectile-speed stratum ${key}`);
  }
  check(
    report.perks[4].broad.outcomeChangeRate > 0 &&
      report.perks[4].broad.metrics.gold.max > 0,
    "War Economy corpus contains no effective sell intervention",
  );
  for (const perk of [10, 11]) {
    check(
      report.perks[perk].targeted.maxOutcomeScore > 0 ||
        report.perks[perk].targeted.maxMechanicalScore > 0,
      `${report.perks[perk].name} targeted search found no active late-game context`,
    );
  }
}
finiteNumbers(report);
console.log(
  `sensitivity report valid: ${report.counts.pairedComparisons} paired comparisons, ${report.counts.totalLogicalSimulations} logical simulations`,
);
