"use strict";

const fs = require("fs");
const nodePath = require("path");
const { execFileSync } = require("child_process");
const selfTestOnly = process.env.SENSITIVITY_CHECK_SELF_TEST === "1";
const reportPath = process.argv[2] || "balance-lab/out/sensitivity-report.json";
const minimumPairs = Number(process.argv[3] || 1);
const fullCorpus = minimumPairs >= 2_000_000;
const report = selfTestOnly ? null : JSON.parse(fs.readFileSync(reportPath, "utf8"));
const repoRoot = nodePath.resolve(__dirname, "..");
const categories = new Set([
  "meaningful",
  "niche-only",
  "redundant",
  "harmful",
  "mechanically inactive",
]);
const insufficientCategory = "insufficient complete evidence";
const ordinaryMetricNames = [
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
  "damagePerCombatSecond",
  "overkillPerKill",
  "wastedShotRate",
  "knockbackPerCombatSecond",
  "slowCoverageRate",
  "powerUses",
];
const metricNames = [
  ...ordinaryMetricNames,
  "tickCapReached",
];

const capRateCountFields = [
  ["pairCensored", "pairCensoredRate", "pairCensoredRateCi95"],
  ["bothCapped", "bothCappedRate", "bothCappedRateCi95"],
  ["offCapped", "offCappedRate", "offCappedRateCi95"],
  ["onCapped", "onCappedRate", "onCappedRateCi95"],
  ["capTransition", "capTransitionRate", "capTransitionRateCi95"],
];
const ordinaryRateCountFields = [
  ["exactZero", "exactZeroRate", "exactZeroRateCi95"],
  ["outcomeChanged", "outcomeChangeRate", "outcomeChangeRateCi95"],
  ["mechanicalChanged", "mechanicalChangeRate", "mechanicalChangeRateCi95"],
  ["positive", "positiveRate", "positiveRateCi95"],
  ["negative", "negativeRate", "negativeRateCi95"],
  ["pathologicalStall", "pathologicalStallRate", "pathologicalStallRateCi95"],
];
const canonicalPerks = [
  [0, "Napalm Doctrine"],
  [1, "Bounty Reform"],
  [2, "Rapid Logistics"],
  [3, "Standing Army"],
  [4, "War Economy"],
  [5, "Scrap Drive"],
  [6, "Shock Doctrine"],
  [7, "Iron Curtain"],
  [8, "Overcharge Rails"],
  [9, "Siege Corps"],
  [10, "Fission Ammo"],
  [11, "Cold Snap"],
];
const canonicalLateScreenPerks = ["Overcharge Rails", "Fission Ammo", "Cold Snap"];
const canonicalCorpusDimensions = {
  maps: [0, 1, 2],
  enemySpeedFactors: [0.65, 0.82, 1, 1.18, 1.4],
  towerRangeFactors: [0.75, 0.9, 1, 1.15, 1.3],
  enemyHpFactors: [0.85, 1, 1.18],
  enemyCountFactors: [0.75, 1, 1.25],
  activationWaves: [10, 14, 15, 20, 24],
  focusTowers: 10,
  policyGenomeDimensions: 46,
  policyVariation: [
    "placements",
    "tower priorities",
    "branches",
    "merges",
    "fusions",
    "doctrines",
    "powerups",
    "early calls",
    "reaction delay",
    "APM",
    "sell timing and target",
  ],
};
const canonicalJointPrefixes = [
  "homing",
  "splash",
  "enemySpeed",
  "range",
  "map",
  "activationWave",
  "rushProximity",
  "focusTower",
  "composition",
  "sellStrategy",
  "screenWave",
  "reaction",
];

function check(condition, message) {
  if (!condition) throw new Error(message);
}

