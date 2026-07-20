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
  "tickCapReached",
];

const rateCountFields = [
  ["pairCensored", "pairCensoredRate", "pairCensoredRateCi95"],
  ["offCapped", "offCappedRate", "offCappedRateCi95"],
  ["onCapped", "onCappedRate", "onCappedRateCi95"],
  ["capTransition", "capTransitionRate", "capTransitionRateCi95"],
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

function closeEnough(left, right, tolerance = 1e-12) {
  return Math.abs(left - right) <= tolerance * Math.max(1, Math.abs(left), Math.abs(right));
}

function wilson95(successes, trials) {
  if (trials === 0) return [0, 1];
  const probability = successes / trials;
  const z = 1.96;
  const denominator = 1 + (z * z) / trials;
  const center = (probability + (z * z) / (2 * trials)) / denominator;
  const radius =
    (z * Math.sqrt((probability * (1 - probability) + (z * z) / (4 * trials)) / trials)) /
    denominator;
  return [Math.max(0, center - radius), Math.min(1, center + radius)];
}

function checkRateCi(rate, ci, at) {
  check(Number.isFinite(rate) && rate >= 0 && rate <= 1, `${at} is not a probability`);
  check(
    Array.isArray(ci) &&
      ci.length === 2 &&
      ci.every(Number.isFinite) &&
      ci[0] >= 0 &&
      (ci[0] <= rate || closeEnough(ci[0], rate)) &&
      (rate <= ci[1] || closeEnough(rate, ci[1])) &&
      ci[1] <= 1,
    `${at} confidence interval is invalid`,
  );
}

function checkMoment(moment, expectedSamples, at) {
  check(moment && typeof moment === "object", `${at} is missing`);
  check(moment.samples === expectedSamples, `${at}.samples does not match its eligible pairs`);
  check(
    [moment.mean, moment.min, moment.max].every(Number.isFinite) &&
      (moment.min <= moment.mean || closeEnough(moment.min, moment.mean)) &&
      (moment.mean <= moment.max || closeEnough(moment.mean, moment.max)),
    `${at} bounds are invalid`,
  );
  check(
    Array.isArray(moment.ci95) &&
      moment.ci95.length === 2 &&
      moment.ci95.every(Number.isFinite) &&
      moment.ci95[0] <= moment.ci95[1],
    `${at}.ci95 is invalid`,
  );
  if (expectedSamples === 0) {
    check(
      moment.mean === 0 &&
        moment.min === 0 &&
        moment.max === 0 &&
        moment.ci95[0] === 0 &&
        moment.ci95[1] === 0,
      `${at} must use zero-valued bounds when it has no eligible samples`,
    );
  }
}

function checkEffectStats(stats, at, expectedPairs) {
  check(stats && typeof stats === "object", `${at} is missing`);
  check(Number.isSafeInteger(stats.pairs) && stats.pairs > 0, `${at}.pairs is invalid`);
  if (expectedPairs !== undefined) {
    check(stats.pairs === expectedPairs, `${at} pair count does not match config`);
  }

  for (const [countName, rateName, ciName] of rateCountFields) {
    const count = stats[countName];
    const rate = stats[rateName];
    check(
      Number.isSafeInteger(count) && count >= 0 && count <= stats.pairs,
      `${at}.${countName} is outside the pair count`,
    );
    check(closeEnough(rate, count / stats.pairs), `${at}.${rateName} does not match its count`);
    checkRateCi(rate, stats[ciName], `${at}.${ciName}`);
    const expectedCi = wilson95(count, stats.pairs);
    check(
      closeEnough(stats[ciName][0], expectedCi[0]) && closeEnough(stats[ciName][1], expectedCi[1]),
      `${at}.${ciName} is not the Wilson interval for its count`,
    );
  }

  check(stats.capTransition <= stats.pairCensored, `${at}.capTransition exceeds censored pairs`);
  check(stats.offCapped <= stats.pairCensored, `${at}.offCapped exceeds censored pairs`);
  check(stats.onCapped <= stats.pairCensored, `${at}.onCapped exceeds censored pairs`);
  check(
    stats.offCapped + stats.onCapped + stats.capTransition === 2 * stats.pairCensored,
    `${at} capped-arm union accounting does not close`,
  );

  for (const rateName of [
    "exactZeroRate",
    "outcomeChangeRate",
    "mechanicalChangeRate",
    "positiveRate",
    "negativeRate",
  ]) {
    check(
      Number.isFinite(stats[rateName]) && stats[rateName] >= 0 && stats[rateName] <= 1,
      `${at}.${rateName} is not a probability`,
    );
  }
  check(
    stats.positiveRate + stats.negativeRate <= 1 + Number.EPSILON,
    `${at} positive and negative rates overlap`,
  );
  checkRateCi(stats.outcomeChangeRate, stats.outcomeChangeRateCi95, `${at}.outcomeChangeRateCi95`);
  checkRateCi(
    stats.mechanicalChangeRate,
    stats.mechanicalChangeRateCi95,
    `${at}.mechanicalChangeRateCi95`,
  );

  checkMoment(stats.utility, stats.pairs, `${at}.utility`);
  for (const metric of metricNames) {
    const expectedSamples =
      metric === "clearSeconds" ? stats.pairs - stats.pairCensored : stats.pairs;
    checkMoment(stats.metrics?.[metric], expectedSamples, `${at}.metrics.${metric}`);
  }
  const tickCapMetric = stats.metrics.tickCapReached;
  const capDeltaCount = stats.onCapped - stats.offCapped;
  const onOnly = (stats.capTransition + capDeltaCount) / 2;
  const offOnly = (stats.capTransition - capDeltaCount) / 2;
  check(Number.isSafeInteger(onOnly) && onOnly >= 0, `${at} on-only cap count is invalid`);
  check(Number.isSafeInteger(offOnly) && offOnly >= 0, `${at} off-only cap count is invalid`);
  check(
    tickCapMetric.min === (offOnly > 0 ? -1 : 0) &&
      tickCapMetric.max === (onOnly > 0 ? 1 : 0),
    `${at}.metrics.tickCapReached bounds disagree with capped-arm counts`,
  );
  check(
    closeEnough(tickCapMetric.mean, capDeltaCount / stats.pairs),
    `${at}.metrics.tickCapReached mean does not match capped-arm counts`,
  );
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
  for (const arm of ["off", "on"]) {
    const capped = example[`${arm}Capped`];
    const ticks = example[`${arm}Ticks`];
    check(typeof capped === "boolean", `${at}.${arm}Capped is not boolean`);
    check(
      Number.isSafeInteger(ticks) && ticks >= 0 && ticks <= report.execution.tickCap,
      `${at}.${arm}Ticks is outside the configured tick cap`,
    );
    if (capped) {
      check(ticks === report.execution.tickCap, `${at}.${arm}Ticks must equal the cap when censored`);
    }
  }
  check(
    metricNames.every((metric) => Object.hasOwn(example.effect?.metrics || {}, metric)),
    `${at}.effect is missing metrics`,
  );
  const capDelta = Number(example.onCapped) - Number(example.offCapped);
  check(
    example.effect.metrics.tickCapReached === capDelta,
    `${at}.effect.metrics.tickCapReached disagrees with arm status`,
  );
  if (example.offCapped || example.onCapped) {
    check(
      example.effect.metrics.clearSeconds === 0,
      `${at}.effect.metrics.clearSeconds must exclude censored arms`,
    );
  }
}

check(report.kind === "perk-sensitivity", "wrong report kind");
check(report.schema === 4, "wrong sensitivity schema");
check(/^(0|[1-9]\d*)$/.test(report.seed), "report seed is not an exact decimal string");
check(/^[0-9a-f]{40}$/.test(report.sourceRevision), "source revision is not an exact Git commit");
check(typeof report.sourceDirty === "boolean", "source dirty-state metadata is missing");
if (minimumPairs >= 2_000_000) {
  check(report.sourceDirty === false, "full sensitivity reports must come from a clean source tree");
}
check(report.config?.gaElitesReevaluated === false, "GA must not count unchanged elites as independent samples");
check(report.config?.gaCensorAware === true, "GA must be censor-aware");
check(
  typeof report.execution?.countUnit === "string" &&
    report.execution.countUnit.length > 0 &&
    !report.execution.countUnit.toLowerCase().includes("complete"),
  "sensitivity count unit must not claim capped outputs are complete",
);
check(report.execution?.tickCap === 3_000_000, "wrong sensitivity tick cap");
check(
  report.execution?.censoredClearTimeExcluded === true,
  "censored clear times must be excluded from clear-time deltas",
);
check(report.execution?.gaCensorAware === true, "execution metadata must mark the GA censor-aware");
check(
  report.tickCapMetadata?.ticksPerArm === report.execution.tickCap,
  "tick-cap metadata disagrees with execution metadata",
);
check(report.tickCapMetadata?.metric === "tickCapReached", "wrong tick-cap transition metric");
check(
  Number.isFinite(report.tickCapMetadata?.dtSeconds) && report.tickCapMetadata.dtSeconds > 0,
  "tick-cap timestep metadata is invalid",
);
check(
  closeEnough(
    report.tickCapMetadata?.simulatedSecondsAtCap,
    report.execution.tickCap * report.tickCapMetadata.dtSeconds,
  ),
  "tick-cap simulated duration does not match ticks and timestep",
);
for (const weight of ["utilityWeight", "outcomeScoreWeight"]) {
  check(
    Number.isFinite(report.tickCapMetadata?.[weight]) && report.tickCapMetadata[weight] > 0,
    `tick-cap ${weight} is invalid`,
  );
}
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
const expectedBroadPolicyRuns =
  report.config.broadScenarios *
  (report.config.lateScreenWave > report.config.screenMaxWave ? 2 : 1);
check(
  report.counts.policyGenerationRuns ===
    expectedBroadPolicyRuns + expectedTargetedPairs + expectedFullWavePairs,
  "policy-generation run count does not match config",
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
    checkEffectStats(stats, `${perk.name}.${phase}`, expectedPairs);
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

const interactions = report.projectileSpeedInteractions || {};
const interactionKeys = Object.keys(interactions);
for (const [key, stats] of Object.entries(interactions)) {
  checkEffectStats(stats, `projectileSpeedInteractions.${key}`);
}

const expectedInteractionLabels = {
  homing: ["false", "true"],
  splash: ["false", "true"],
  map: report.config.corpusDimensions.maps.map(String),
  enemySpeed: report.config.corpusDimensions.enemySpeedFactors.map((value) => value.toFixed(2)),
  range: report.config.corpusDimensions.towerRangeFactors.map((value) => value.toFixed(2)),
  activationWave: report.config.corpusDimensions.activationWaves.map(String),
  rushProximity: [
    ...new Set(
      report.config.corpusDimensions.activationWaves.map((wave) => {
        if (wave % 5 === 4) return "immediately-before";
        if (wave % 5 === 0) return "on-rush";
        return "between-rushes";
      }),
    ),
  ],
  focusTower: Array.from(
    { length: report.config.corpusDimensions.focusTowers },
    (_, index) => String(index),
  ),
  composition: ["empty", "mono", "focused-mix", "broad-mix"],
  sellStrategy: ["false", "true"],
  screenWave: [String(report.config.lateScreenWave)],
  reaction: ["fast", "medium", "slow"],
};
const fullCorpus = minimumPairs >= 2_000_000;
const interactionPairs = report.perks[8].broad.pairs;
for (const [prefix, allowedLabels] of Object.entries(expectedInteractionLabels)) {
  const entries = Object.entries(interactions).filter(([key]) => key.startsWith(`${prefix}:`));
  check(entries.length > 0, `missing projectile-speed interaction ${prefix}:`);
  const observedLabels = entries.map(([key]) => key.slice(prefix.length + 1));
  check(
    observedLabels.every((label) => allowedLabels.includes(label)),
    `projectile-speed interaction ${prefix}: has an unexpected stratum`,
  );
  check(
    entries.reduce((sum, [, stats]) => sum + stats.pairs, 0) === interactionPairs,
    `projectile-speed interaction ${prefix}: partition does not close`,
  );
  for (const countName of rateCountFields.map(([name]) => name)) {
    check(
      entries.reduce((sum, [, stats]) => sum + stats[countName], 0) ===
        report.perks[8].broad[countName],
      `projectile-speed interaction ${prefix}: ${countName} partition does not close`,
    );
  }
  check(
    entries.reduce((sum, [, stats]) => sum + stats.metrics.clearSeconds.samples, 0) ===
      report.perks[8].broad.metrics.clearSeconds.samples,
    `projectile-speed interaction ${prefix}: clear-time sample partition does not close`,
  );
  if (fullCorpus) {
    for (const label of allowedLabels) {
      check(
        interactions[`${prefix}:${label}`]?.pairs > 0,
        `missing populated projectile-speed stratum ${prefix}:${label}`,
      );
    }
  }
}
const jointEntries = Object.entries(interactions).filter(([key]) => key.startsWith("joint:"));
check(jointEntries.length > 0, "missing joint projectile-speed interactions");
check(
  jointEntries.reduce((sum, [, stats]) => sum + stats.pairs, 0) === interactionPairs,
  "joint projectile-speed interaction partition does not close",
);
for (const countName of rateCountFields.map(([name]) => name)) {
  check(
    jointEntries.reduce((sum, [, stats]) => sum + stats[countName], 0) ===
      report.perks[8].broad[countName],
    `joint projectile-speed interaction ${countName} partition does not close`,
  );
}
check(
  jointEntries.reduce((sum, [, stats]) => sum + stats.metrics.clearSeconds.samples, 0) ===
    report.perks[8].broad.metrics.clearSeconds.samples,
  "joint projectile-speed clear-time sample partition does not close",
);

if (minimumPairs >= 2_000_000) {
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