function exactlyEqual(actual, expected) {
  if (Array.isArray(expected)) {
    return (
      Array.isArray(actual) &&
      actual.length === expected.length &&
      expected.every((value, index) => exactlyEqual(actual[index], value))
    );
  }
  if (expected && typeof expected === "object") {
    if (!actual || typeof actual !== "object" || Array.isArray(actual)) return false;
    const actualKeys = Object.keys(actual).sort();
    const expectedKeys = Object.keys(expected).sort();
    return (
      exactlyEqual(actualKeys, expectedKeys) &&
      expectedKeys.every((key) => exactlyEqual(actual[key], expected[key]))
    );
  }
  return Object.is(actual, expected);
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

function tickCapMoment(pairs, offOnly, onOnly) {
  const zero = pairs - offOnly - onOnly;
  check(zero >= 0, "tick-cap distribution exceeds its pair count");
  const sum = onOnly - offOnly;
  const sumSquares = onOnly + offOnly;
  const mean = sum / pairs;
  let ci95 = [mean, mean];
  if (pairs >= 2) {
    const variance = Math.max(0, (sumSquares - (sum * sum) / pairs) / (pairs - 1));
    const radius = 1.96 * Math.sqrt(variance / pairs);
    ci95 = [mean - radius, mean + radius];
  }
  return {
    samples: pairs,
    mean,
    min: offOnly > 0 ? -1 : zero > 0 ? 0 : 1,
    max: onOnly > 0 ? 1 : zero > 0 ? 0 : -1,
    ci95,
  };
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
      moment.ci95[0] <= moment.ci95[1] &&
      (moment.ci95[0] <= moment.mean || closeEnough(moment.ci95[0], moment.mean)) &&
      (moment.mean <= moment.ci95[1] || closeEnough(moment.mean, moment.ci95[1])),
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

  check(
    Number.isSafeInteger(stats.completePairs) &&
      stats.completePairs >= 0 &&
      stats.completePairs <= stats.pairs,
    `${at}.completePairs is outside the pair count`,
  );

  for (const [countName, rateName, ciName] of capRateCountFields) {
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

  check(
    stats.completePairs === stats.pairs - stats.pairCensored,
    `${at}.completePairs does not exclude every censored pair`,
  );
  check(
    stats.bothCapped === stats.pairCensored - stats.capTransition,
    `${at}.bothCapped does not match censored pairs minus cap transitions`,
  );
  check(stats.bothCapped <= stats.offCapped, `${at}.bothCapped exceeds off-capped arms`);
  check(stats.bothCapped <= stats.onCapped, `${at}.bothCapped exceeds on-capped arms`);
  const offOnly = stats.offCapped - stats.bothCapped;
  const onOnly = stats.onCapped - stats.bothCapped;
  check(
    offOnly >= 0 && onOnly >= 0 && offOnly + onOnly === stats.capTransition,
    `${at} one-arm cap-transition accounting does not close`,
  );
  check(
    stats.bothCapped + offOnly + onOnly === stats.pairCensored,
    `${at} capped-pair direction accounting does not close`,
  );

  for (const [countName, rateName, ciName] of ordinaryRateCountFields) {
    const count = stats[countName];
    const rate = stats[rateName];
    check(
      Number.isSafeInteger(count) && count >= 0 && count <= stats.completePairs,
      `${at}.${countName} is outside the complete-pair count`,
    );
    const expectedRate = stats.completePairs === 0 ? 0 : count / stats.completePairs;
    check(closeEnough(rate, expectedRate), `${at}.${rateName} does not match its count`);
    checkRateCi(rate, stats[ciName], `${at}.${ciName}`);
    const expectedCi = wilson95(count, stats.completePairs);
    check(
      closeEnough(stats[ciName][0], expectedCi[0]) && closeEnough(stats[ciName][1], expectedCi[1]),
      `${at}.${ciName} is not the Wilson interval for its complete-pair count`,
    );
  }
  check(
    stats.positive + stats.negative <= stats.completePairs,
    `${at} positive and negative complete-pair counts overlap`,
  );
  const changedUnion = stats.completePairs - stats.exactZero;
  check(
    changedUnion >= Math.max(stats.outcomeChanged, stats.mechanicalChanged) &&
      changedUnion <= stats.outcomeChanged + stats.mechanicalChanged,
    `${at} exact-zero count is infeasible for its outcome/mechanical change counts`,
  );

  checkMoment(stats.utility, stats.completePairs, `${at}.utility`);
  for (const metric of ordinaryMetricNames) {
    checkMoment(stats.metrics?.[metric], stats.completePairs, `${at}.metrics.${metric}`);
  }
  checkMoment(stats.metrics?.tickCapReached, stats.pairs, `${at}.metrics.tickCapReached`);
  const tickCapMetric = stats.metrics.tickCapReached;
  const expectedTickCapMoment = tickCapMoment(stats.pairs, offOnly, onOnly);
  check(
    tickCapMetric.min === expectedTickCapMoment.min &&
      tickCapMetric.max === expectedTickCapMoment.max,
    `${at}.metrics.tickCapReached extrema disagree with capped-arm counts`,
  );
  check(
    closeEnough(tickCapMetric.mean, expectedTickCapMoment.mean),
    `${at}.metrics.tickCapReached mean does not match capped-arm counts`,
  );
  check(
    closeEnough(tickCapMetric.ci95[0], expectedTickCapMoment.ci95[0]) &&
      closeEnough(tickCapMetric.ci95[1], expectedTickCapMoment.ci95[1]),
    `${at}.metrics.tickCapReached.ci95 does not match the producer Moment formula`,
  );
  for (const [scoreName, contextName] of [
    ["maxOutcomeScore", "maxOutcomeContext"],
    ["maxMechanicalScore", "maxMechanicalContext"],
  ]) {
    check(
      Number.isFinite(stats[scoreName]) && stats[scoreName] >= 0,
      `${at}.${scoreName} is invalid`,
    );
    if (stats.completePairs === 0) {
      check(stats[scoreName] === 0, `${at}.${scoreName} must ignore censored pairs`);
    }
    if (stats[scoreName] === 0) {
      check(stats[contextName] == null, `${at}.${contextName} must be null for a zero maximum`);
    } else {
      check(
        stats[contextName] && typeof stats[contextName] === "object",
        `${at}.${contextName} is missing for a positive maximum`,
      );
    }
  }
}

function checkSensitivityContext(context, at, expectedWave) {
  check(context && typeof context === "object", `${at} is missing`);
  for (const key of ["homing", "splash", "sellStrategy", "sellScheduled"]) {
    check(typeof context[key] === "boolean", `${at}.${key} is invalid`);
  }
  for (const [key, allowed] of [
    ["map", canonicalCorpusDimensions.maps],
    ["enemySpeedFactor", canonicalCorpusDimensions.enemySpeedFactors],
    ["rangeFactor", canonicalCorpusDimensions.towerRangeFactors],
    ["hpFactor", canonicalCorpusDimensions.enemyHpFactors],
    ["countFactor", canonicalCorpusDimensions.enemyCountFactors],
    ["activationWave", canonicalCorpusDimensions.activationWaves],
  ]) {
    check(allowed.includes(context[key]), `${at}.${key} is outside the canonical corpus`);
  }
  check(
    Number.isSafeInteger(context.focusTower) &&
      context.focusTower >= 0 &&
      context.focusTower < canonicalCorpusDimensions.focusTowers,
    `${at}.focusTower is invalid`,
  );
  check(
    ["empty", "mono", "focused-mix", "broad-mix"].includes(context.composition),
    `${at}.composition is invalid`,
  );
  const expectedRushProximity =
    context.activationWave % 5 === 4
      ? "immediately-before"
      : context.activationWave % 5 === 0
        ? "on-rush"
        : "between-rushes";
  check(
    context.rushProximity === expectedRushProximity,
    `${at}.rushProximity disagrees with activationWave`,
  );
  check(
    Number.isSafeInteger(context.sellSlot) && context.sellSlot >= 0 && context.sellSlot < 8,
    `${at}.sellSlot is invalid`,
  );
  check(
    Number.isSafeInteger(context.reactionTicks) &&
      context.reactionTicks >= 0 &&
      context.reactionTicks <= 90,
    `${at}.reactionTicks is invalid`,
  );
  const expectedReactionBucket =
    context.reactionTicks <= 10 ? "fast" : context.reactionTicks <= 30 ? "medium" : "slow";
  check(
    context.reactionBucket === expectedReactionBucket,
    `${at}.reactionBucket disagrees with reactionTicks`,
  );
  check(
    Number.isFinite(context.apm) && context.apm >= 10 && context.apm <= 180,
    `${at}.apm is invalid`,
  );
  check(context.screenWave === expectedWave, `${at}.screenWave is wrong`);
}

function checkExample(example, at, expectedWave) {
  check(example && typeof example === "object", `${at} is missing`);
  check(/^(0|[1-9]\d*)$/.test(example.scenarioSeed), `${at}.scenarioSeed is not an exact decimal string`);
  check(example.screenSeed === example.scenarioSeed, `${at} seed fields disagree`);
  check(example.screenWave === expectedWave, `${at}.screenWave is wrong`);
  check(
    Array.isArray(example.genome) &&
      example.genome.length === report.config.corpusDimensions.policyGenomeDimensions &&
      example.genome.every(Number.isFinite),
    `${at} is missing its exact policy genome`,
  );
  const context = example.context;
  checkSensitivityContext(context, `${at}.context`, expectedWave);
  for (const key of [
    "screenWave",
    "map",
    "enemySpeedFactor",
    "rangeFactor",
    "hpFactor",
    "countFactor",
    "activationWave",
    "rushProximity",
    "focusTower",
    "sellStrategy",
    "sellSlot",
    "sellScheduled",
    "reactionTicks",
    "apm",
  ]) {
    check(exactlyEqual(example[key], context[key]), `${at}.${key} disagrees with its context`);
  }
  check(example.genome[44] === context.reactionTicks, `${at}.genome[44] disagrees with reactionTicks`);
  check(example.genome[45] === context.apm, `${at}.genome[45] disagrees with apm`);
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
    example.offCapped === false && example.onCapped === false,
    `${at} is an ordinary best example but contains a censored arm`,
  );
  check(
    example.ordinaryMetricsEligible === true,
    `${at} must be explicitly eligible for ordinary metrics`,
  );
  check(
    exactlyEqual(Object.keys(example.effect?.metrics || {}).sort(), [...metricNames].sort()),
    `${at}.effect does not contain exactly the canonical metrics`,
  );
  const metrics = example.effect.metrics;
  check(metricNames.every((metric) => Number.isFinite(metrics[metric])), `${at}.effect has non-finite metrics`);
  check(
    metrics.tickCapReached === 0,
    `${at}.effect.metrics.tickCapReached must be zero for an ordinary best example`,
  );
  const expectedUtility =
    metrics.survivalWave * 100 +
    metrics.lives * 5 +
    metrics.gold * 0.02 -
    metrics.leaks * 2 -
    metrics.leakDamage * 3 -
    metrics.clearSeconds * 0.05 -
    metrics.wastedProjectiles * 0.1;
  const expectedOutcomeScore =
    Math.abs(metrics.survivalWave) * 100 +
    Math.abs(metrics.lives) * 10 +
    Math.abs(metrics.gold) * 0.01 +
    Math.abs(metrics.leaks) * 5 +
    Math.abs(metrics.leakDamage) * 10 +
    Math.abs(metrics.clearSeconds) * 0.1;
  const expectedMechanicalScore =
    Math.abs(metrics.effectiveDamage) * 0.001 +
    Math.abs(metrics.overkillDamage) * 0.001 +
    Math.abs(metrics.projectileLatencySeconds) * 10 +
    Math.abs(metrics.wastedProjectiles) +
    Math.abs(metrics.damagePerCombatSecond) +
    Math.abs(metrics.overkillPerKill) +
    Math.abs(metrics.wastedShotRate) * 100 +
    Math.abs(metrics.knockbackPerCombatSecond) * 10 +
    Math.abs(metrics.slowCoverageRate) * 10 +
    Math.abs(metrics.powerUses);
  check(
    Number.isFinite(example.effect.utility) && closeEnough(example.effect.utility, expectedUtility),
    `${at}.effect.utility does not match its metrics`,
  );
  check(
    Number.isFinite(example.effect.outcomeScore) &&
      closeEnough(example.effect.outcomeScore, expectedOutcomeScore),
    `${at}.effect.outcomeScore does not match its metrics`,
  );
  check(
    Number.isFinite(example.effect.mechanicalScore) &&
      closeEnough(example.effect.mechanicalScore, expectedMechanicalScore),
    `${at}.effect.mechanicalScore does not match its metrics`,
  );
  check(["benefit", "harm", "mechanical"].includes(example.searchObjective), `${at}.searchObjective is invalid`);
  const expectedFitness =
    example.searchObjective === "benefit"
      ? expectedUtility * 1_000 + expectedMechanicalScore
      : example.searchObjective === "harm"
        ? -expectedUtility * 1_000 + expectedMechanicalScore
        : expectedMechanicalScore * 1_000 + expectedOutcomeScore;
  check(
    Number.isFinite(example.fitness) && closeEnough(example.fitness, expectedFitness),
    `${at}.fitness does not match its search objective`,
  );
}

function checkStatsPartition(entries, target, at) {
  const countFields = [
    "pairs",
    "completePairs",
    ...capRateCountFields.map(([countName]) => countName),
    ...ordinaryRateCountFields.map(([countName]) => countName),
  ];
  for (const field of countFields) {
    check(
      entries.reduce((sum, [, stats]) => sum + stats[field], 0) === target[field],
      `${at} ${field} partition does not close`,
    );
  }
  function checkMomentPartition(getMoment, targetMoment, label) {
    const moments = entries.map(([, stats]) => getMoment(stats));
    const samples = moments.reduce((sum, moment) => sum + moment.samples, 0);
    check(samples === targetMoment.samples, `${at} ${label} sample partition does not close`);
    if (samples === 0) {
      check(
        targetMoment.mean === 0 && targetMoment.min === 0 && targetMoment.max === 0,
        `${at} ${label} zero-sample target moment is invalid`,
      );
      return;
    }
    const weightedMean =
      moments.reduce((sum, moment) => sum + moment.mean * moment.samples, 0) / samples;
    const populated = moments.filter((moment) => moment.samples > 0);
    const expectedMin = Math.min(...populated.map((moment) => moment.min));
    const expectedMax = Math.max(...populated.map((moment) => moment.max));
    check(
      closeEnough(weightedMean, targetMoment.mean),
      `${at} ${label} weighted mean partition does not close`,
    );
    check(
      closeEnough(expectedMin, targetMoment.min) && closeEnough(expectedMax, targetMoment.max),
      `${at} ${label} extrema partition does not close`,
    );
  }
  checkMomentPartition((stats) => stats.utility, target.utility, "utility");
  for (const metric of metricNames) {
    checkMomentPartition(
      (stats) => stats.metrics[metric],
      target.metrics[metric],
      metric,
    );
  }
  for (const scoreName of ["maxOutcomeScore", "maxMechanicalScore"]) {
    const expectedMaximum = Math.max(...entries.map(([, stats]) => stats[scoreName]));
    check(
      closeEnough(expectedMaximum, target[scoreName]),
      `${at} ${scoreName} partition does not close`,
    );
  }
}

function expectedClassification(perk) {
  const broad = perk.broad;
  const targeted = perk.targeted;
  const full = perk.fullWaveValidation;
  if (broad.completePairs === 0) return insufficientCategory;

  const outcomeUpper = wilson95(broad.outcomeChanged, broad.completePairs)[1];
  const capTransitionUpper = wilson95(broad.capTransition, broad.pairs)[1];
  const allMechanical =
    broad.mechanicalChanged + targeted.mechanicalChanged + full.mechanicalChanged;
  const allOutcome = broad.outcomeChanged + targeted.outcomeChanged + full.outcomeChanged;
  const allCapTransitions =
    broad.capTransition + targeted.capTransition + full.capTransition;
  const maxOutcome = Math.max(
    broad.maxOutcomeScore,
    targeted.maxOutcomeScore,
    full.maxOutcomeScore,
  );

  if (allMechanical === 0 && allOutcome === 0 && allCapTransitions === 0) {
    return "mechanically inactive";
  }
  if (broad.utility.ci95[1] < -0.1) return "harmful";
  if (outcomeUpper < 0.001 && maxOutcome < 1 && allCapTransitions === 0) {
    return "redundant";
  }
  if (
    (outcomeUpper < 0.01 && maxOutcome >= 1) ||
    (allCapTransitions > 0 && outcomeUpper < 0.01 && capTransitionUpper < 0.01)
  ) {
    return "niche-only";
  }
  return "meaningful";
}

function syntheticMoment(samples, value = 0) {
  return {
    samples,
    mean: value,
    min: value,
    max: value,
    ci95: [value, value],
  };
}

function syntheticEffectStats({ pairs = 1, bothCapped = 0, offOnly = 0, onOnly = 0 }) {
  const pairCensored = bothCapped + offOnly + onOnly;
  const completePairs = pairs - pairCensored;
  check(completePairs >= 0, "synthetic fixture has more censored pairs than pairs");
  const stats = {
    pairs,
    completePairs,
    pairCensored,
    bothCapped,
    offCapped: bothCapped + offOnly,
    onCapped: bothCapped + onOnly,
    capTransition: offOnly + onOnly,
    exactZero: completePairs,
    outcomeChanged: 0,
    mechanicalChanged: 0,
    positive: 0,
    negative: 0,
    pathologicalStall: 0,
    maxOutcomeScore: 0,
    maxMechanicalScore: 0,
    maxOutcomeContext: null,
    maxMechanicalContext: null,
    utility: syntheticMoment(completePairs),
    metrics: Object.fromEntries(
      ordinaryMetricNames.map((metric) => [metric, syntheticMoment(completePairs)]),
    ),
  };
  for (const [countName, rateName, ciName] of capRateCountFields) {
    stats[rateName] = stats[countName] / pairs;
    stats[ciName] = wilson95(stats[countName], pairs);
  }
  for (const [countName, rateName, ciName] of ordinaryRateCountFields) {
    stats[rateName] = completePairs === 0 ? 0 : stats[countName] / completePairs;
    stats[ciName] = wilson95(stats[countName], completePairs);
  }
  stats.metrics.tickCapReached = tickCapMoment(pairs, offOnly, onOnly);
  return stats;
}

function runSyntheticEffectStatsChecks() {
  checkEffectStats(syntheticEffectStats({}), "synthetic.uncensored", 1);
  checkEffectStats(
    syntheticEffectStats({ offOnly: 1 }),
    "synthetic.oneArmOffCapped",
    1,
  );
  checkEffectStats(
    syntheticEffectStats({ onOnly: 1 }),
    "synthetic.oneArmOnCapped",
    1,
  );
  checkEffectStats(
    syntheticEffectStats({ bothCapped: 1 }),
    "synthetic.bothArmsCapped",
    1,
  );
  checkEffectStats(
    syntheticEffectStats({ pairs: 5, bothCapped: 1, offOnly: 1, onOnly: 1 }),
    "synthetic.mixedCapDistribution",
    5,
  );
}

runSyntheticEffectStatsChecks();
if (selfTestOnly) {
  console.log("sensitivity checker synthetic fixtures valid");
  process.exit(0);
}

check(report.kind === "perk-sensitivity", "wrong report kind");
check(report.schema === 6, "wrong sensitivity schema");
check(/^(0|[1-9]\d*)$/.test(report.seed), "report seed is not an exact decimal string");
const gitRevisionPattern = /^(?!0{40}$)[0-9a-f]{40}$/;
check(gitRevisionPattern.test(report.sourceRevision), "source revision is not an exact nonzero Git commit");
check(
  gitRevisionPattern.test(report.sourceRevisionAtCompletion),
  "completion source revision is not an exact nonzero Git commit",
);
check(typeof report.sourceDirty === "boolean", "source dirty-state metadata is missing");
check(
  typeof report.sourceDirtyAtCompletion === "boolean",
  "completion dirty-state metadata is missing",
);
check(
  report.sourceRevisionAtCompletion === report.sourceRevision,
  "source revision changed during the sensitivity run",
);
check(
  report.sourceDirtyAtCompletion === report.sourceDirty,
  "source dirty state changed during the sensitivity run",
);
check(report.sourceStableAcrossRun === true, "source was not stable across the sensitivity run");
if (fullCorpus) {
  const currentRevision = execFileSync("git", ["rev-parse", "HEAD"], {
    cwd: repoRoot,
    encoding: "utf8",
  }).trim();
  const currentTrackedStatus = execFileSync(
    "git",
    ["status", "--porcelain", "--untracked-files=no"],
    { cwd: repoRoot, encoding: "utf8" },
  ).trim();
  check(report.sourceRevision === currentRevision, "full report start revision is not current HEAD");
  check(
    report.sourceRevisionAtCompletion === currentRevision,
    "full report completion revision is not current HEAD",
  );
  check(
    report.sourceDirty === false && report.sourceDirtyAtCompletion === false,
    "full sensitivity reports must start and finish from a clean source tree",
  );
  check(currentTrackedStatus === "", "current tracked worktree is dirty during full validation");
}
check(report.config?.gaElitesReevaluated === false, "GA must not count unchanged elites as independent samples");
check(report.config?.gaCensorAware === true, "GA must be censor-aware");
check(
  exactlyEqual(report.config?.gaObjectives, ["benefit", "harm", "mechanical"]),
  "GA objectives must separately cover benefit, harm, and mechanical activity",
);
check(
  report.config?.gaCappedCandidatesRankBelowComplete === true,
  "GA config must rank capped candidates below complete candidates",
);
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
check(
  report.execution?.censoredOrdinaryMetricsExcluded === true,
  "every ordinary metric must exclude censored pairs",
);
check(
  report.execution?.completePairMetricsOnly === true,
  "ordinary effects must use complete pairs only",
);
check(report.execution?.gaCensorAware === true, "execution metadata must mark the GA censor-aware");
check(
  report.execution?.gaCappedCandidatesRankBelowComplete === true,
  "capped GA candidates must rank below complete candidates",
);
check(
  report.tickCapMetadata?.ticksPerArm === report.execution.tickCap,
  "tick-cap metadata disagrees with execution metadata",
);
check(report.tickCapMetadata?.metric === "tickCapReached", "wrong tick-cap transition metric");
check(
  report.tickCapMetadata?.interpretation === "diagnostic-only",
  "tick-cap transitions must be marked diagnostic-only",
);
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
check(
  !Object.hasOwn(report.tickCapMetadata, "utilityWeight") &&
    !Object.hasOwn(report.tickCapMetadata, "outcomeScoreWeight"),
  "diagnostic-only cap transitions must not carry ordinary GA weights",
);
check(report.execution?.sharedPreInterventionPrefixes === true, "paired prefixes must be shared");
check(report.execution?.broadPrefixesSharedAcrossPerks === true, "broad prefixes must be shared across perks");
check(report.execution?.policyCheckpointsReused === true, "policy checkpoints must be reused");
check(report.execution?.naturalPolicyBaselineArmsReused === true, "identical natural policy arms must be reused");
check(report.execution?.compactSensitivityResults === true, "sensitivity runs must skip unused trace payloads");
check(typeof report.inferenceCaveat === "string" && report.inferenceCaveat.length > 0, "inference caveat is missing");
check(report.perks?.length === 12, "expected all 12 perks");
check(
  exactlyEqual(report.config?.lateScreenPerks, canonicalLateScreenPerks),
  "config.lateScreenPerks does not match the canonical ordered perk set",
);
check(
  exactlyEqual(report.config?.corpusDimensions, canonicalCorpusDimensions),
  "config.corpusDimensions does not match the canonical corpus",
);
for (const [index, [expectedId, expectedName]] of canonicalPerks.entries()) {
  check(
    report.perks[index]?.id === expectedId && report.perks[index]?.name === expectedName,
    `perks[${index}] does not match canonical id/name/order`,
  );
}

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
if (fullCorpus) {
  check(report.seed === "20260719", "full sensitivity corpus uses the wrong canonical seed");
  check(report.config.broadScenarios >= 100_000, "full broad corpus is below 100,000 scenarios");
  check(report.config.gaGenerations >= 34, "full genetic search has fewer than 34 generations");
  check(report.config.gaPopulation >= 2_048, "full genetic search population is below 2,048");
  check(
    report.config.fullWaveFinalistsPerPerk >= 64,
    "full-wave validation has fewer than 64 finalists per perk",
  );
  check(
    report.config.screenMaxWave === 25 && report.config.lateScreenWave === 50,
    "full corpus must use screen wave 25 and late-screen wave 50",
  );
}

const expectedBroadPairs = report.config.broadScenarios * report.perks.length;
const expectedTargetedPairs =
  report.config.gaGenerations *
  report.config.gaPopulation *
  report.perks.length *
  report.config.gaObjectives.length;
const expectedFullWavePairs = report.config.fullWaveFinalistsPerPerk * report.perks.length;
check(report.counts.broadPairs === expectedBroadPairs, "broad-pair count does not match config");
check(report.counts.targetedPairs === expectedTargetedPairs, "targeted-pair count does not match config");
check(report.counts.fullWavePairs === expectedFullWavePairs, "full-wave-pair count does not match config");
check(
  report.counts.pairedComparisons ===
    expectedBroadPairs + expectedTargetedPairs + expectedFullWavePairs,
  "paired-comparison phase accounting does not close",
);
for (const field of ["completePairedComparisons", "censoredPairedComparisons"]) {
  check(
    Number.isSafeInteger(report.counts[field]) && report.counts[field] >= 0,
    `counts.${field} is invalid`,
  );
}
check(
  report.counts.completePairedComparisons + report.counts.censoredPairedComparisons ===
    report.counts.pairedComparisons,
  "complete and censored paired-comparison counts do not close",
);
check(
  report.counts.completePairedComparisons >= minimumPairs,
  "complete paired corpus is below the requested minimum",
);
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
check(report.releaseGates && typeof report.releaseGates === "object", "release gates are missing");
check(
  Number.isSafeInteger(report.releaseGates.pathologicalStallPairs) &&
    report.releaseGates.pathologicalStallPairs >= 0,
  "pathological-stall gate count is invalid",
);
for (const key of [
  "minimumThreeMillionPairs",
  "completePairRateAtLeast99_5Percent",
  "noPathologicalStalls",
  "separateBenefitHarmMechanicalSearches",
  "commonRandomNumbers",
  "exactPairedInterventions",
  "relevantContextStrata",
]) {
  check(typeof report.releaseGates.checks?.[key] === "boolean", `release gate ${key} is missing`);
}
check(
  report.releaseGates.checks.noPathologicalStalls ===
    (report.releaseGates.pathologicalStallPairs === 0),
  "pathological-stall release gate disagrees with its count",
);
check(
  report.releaseGates.pass === Object.values(report.releaseGates.checks).every(Boolean),
  "release-gate pass does not equal the conjunction of checks",
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
  check(
    categories.has(perk.classification) || perk.classification === insufficientCategory,
    `${perk.name} has an invalid classification`,
  );
  const targetedWave = report.config.lateScreenPerks.includes(perk.name)
    ? report.config.lateScreenWave
    : report.config.screenMaxWave;
  const expectedPairsByPhase = {
    broad: report.config.broadScenarios,
    targeted:
      report.config.gaGenerations *
      report.config.gaPopulation *
      report.config.gaObjectives.length,
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
      checkSensitivityContext(
        stats.maxOutcomeContext,
        `${perk.name}.${phase}.maxOutcomeContext`,
        expectedWaveByPhase[phase],
      );
    }
    if (stats.maxMechanicalScore > 0) {
      checkSensitivityContext(
        stats.maxMechanicalContext,
        `${perk.name}.${phase}.maxMechanicalContext`,
        expectedWaveByPhase[phase],
      );
    }
  }
  for (const [field, target, expectedPerObjective, expectedWave] of [
    [
      "targetedByObjective",
      perk.targeted,
      report.config.gaGenerations * report.config.gaPopulation,
      targetedWave,
    ],
    [
      "fullWaveByObjective",
      perk.fullWaveValidation,
      report.config.fullWaveFinalistsPerPerk / report.config.gaObjectives.length,
      report.config.lateScreenWave,
    ],
  ]) {
    const entries = Object.entries(perk[field] || {});
    check(
      exactlyEqual(entries.map(([objective]) => objective).sort(), [...report.config.gaObjectives].sort()),
      `${perk.name}.${field} does not contain the canonical objectives`,
    );
    for (const [objective, stats] of entries) {
      checkEffectStats(stats, `${perk.name}.${field}.${objective}`, expectedPerObjective);
      for (const [scoreName, contextName] of [
        ["maxOutcomeScore", "maxOutcomeContext"],
        ["maxMechanicalScore", "maxMechanicalContext"],
      ]) {
        if (stats[scoreName] > 0) {
          checkSensitivityContext(
            stats[contextName],
            `${perk.name}.${field}.${objective}.${contextName}`,
            expectedWave,
          );
        }
      }
    }
    checkStatsPartition(entries, target, `${perk.name}.${field}`);
  }
  for (const [field, statsField, expectedWave] of [
    ["bestExamplesByObjective", "targetedByObjective", targetedWave],
    ["bestFullWaveExamplesByObjective", "fullWaveByObjective", report.config.lateScreenWave],
  ]) {
    const examples = perk[field] || {};
    check(
      exactlyEqual(Object.keys(examples).sort(), [...report.config.gaObjectives].sort()),
      `${perk.name}.${field} does not contain the canonical objectives`,
    );
    for (const objective of report.config.gaObjectives) {
      const stats = perk[statsField][objective];
      if (stats.completePairs === 0) {
        check(examples[objective] == null, `${perk.name}.${field}.${objective} must be absent without a complete pair`);
      } else {
        checkExample(examples[objective], `${perk.name}.${field}.${objective}`, expectedWave);
        check(examples[objective].searchObjective === objective, `${perk.name}.${field}.${objective} objective mismatch`);
      }
    }
  }
  const strata = Object.entries(perk.contextStrata || {});
  for (const [key, stats] of strata) {
    check(
      key.startsWith("composition:") || key.startsWith("mechanic:"),
      `${perk.name}.contextStrata contains unknown key ${key}`,
    );
    checkEffectStats(stats, `${perk.name}.contextStrata.${key}`);
  }
  for (const prefix of ["composition", "mechanic"]) {
    const entries = strata.filter(([key]) => key.startsWith(`${prefix}:`));
    check(entries.length > 0, `${perk.name} is missing ${prefix} strata`);
    checkStatsPartition(entries, perk.broad, `${perk.name}.${prefix} strata`);
  }
  check(
    perk.classification === expectedClassification(perk),
    `${perk.name} classification does not match its complete-pair and cap-transition evidence`,
  );
  if (fullCorpus) {
    for (const phase of ["broad", "targeted", "fullWaveValidation"]) {
      check(
        perk[phase].completePairs > 0,
        `${perk.name}.${phase} has no complete pair in a full run`,
      );
    }
    check(
      categories.has(perk.classification),
      `${perk.name} lacks one of the five requested effect classifications in a full run`,
    );
  }
  if (perk.targeted.completePairs === 0) {
    check(
      perk.bestAdversarialExample == null,
      `${perk.name}.bestAdversarialExample must be absent without a complete targeted pair`,
    );
  } else {
    checkExample(perk.bestAdversarialExample, `${perk.name}.bestAdversarialExample`, targetedWave);
  }
  if (perk.fullWaveValidation.completePairs === 0) {
    check(
      perk.bestFullWaveExample == null,
      `${perk.name}.bestFullWaveExample must be absent without a complete full-wave pair`,
    );
  } else {
    checkExample(
      perk.bestFullWaveExample,
      `${perk.name}.bestFullWaveExample`,
      report.config.lateScreenWave,
    );
  }
}

const allPerkPhaseStats = report.perks.flatMap((perk) => [
  perk.broad,
  perk.targeted,
  perk.fullWaveValidation,
]);
check(
  allPerkPhaseStats.reduce((sum, stats) => sum + stats.completePairs, 0) ===
    report.counts.completePairedComparisons,
  "counts.completePairedComparisons does not match all perk phases",
);
check(
  allPerkPhaseStats.reduce((sum, stats) => sum + stats.pairCensored, 0) ===
    report.counts.censoredPairedComparisons,
  "counts.censoredPairedComparisons does not match all perk phases",
);
check(
  allPerkPhaseStats.reduce((sum, stats) => sum + stats.pathologicalStall, 0) ===
    report.releaseGates.pathologicalStallPairs,
  "release-gate pathological-stall count does not match all perk phases",
);
if (fullCorpus) {
  check(report.releaseGates.pass === true, "full sensitivity corpus did not pass every release gate");
}

const interactions = report.projectileSpeedInteractions || {};
const interactionKeys = Object.keys(interactions);
for (const [key, stats] of Object.entries(interactions)) {
  checkEffectStats(stats, `projectileSpeedInteractions.${key}`);
  for (const [scoreName, contextName] of [
    ["maxOutcomeScore", "maxOutcomeContext"],
    ["maxMechanicalScore", "maxMechanicalContext"],
  ]) {
    if (stats[scoreName] > 0) {
      checkSensitivityContext(
        stats[contextName],
        `projectileSpeedInteractions.${key}.${contextName}`,
        report.config.lateScreenWave,
      );
    }
  }
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
const allowedDirectInteractionKeys = new Set(
  Object.entries(expectedInteractionLabels).flatMap(([prefix, labels]) =>
    labels.map((label) => `${prefix}:${label}`),
  ),
);
const jointComponentsByKey = new Map();
for (const key of interactionKeys) {
  if (!key.startsWith("joint:")) {
    check(allowedDirectInteractionKeys.has(key), `unknown projectile-speed interaction key ${key}`);
    continue;
  }
  const components = key.slice("joint:".length).split("|");
  check(
    components.length === canonicalJointPrefixes.length,
    `${key} does not have exactly ${canonicalJointPrefixes.length} joint components`,
  );
  const parsed = {};
  for (const [index, component] of components.entries()) {
    const separator = component.indexOf(":");
    check(
      separator > 0 && separator === component.lastIndexOf(":"),
      `${key} has a malformed joint component at index ${index}`,
    );
    const prefix = component.slice(0, separator);
    const label = component.slice(separator + 1);
    check(
      prefix === canonicalJointPrefixes[index],
      `${key} has ${prefix} at joint index ${index}; expected ${canonicalJointPrefixes[index]}`,
    );
    check(
      expectedInteractionLabels[prefix]?.includes(label),
      `${key} has invalid ${prefix} label ${label}`,
    );
    parsed[prefix] = label;
  }
  jointComponentsByKey.set(key, parsed);
}
for (const [prefix, allowedLabels] of Object.entries(expectedInteractionLabels)) {
  const entries = Object.entries(interactions).filter(([key]) => key.startsWith(`${prefix}:`));
  check(entries.length > 0, `missing projectile-speed interaction ${prefix}:`);
  const observedLabels = entries.map(([key]) => key.slice(prefix.length + 1));
  check(
    observedLabels.every((label) => allowedLabels.includes(label)),
    `projectile-speed interaction ${prefix}: has an unexpected stratum`,
  );
  checkStatsPartition(
    entries,
    report.perks[8].broad,
    `projectile-speed interaction ${prefix}:`,
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
checkStatsPartition(jointEntries, report.perks[8].broad, "joint projectile-speed interaction");
for (const [prefix, allowedLabels] of Object.entries(expectedInteractionLabels)) {
  for (const label of allowedLabels) {
    const directKey = `${prefix}:${label}`;
    const directStats = interactions[directKey];
    const matchingJointEntries = jointEntries.filter(
      ([key]) => jointComponentsByKey.get(key)?.[prefix] === label,
    );
    if (directStats) {
      check(matchingJointEntries.length > 0, `${directKey} has no matching joint cells`);
      checkStatsPartition(matchingJointEntries, directStats, `joint marginal ${directKey}`);
    } else {
      check(
        matchingJointEntries.length === 0,
        `joint cells exist for missing direct marginal ${directKey}`,
      );
    }
  }
}

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
