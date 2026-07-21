use epoch_core::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    env, fs,
    io::{self, Read, Write},
    process::Command,
    time::Instant,
};

fn read_input(path: Option<&String>) -> String {
    if let Some(p) = path {
        fs::read_to_string(p).unwrap()
    } else {
        let mut s = String::new();
        io::stdin().read_to_string(&mut s).unwrap();
        s
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("run") => {
            let input: Input = serde_json::from_str(&read_input(args.get(2))).unwrap();
            println!(
                "{}",
                serde_json::to_string(&run(
                    &input,
                    input
                        .params
                        .as_ref()
                        .map(ParamsPatch::apply)
                        .unwrap_or_default()
                ))
                .unwrap()
            )
        }
        Some("debug") => {
            let input: Input = serde_json::from_str(&read_input(args.get(2))).unwrap();
            let wave = args.get(3).and_then(|x| x.parse().ok()).unwrap_or(7);
            println!(
                "{}",
                serde_json::to_string(&debug_wave(
                    &input,
                    wave,
                    input
                        .params
                        .as_ref()
                        .map(ParamsPatch::apply)
                        .unwrap_or_default()
                ))
                .unwrap()
            );
        }
        Some("golden") => golden(args.get(2)),
        Some("policy-golden") => policy_golden(&args),
        Some("attack") => attack(&args),
        Some("defend") => defend(&args),
        Some("dashboard") => dashboard(args.get(2)),
        Some("bench") => bench(&args),
        Some("batch") => batch(args.get(2)),
        Some("calibrate") => calibrate(args.get(2)),
        Some("sensitivity") => sensitivity(&args),
        Some("sensitivity-replay") => sensitivity_replay(&args),
        _ => help(),
    }
}
fn help() {
    eprintln!(
        "epoch-lab <run [case.json]|batch [jobs.json]|calibrate [traces.json]|golden [cases.json]|policy-golden [cases seed]|attack [generations population seed calibration.json]|defend [generations candidates policies seed calibration.json]|sensitivity [broad-scenarios ga-generations ga-population seed max-wave output.json]|sensitivity-replay [report.json perk field]|dashboard [report.json]|bench>"
    )
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchJob {
    seed: u64,
    #[serde(default)]
    map: usize,
    #[serde(default = "batch_fifty")]
    max_wave: usize,
    genome: Genome,
    #[serde(default)]
    params: Option<ParamsPatch>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchResult {
    index: usize,
    seed: u64,
    wave: usize,
    lives: i32,
    gold: f64,
    actions: Vec<Action>,
    fusion_counts: [usize; 6],
    doctrine_counts: [usize; 12],
    power_counts: [usize; 3],
    early_calls: usize,
}
fn batch_fifty() -> usize {
    50
}
fn batch(path: Option<&String>) {
    let jobs: Vec<BatchJob> = serde_json::from_str(&read_input(path)).unwrap();
    let results: Vec<_> = jobs
        .par_iter()
        .enumerate()
        .map(|(index, job)| {
            let run = run_policy(
                &job.genome,
                job.seed,
                job.map,
                job.max_wave,
                job.params
                    .as_ref()
                    .map(ParamsPatch::apply)
                    .unwrap_or_default(),
            );
            BatchResult {
                index,
                seed: job.seed,
                wave: run.run.result.wave,
                lives: run.run.result.lives,
                gold: run.run.result.gold,
                actions: run.actions,
                fusion_counts: run.fusion_counts,
                doctrine_counts: run.doctrine_counts,
                power_counts: run.power_counts,
                early_calls: run.early_calls,
            }
        })
        .collect();
    println!("{}", serde_json::to_string(&results).unwrap());
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HumanTrace {
    actions: Vec<Action>,
    duration_seconds: Option<f64>,
    result: Option<HumanResult>,
}
#[derive(Deserialize)]
struct HumanResult {
    wave: usize,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CalibrationPrior {
    sessions: usize,
    actions: usize,
    reaction_ticks: u32,
    apm: f64,
    tower_weights: [f64; 10],
    doctrine_weights: [f64; 12],
    power_weights: [f64; 3],
    early_call_weight: f64,
    support_adjacency_weight: f64,
}
fn calibrate(path: Option<&String>) {
    let traces: Vec<HumanTrace> = serde_json::from_str(&read_input(path)).unwrap();
    assert!(
        !traces.is_empty(),
        "calibration needs at least one browser playtrace"
    );
    let mut tower = [0usize; 10];
    let mut doctrine = [0usize; 12];
    let mut power = [0usize; 3];
    let mut first_ticks = BTreeMap::<(usize, usize), u32>::new();
    let mut actions = 0usize;
    let mut early = 0usize;
    let mut support_adjacent = 0usize;
    let mut placements = 0usize;
    let mut wave_opportunities = 0usize;
    let mut seconds = 0.;
    for (session, trace) in traces.iter().enumerate() {
        wave_opportunities += trace
            .result
            .as_ref()
            .map(|result| result.wave)
            .unwrap_or_else(|| {
                trace
                    .actions
                    .iter()
                    .map(|action| action.wave)
                    .max()
                    .unwrap_or(0)
            });
        seconds += trace.duration_seconds.unwrap_or_else(|| {
            trace
                .actions
                .iter()
                .filter_map(|a| a.tick)
                .max()
                .unwrap_or(0) as f64
                / 30.
        });
        let mut placed = Vec::<(usize, i32, i32)>::new();
        for action in &trace.actions {
            actions += 1;
            if let Some(tick) = action.tick {
                first_ticks
                    .entry((session, action.wave))
                    .and_modify(|v| *v = (*v).min(tick))
                    .or_insert(tick);
            }
            match action.op.as_str() {
                "place" => {
                    if action.tower < 10 {
                        tower[action.tower] += 1
                    }
                    placements += 1;
                    if placed.iter().any(|(kind, x, y)| {
                        [3, 7, 8].contains(kind)
                            && (x - action.x).abs().max((y - action.y).abs()) == 1
                    }) {
                        support_adjacent += 1
                    }
                    placed.push((action.tower, action.x, action.y));
                }
                "relic" => {
                    if action.tower < 12 {
                        doctrine[action.tower] += 1
                    }
                }
                "power" => {
                    if action.tower < 3 {
                        power[action.tower] += 1
                    }
                }
                "early" => early += 1,
                _ => {}
            }
        }
    }
    let mut ticks = first_ticks.into_values().collect::<Vec<_>>();
    ticks.sort_unstable();
    let reaction_ticks = ticks.get(ticks.len() / 2).copied().unwrap_or(15);
    let normalize = |counts: &[usize]| {
        let max = counts.iter().copied().max().unwrap_or(1).max(1) as f64;
        counts
            .iter()
            .map(|&n| n as f64 / max * 2. - 1.)
            .collect::<Vec<_>>()
    };
    let tw = normalize(&tower);
    let dw = normalize(&doctrine);
    let pw = normalize(&power);
    let prior = CalibrationPrior {
        sessions: traces.len(),
        actions,
        reaction_ticks,
        apm: (actions as f64 / (seconds.max(1.) / 60.)).clamp(10., 180.),
        tower_weights: std::array::from_fn(|i| tw[i]),
        doctrine_weights: std::array::from_fn(|i| dw[i]),
        power_weights: std::array::from_fn(|i| pw[i]),
        early_call_weight: if wave_opportunities > 0 {
            (early as f64 / wave_opportunities as f64 * 2. - 1.).clamp(-1., 1.)
        } else {
            -1.
        },
        support_adjacency_weight: if placements > 0 {
            support_adjacent as f64 / placements as f64
        } else {
            0.
        },
    };
    fs::create_dir_all("balance-lab/out").unwrap();
    fs::write(
        "balance-lab/out/calibration.json",
        serde_json::to_vec_pretty(&prior).unwrap(),
    )
    .unwrap();
    println!("balance-lab/out/calibration.json");
}

fn seed_from_prior(genome: &mut Genome, prior: &CalibrationPrior) {
    genome.tower_weights = prior.tower_weights;
    genome.doctrine_weights = prior.doctrine_weights;
    genome.power_weights = prior.power_weights;
    genome.early_call_weight = prior.early_call_weight;
    genome.reaction_ticks = prior.reaction_ticks;
    genome.apm = prior.apm;
    genome.placement[4] = prior.support_adjacency_weight;
}

fn golden(path: Option<&String>) {
    let raw = read_input(path);
    let inputs: Vec<Input> = serde_json::from_str(&raw).unwrap();
    golden_inputs(&inputs, &raw);
}

fn policy_golden(args: &[String]) {
    let count = args.get(2).and_then(|x| x.parse().ok()).unwrap_or(64);
    let seed = args.get(3).and_then(|x| x.parse().ok()).unwrap_or(4242);
    let mut rng = Xoshiro::new(seed);
    let inputs: Vec<_> = (0..count)
        .map(|i| {
            let case_seed = seed + i as u64 * 104729;
            let genome = Genome::random(&mut rng);
            let policy = run_policy(
                &genome,
                case_seed,
                (case_seed % 3) as usize,
                50,
                Params::default(),
            );
            Input {
                seed: case_seed,
                map: (case_seed % 3) as usize,
                max_wave: 50,
                actions: policy.actions,
                params: None,
                initial: None,
                endless: false,
            }
        })
        .collect();
    let raw = serde_json::to_string(&inputs).unwrap();
    golden_inputs(&inputs, &raw);
}

fn golden_inputs(inputs: &[Input], _raw: &str) {
    let root = env!("CARGO_MANIFEST_DIR").trim_end_matches("/balance-lab");
    let mut differences = vec![];
    let mut wave_traces = 0usize;
    for (batch, chunk) in inputs.chunks(64).enumerate() {
        let rust: Vec<_> = chunk
            .iter()
            .map(|input| {
                run(
                    input,
                    input
                        .params
                        .as_ref()
                        .map(ParamsPatch::apply)
                        .unwrap_or_default(),
                )
            })
            .collect();
        wave_traces += rust.iter().map(|run| run.trace.len()).sum::<usize>();
        let mut oracle_input = serde_json::to_value(chunk).unwrap();
        strip_nulls(&mut oracle_input);
        let raw = serde_json::to_vec(&oracle_input).unwrap();
        let mut child = Command::new("node")
            .arg(format!("{root}/oracle.js"))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&raw).unwrap();
        let out = child.wait_with_output().unwrap();
        if !out.status.success() {
            panic!("JS oracle failed")
        };
        let js: Vec<Value> = serde_json::from_slice(&out.stdout).unwrap();
        for (i, (r, j)) in rust.iter().zip(js.iter()).enumerate() {
            let i = batch * 64 + i;
            if r.rejected.len() != j["rejected"].as_array().unwrap().len()
                || r.pending != j["pending"].as_u64().unwrap() as usize
            {
                differences.push(format!(
                    "case {i}: action state differs (rejected {}/{}, pending {}/{})",
                    r.rejected.len(),
                    j["rejected"].as_array().unwrap().len(),
                    r.pending,
                    j["pending"]
                ));
                continue;
            }
            let jr = &j["trace"];
            if r.trace.len() != jr.as_array().unwrap().len() {
                differences.push(format!(
                    "case {i}: trace length {} != {}",
                    r.trace.len(),
                    jr.as_array().unwrap().len()
                ));
                continue;
            }
            for (w, (rr, jj)) in r.trace.iter().zip(jr.as_array().unwrap()).enumerate() {
                let gold = (jj["gold"].as_f64().unwrap() - rr.gold).abs();
                let seconds = (jj["seconds"].as_f64().unwrap() - rr.seconds).abs();
                let jt = jj["towerDamage"].as_array().unwrap();
                let tower_diff = rr.tower_damage.len() != jt.len()
                    || rr.tower_damage.iter().zip(jt).any(|(a, b)| {
                        a.x != b["x"].as_i64().unwrap() as i32
                            || a.y != b["y"].as_i64().unwrap() as i32
                            || a.tower != b["tower"].as_u64().unwrap() as usize
                            || a.level != b["level"].as_u64().unwrap() as u8
                            || a.branch != b["branch"].as_u64().unwrap() as u8
                            || a.fusion.map(|x| x as u64) != b["fusion"].as_u64()
                            || (a.damage - b["damage"].as_f64().unwrap()).abs() > 1e-4
                    });
                if rr.lives != jj["lives"].as_i64().unwrap() as i32
                    || rr.kills != jj["kills"].as_u64().unwrap()
                    || rr.won != jj["won"].as_bool().unwrap()
                    || rr.over != jj["over"].as_bool().unwrap()
                    || gold > 1e-4
                    || seconds > 1e-4
                    || tower_diff
                {
                    differences.push(format!("case {i} wave {}: rust lives={} gold={} kills={} t={}, js lives={} gold={} kills={} t={}",w+1,rr.lives,rr.gold,rr.kills,rr.seconds,jj["lives"],jj["gold"],jj["kills"],jj["seconds"]));
                    break;
                }
            }
        }
    }
    if differences.is_empty() {
        println!(
            "golden: PASS ({} cases, {} wave traces)",
            inputs.len(),
            wave_traces
        )
    } else {
        for d in differences.iter().take(20) {
            eprintln!("{d}")
        }
        panic!("golden: FAIL ({} cases differ)", differences.len())
    }
}

fn strip_nulls(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|_, value| !value.is_null());
            for value in map.values_mut() {
                strip_nulls(value)
            }
        }
        Value::Array(values) => {
            for value in values {
                strip_nulls(value)
            }
        }
        _ => {}
    }
}

#[derive(Clone)]
struct DiagonalCma {
    mean: Vec<f64>,
    variance: Vec<f64>,
    path: Vec<f64>,
    sigma: f64,
}

impl DiagonalCma {
    fn new(genome: &Genome) -> Self {
        let mean = genome.to_vector();
        let mut variance = vec![1.; mean.len()];
        for value in &mut variance[39..42] {
            *value = 10000.;
        }
        variance[43] = 40000.;
        variance[44] = 100.;
        variance[45] = 400.;
        Self {
            path: vec![0.; mean.len()],
            mean,
            variance,
            sigma: 0.35,
        }
    }

    fn update(&mut self, samples: &mut [(f64, Genome)]) {
        samples.sort_by(|a, b| b.0.total_cmp(&a.0));
        let mu = (samples.len() / 2).max(1);
        let mut weights: Vec<_> = (0..mu)
            .map(|rank| (mu as f64 + 0.5).ln() - (rank as f64 + 1.).ln())
            .collect();
        let weight_sum = weights.iter().sum::<f64>();
        for weight in &mut weights {
            *weight /= weight_sum;
        }
        let old_mean = self.mean.clone();
        let vectors: Vec<_> = samples[..mu]
            .iter()
            .map(|(_, genome)| genome.to_vector())
            .collect();
        for dimension in 0..self.mean.len() {
            self.mean[dimension] = vectors
                .iter()
                .zip(&weights)
                .map(|(vector, weight)| vector[dimension] * weight)
                .sum();
            let scale = self.variance[dimension].sqrt().max(1e-9);
            let step = (self.mean[dimension] - old_mean[dimension]) / (self.sigma * scale);
            self.path[dimension] = 0.8 * self.path[dimension] + 0.2 * step;
            let spread = vectors
                .iter()
                .zip(&weights)
                .map(|(vector, weight)| {
                    let delta = (vector[dimension] - old_mean[dimension]) / self.sigma.max(1e-9);
                    delta * delta * weight
                })
                .sum::<f64>();
            self.variance[dimension] =
                (0.9 * self.variance[dimension] + 0.1 * spread).clamp(1e-6, 1e8);
        }
        let normalized_path = (self.path.iter().map(|value| value * value).sum::<f64>()
            / self.path.len() as f64)
            .sqrt();
        self.sigma = (self.sigma * ((normalized_path - 1.) * 0.12).exp()).clamp(0.03, 1.5);
    }

    fn sample(&self, rng: &mut Xoshiro) -> Genome {
        let vector: Vec<_> = self
            .mean
            .iter()
            .zip(&self.variance)
            .map(|(&mean, &variance)| mean + self.sigma * variance.sqrt() * normal(rng))
            .collect();
        Genome::from_vector(&vector)
    }
}

fn normal(rng: &mut Xoshiro) -> f64 {
    let u1 = rng.next_f64().max(f64::MIN_POSITIVE);
    let u2 = rng.next_f64();
    (-2. * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}
fn attack(args: &[String]) {
    let generations = args.get(2).and_then(|x| x.parse().ok()).unwrap_or(20);
    let population = args.get(3).and_then(|x| x.parse().ok()).unwrap_or(256);
    let seed = args.get(4).and_then(|x| x.parse().ok()).unwrap_or(1);
    let mut rng = Xoshiro::new(seed);
    let mut genomes: Vec<_> = (0..population).map(|_| Genome::random(&mut rng)).collect();
    let calibration: Option<CalibrationPrior> = args
        .get(5)
        .map(|path| serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap());
    if let Some(prior) = &calibration {
        for genome in genomes.iter_mut().take((population / 4).max(1)) {
            seed_from_prior(genome, prior)
        }
    }
    let started = Instant::now();
    let mut archive = Archive::new();
    let mut cma_states: BTreeMap<(usize, bool, Option<usize>), DiagonalCma> = BTreeMap::new();
    let mut history = vec![];
    for generation in 0..generations {
        let evaluated: Vec<_> = genomes
            .par_iter()
            .enumerate()
            .map(|(i, g)| {
                let s = seed + generation as u64 * population as u64 + i as u64;
                let pr = run_policy(
                    g,
                    s,
                    (s % 3) as usize,
                    if generation + 1 == generations {
                        50
                    } else {
                        25
                    },
                    Params::default(),
                );
                let fitness = pr.run.result.wave as f64 * 1000.
                    + pr.run.result.lives as f64 * 10.
                    + pr.run.result.gold * 0.01;
                let mono = (0..10)
                    .max_by(|&a, &b| g.tower_weights[a].total_cmp(&g.tower_weights[b]))
                    .unwrap();
                let niche = (
                    mono,
                    pr.merges == 0,
                    pr.fusion_counts.iter().position(|&count| count > 0),
                );
                (g.clone(), s, pr, fitness, niche)
            })
            .collect();
        let mut generation_fitness: Vec<f64> = evaluated.iter().map(|x| x.3).collect();
        generation_fitness.sort_by(f64::total_cmp);
        let generation_median = generation_fitness[generation_fitness.len() / 2];
        let mut cma_samples: BTreeMap<_, Vec<(f64, Genome)>> = BTreeMap::new();
        for (genome, _, _, fitness, niche) in &evaluated {
            cma_samples
                .entry(*niche)
                .or_default()
                .push((*fitness, genome.clone()));
        }
        for (niche, mut samples) in cma_samples {
            cma_states
                .entry(niche)
                .or_insert_with(|| DiagonalCma::new(&samples[0].1))
                .update(&mut samples);
        }
        for (g, s, pr, fitness, niche) in evaluated {
            let mut tower_damage = [0.; 10];
            for tower in &pr.run.result.tower_damage {
                tower_damage[tower.tower] += tower.damage;
            }
            let e = Elite {
                fitness,
                wave: pr.run.result.wave,
                lives: pr.run.result.lives,
                mono_tower: niche.0,
                no_merge: pr.merges == 0,
                fusion: pr.fusion_counts.iter().position(|&n| n > 0),
                genome: g,
                seed: s,
                actions: pr.actions,
                clear_times: pr.run.trace.iter().map(|x| x.seconds).collect(),
                tower_counts: pr.tower_counts,
                branch_counts: pr.branch_counts,
                fusion_counts: pr.fusion_counts,
                doctrine_counts: pr.doctrine_counts,
                power_counts: pr.power_counts,
                early_calls: pr.early_calls,
                tower_damage,
            };
            archive
                .entry(niche)
                .and_modify(|old| {
                    if e.fitness > old.fitness {
                        *old = e.clone()
                    }
                })
                .or_insert(e);
        }
        let best = archive
            .values()
            .map(|e| e.fitness)
            .fold(f64::NEG_INFINITY, f64::max);
        let mean_sigma = cma_states.values().map(|state| state.sigma).sum::<f64>()
            / cma_states.len().max(1) as f64;
        history.push(json!({"generation":generation,"bestFitness":best,"medianFitness":generation_median,"exploitability":best-generation_median,"viableCells":archive.len(),"cmaCells":cma_states.len(),"meanSigma":mean_sigma}));
        eprintln!(
            "generation {generation}: best {best:0.1}, cells {}",
            archive.len()
        );
        for (niche, elite) in &archive {
            cma_states
                .entry(*niche)
                .or_insert_with(|| DiagonalCma::new(&elite.genome));
        }
        let niches: Vec<_> = archive.keys().copied().collect();
        genomes = (0..population)
            .map(|i| {
                if i % 10 == 0 {
                    Genome::random(&mut rng)
                } else {
                    cma_states[&niches[i % niches.len()]].sample(&mut rng)
                }
            })
            .collect();
    }
    let mut first_loss_histogram = vec![0usize; 51];
    let mut clear_sum = [0.; 50];
    let mut clear_sq = [0.; 50];
    let mut clear_n = [0usize; 50];
    let mut tower_picks = [0usize; 10];
    let mut tower_damage = [0.; 10];
    let mut branch_picks = [[0usize; 2]; 10];
    let mut fusion_picks = [0usize; 6];
    let mut doctrine_picks = [0usize; 12];
    let mut power_uses = [0usize; 3];
    let mut early_calls = 0usize;
    for elite in archive.values() {
        first_loss_histogram[elite.wave.min(50)] += 1;
        for (i, &v) in elite.clear_times.iter().enumerate() {
            clear_sum[i] += v;
            clear_sq[i] += v * v;
            clear_n[i] += 1
        }
        for i in 0..10 {
            tower_picks[i] += elite.tower_counts[i];
            tower_damage[i] += elite.tower_damage[i];
            branch_picks[i][0] += elite.branch_counts[i][0];
            branch_picks[i][1] += elite.branch_counts[i][1]
        }
        for (total, count) in fusion_picks.iter_mut().zip(elite.fusion_counts) {
            *total += count
        }
        for (total, count) in doctrine_picks.iter_mut().zip(elite.doctrine_counts) {
            *total += count
        }
        for (total, count) in power_uses.iter_mut().zip(elite.power_counts) {
            *total += count
        }
        early_calls += elite.early_calls;
    }
    let clear_time_curve:Vec<_>=(0..50).filter(|&i|clear_n[i]>0).map(|i|{let mean=clear_sum[i]/clear_n[i]as f64;json!({"wave":i+1,"mean":mean,"stddev":(clear_sq[i]/clear_n[i]as f64-mean*mean).max(0.).sqrt(),"samples":clear_n[i]})}).collect();
    let elapsed = started.elapsed().as_secs_f64();
    let report = json!({"schema":1,"kind":"attack","seed":seed,"generations":generations,"population":population,"calibration":calibration.as_ref().map(|p|json!({"sessions":p.sessions,"actions":p.actions})),"elapsedSeconds":elapsed,"threads":rayon::current_num_threads(),"coreHours":elapsed*rayon::current_num_threads() as f64/3600.,"games":generations*population,"gamesPerSecond":generations as f64*population as f64/elapsed,"history":history,"firstLossHistogram":first_loss_histogram,"clearTimeCurve":clear_time_curve,"towerPicks":tower_picks,"branchPicks":branch_picks,"fusionPicks":fusion_picks,"doctrinePicks":doctrine_picks,"powerUses":power_uses,"earlyCalls":early_calls,"towerDamage":tower_damage,"archive":archive.values().collect::<Vec<_>>()});
    fs::create_dir_all("balance-lab/out").unwrap();
    fs::write(
        "balance-lab/out/report.json",
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("balance-lab/out/report.json")
}
fn parameter_loss(params: &Params, policies: &[(Genome, u64)], max_wave: usize) -> (f64, Value) {
    let runs: Vec<_> = policies
        .par_iter()
        .map(|(g, s)| run_policy(g, *s, (*s % 3) as usize, max_wave, params.clone()))
        .collect();
    let mut waves: Vec<f64> = runs.iter().map(|r| r.run.result.wave as f64).collect();
    waves.sort_by(f64::total_cmp);
    let median = waves[waves.len() / 2];
    let best = *waves.last().unwrap();
    let mean = waves.iter().sum::<f64>() / waves.len() as f64;
    let target = if max_wave < 28 { max_wave as f64 } else { 31. };
    let exploitability = best - median;
    let mut picks = [0usize; 10];
    for run in &runs {
        for (i, n) in run.tower_counts.iter().enumerate() {
            picks[i] += n;
        }
    }
    let total = picks.iter().sum::<usize>().max(1) as f64;
    let entropy = -picks
        .iter()
        .filter(|&&n| n > 0)
        .map(|&n| {
            let p = n as f64 / total;
            p * p.ln()
        })
        .sum::<f64>();
    let mut clear_sum = vec![0.; max_wave];
    let mut clear_n = vec![0usize; max_wave];
    for run in &runs {
        for (wave, trace) in run.run.trace.iter().enumerate() {
            clear_sum[wave] += trace.seconds;
            clear_n[wave] += 1;
        }
    }
    let clear_curve: Vec<_> = (0..max_wave)
        .filter(|&wave| clear_n[wave] > 0)
        .map(|wave| clear_sum[wave] / clear_n[wave] as f64)
        .collect();
    let curve_roughness = if clear_curve.len() < 3 {
        0.
    } else {
        clear_curve
            .windows(3)
            .map(|window| (window[2] - 2. * window[1] + window[0]).abs())
            .sum::<f64>()
            / (clear_curve.len() - 2) as f64
    };
    let loss = (mean - target).powi(2)
        + exploitability.powi(2) * 0.35
        + (2. - entropy).max(0.) * 20.
        + curve_roughness * 0.2;
    (
        loss,
        json!({"meanFirstLossWave":mean,"medianWave":median,"bestWave":best,"exploitability":exploitability,"towerEntropy":entropy,"towerPicks":picks,"curveRoughness":curve_roughness}),
    )
}

fn modifier_vector(modifier: &Modifiers, baseline: &Modifiers, out: &mut Vec<f64>) {
    for (value, base) in [
        modifier.dm,
        modifier.sp,
        modifier.pv,
        modifier.rgm,
        modifier.kb,
        modifier.bum,
        modifier.hc,
        modifier.sf,
        modifier.hr,
        modifier.rm,
        modifier.bf,
    ]
    .into_iter()
    .zip([
        baseline.dm,
        baseline.sp,
        baseline.pv,
        baseline.rgm,
        baseline.kb,
        baseline.bum,
        baseline.hc,
        baseline.sf,
        baseline.hr,
        baseline.rm,
        baseline.bf,
    ]) {
        if let Some(base) = base {
            out.push(value.unwrap_or(base) / base)
        }
    }
}

fn apply_modifier_vector(modifier: &mut Modifiers, values: &mut impl Iterator<Item = f64>) {
    macro_rules! apply {
        ($field:ident) => {
            if let Some(base) = modifier.$field {
                modifier.$field = Some(base * values.next().unwrap().clamp(0.45, 1.8));
            }
        };
    }
    apply!(dm);
    apply!(sp);
    apply!(pv);
    apply!(rgm);
    apply!(kb);
    apply!(bum);
    apply!(hc);
    apply!(sf);
    apply!(hr);
    apply!(rm);
    apply!(bf);
}

fn params_vector(params: &Params) -> Vec<f64> {
    let baseline = Params::default();
    let mut out = vec![
        params.constants.hp / baseline.constants.hp,
        params.constants.growth / baseline.constants.growth,
        params.constants.count / baseline.constants.count,
        params.constants.speed / baseline.constants.speed,
        params.constants.bounty / baseline.constants.bounty,
    ];
    for (tower, base) in params.towers.iter().zip(&baseline.towers) {
        for (value, base) in [
            tower.cost,
            tower.damage,
            tower.rate,
            tower.range,
            tower.splash,
            tower.knockback,
        ]
        .into_iter()
        .zip([
            base.cost,
            base.damage,
            base.rate,
            base.range,
            base.splash,
            base.knockback,
        ]) {
            if base != 0. {
                out.push(value / base)
            }
        }
        modifier_vector(&tower.branch1, &base.branch1, &mut out);
        modifier_vector(&tower.branch2, &base.branch2, &mut out);
    }
    for (fusion, base) in params.fusions.iter().zip(&baseline.fusions) {
        modifier_vector(&fusion.modifiers, &base.modifiers, &mut out);
    }
    // Every continuous rule, plus the integer-valued rules represented in a
    // continuous CMA space and rounded when materialized.
    macro_rules! r {
        ($field:ident) => {
            out.push(params.rules.$field / baseline.rules.$field)
        };
    }
    r!(upgrade_cost);
    r!(sell_refund);
    r!(doctrine_upgrade_cost);
    r!(doctrine_sell_refund);
    r!(doctrine_bounty);
    r!(doctrine_range);
    r!(doctrine_cooldown);
    r!(doctrine_knockback);
    r!(doctrine_boss_knockback);
    out.push(params.rules.doctrine_lives as f64 / baseline.rules.doctrine_lives as f64);
    r!(doctrine_projectile_speed);
    r!(doctrine_splash);
    r!(doctrine_irradiate);
    r!(doctrine_slow);
    r!(burn_cap);
    r!(doctrine_burn_cap);
    for i in 0..3 {
        out.push(params.rules.power_cooldowns[i] / baseline.rules.power_cooldowns[i])
    }
    r!(meteor_damage);
    r!(meteor_radius);
    r!(stasis_duration);
    r!(stasis_factor);
    r!(overdrive_duration);
    r!(overdrive_multiplier);
    out
}

fn params_from_vector(values: &[f64]) -> Params {
    let baseline = Params::default();
    let mut params = baseline.clone();
    let mut values = values.iter().copied();
    macro_rules! ratio {
        ($base:expr, $lo:expr, $hi:expr) => {
            $base * values.next().unwrap().clamp($lo, $hi)
        };
    }
    params.constants.hp = ratio!(baseline.constants.hp, 0.85, 1.2);
    params.constants.growth = ratio!(baseline.constants.growth, 0.97, 1.04).clamp(1.08, 1.16);
    params.constants.count = ratio!(baseline.constants.count, 0.8, 1.25).clamp(0.6, 1.);
    params.constants.speed = ratio!(baseline.constants.speed, 0.8, 1.25);
    params.constants.bounty = ratio!(baseline.constants.bounty, 0.75, 1.3);
    for (tower, base) in params.towers.iter_mut().zip(&baseline.towers) {
        macro_rules! tower_field {
            ($field:ident) => {
                if base.$field != 0. {
                    tower.$field = ratio!(base.$field, 0.55, 1.65);
                }
            };
        }
        tower_field!(cost);
        tower_field!(damage);
        tower_field!(rate);
        tower_field!(range);
        tower_field!(splash);
        tower_field!(knockback);
        apply_modifier_vector(&mut tower.branch1, &mut values);
        apply_modifier_vector(&mut tower.branch2, &mut values);
    }
    for fusion in &mut params.fusions {
        apply_modifier_vector(&mut fusion.modifiers, &mut values);
    }
    macro_rules! rule {
        ($field:ident, $lo:expr, $hi:expr) => {
            params.rules.$field = ratio!(baseline.rules.$field, $lo, $hi);
        };
    }
    rule!(upgrade_cost, 0.75, 1.25);
    rule!(sell_refund, 0.8, 1.25);
    rule!(doctrine_upgrade_cost, 0.75, 1.25);
    rule!(doctrine_sell_refund, 0.8, 1.15);
    rule!(doctrine_bounty, 0.5, 1.75);
    rule!(doctrine_range, 0.9, 1.15);
    rule!(doctrine_cooldown, 0.75, 1.2);
    rule!(doctrine_knockback, 0.7, 1.4);
    rule!(doctrine_boss_knockback, 0.65, 1.5);
    params.rules.doctrine_lives =
        ratio!(baseline.rules.doctrine_lives as f64, 0.67, 1.67).round() as i32;
    rule!(doctrine_projectile_speed, 0.8, 1.25);
    rule!(doctrine_splash, 0.8, 1.3);
    rule!(doctrine_irradiate, 0.65, 1.5);
    rule!(doctrine_slow, 0.75, 1.25);
    rule!(burn_cap, 0.8, 1.3);
    rule!(doctrine_burn_cap, 0.75, 1.35);
    for i in 0..3 {
        params.rules.power_cooldowns[i] = ratio!(baseline.rules.power_cooldowns[i], 0.7, 1.3);
    }
    // Unlock ages remain categorical product rules; all effect magnitudes are searched.
    rule!(meteor_damage, 0.65, 1.5);
    rule!(meteor_radius, 0.75, 1.35);
    rule!(stasis_duration, 0.7, 1.4);
    rule!(stasis_factor, 0.6, 1.5);
    rule!(overdrive_duration, 0.7, 1.4);
    rule!(overdrive_multiplier, 0.75, 1.3);
    debug_assert!(values.next().is_none());
    params
}

#[derive(Clone)]
struct ParamCma {
    mean: Vec<f64>,
    variance: Vec<f64>,
    path: Vec<f64>,
    sigma: f64,
}
impl ParamCma {
    fn new() -> Self {
        let mean = params_vector(&Params::default());
        Self {
            variance: vec![0.04; mean.len()],
            path: vec![0.; mean.len()],
            mean,
            sigma: 0.35,
        }
    }
    fn sample(&self, rng: &mut Xoshiro) -> Params {
        let vector = self
            .mean
            .iter()
            .zip(&self.variance)
            .map(|(&mean, &variance)| mean + self.sigma * variance.sqrt() * normal(rng))
            .collect::<Vec<_>>();
        params_from_vector(&vector)
    }
    fn update(&mut self, samples: &mut [(f64, Params)]) {
        samples.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mu = (samples.len() / 2).max(1);
        let mut weights = (0..mu)
            .map(|rank| (mu as f64 + 0.5).ln() - (rank as f64 + 1.).ln())
            .collect::<Vec<_>>();
        let sum = weights.iter().sum::<f64>();
        for weight in &mut weights {
            *weight /= sum
        }
        let vectors = samples[..mu]
            .iter()
            .map(|(_, p)| params_vector(p))
            .collect::<Vec<_>>();
        let old = self.mean.clone();
        for d in 0..self.mean.len() {
            self.mean[d] = vectors.iter().zip(&weights).map(|(v, w)| v[d] * w).sum();
            let delta = (self.mean[d] - old[d]) / self.variance[d].sqrt().max(1e-9);
            self.path[d] = 0.8 * self.path[d] + 0.2 * delta / self.sigma.max(1e-9);
            let spread = vectors
                .iter()
                .zip(&weights)
                .map(|(v, w)| w * (v[d] - self.mean[d]).powi(2))
                .sum::<f64>();
            self.variance[d] = (0.8 * self.variance[d] + 0.2 * spread.max(1e-5)).clamp(1e-5, 0.25);
        }
        let path_norm =
            (self.path.iter().map(|x| x * x).sum::<f64>() / self.path.len() as f64).sqrt();
        self.sigma = (self.sigma * (0.16 * (path_norm - 1.)).exp()).clamp(0.06, 0.8);
    }
}

fn refine_policies(
    policies: &mut Vec<(Genome, u64)>,
    params: &Params,
    rng: &mut Xoshiro,
    seed: u64,
) {
    let evaluated = policies
        .iter()
        .map(|(g, s)| {
            let run = run_policy(g, *s, (*s % 3) as usize, 25, params.clone());
            (
                run.run.result.wave as f64 * 1000. + run.run.result.lives as f64 * 10.,
                g.clone(),
            )
        })
        .collect::<Vec<_>>();
    let champion = evaluated
        .iter()
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .unwrap()
        .1
        .clone();
    let cma = DiagonalCma::new(&champion);
    let count = policies.len();
    let mut challengers = policies.clone();
    challengers.extend((0..count).map(|i| (cma.sample(rng), seed + 900_000 + i as u64 * 65_537)));
    let mut ranked = challengers
        .into_par_iter()
        .map(|(g, s)| {
            let run = run_policy(&g, s, (s % 3) as usize, 25, params.clone());
            (
                run.run.result.wave as f64 * 1000. + run.run.result.lives as f64 * 10.,
                g,
                s,
            )
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|a, b| b.0.total_cmp(&a.0));
    *policies = ranked
        .into_iter()
        .take(count)
        .map(|(_, g, s)| (g, s))
        .collect();
}

fn tier0_ok(params: &Params) -> bool {
    let baseline = Params::default();
    if !params.constants.growth.is_finite()
        || !params.constants.count.is_finite()
        || !(1.08..=1.16).contains(&params.constants.growth)
        || !(0.6..=1.0).contains(&params.constants.count)
    {
        return false;
    }
    let towers_ok = params
        .towers
        .iter()
        .zip(&baseline.towers)
        .all(|(tower, base)| {
            let score = tower.damage * if tower.rate > 0. { tower.rate } else { 1. };
            let base_score = base.damage * if base.rate > 0. { base.rate } else { 1. };
            tower.damage.is_finite()
                && tower.rate.is_finite()
                && (base_score == 0. || (0.4..=2.5).contains(&(score / base_score)))
        });
    towers_ok
        && params_vector(params)
            .into_iter()
            .all(|value| value.is_finite() && (0.4..=2.5).contains(&value))
}

fn modifier_patch(modifiers: &Modifiers) -> Value {
    let mut value = serde_json::to_value(modifiers).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .retain(|_, value| !value.is_null());
    value
}

fn js_params_patch(params: &Params) -> Value {
    let towers: Vec<_> = params
        .towers
        .iter()
        .map(|tower| {
            json!({
                "c":tower.cost,"d":tower.damage,"r":tower.rate,"rg":tower.range,
                "sp":tower.splash,"pi":tower.pierce,"bu":tower.burn,
                "kb":tower.knockback,"ramp":tower.ramp,"home":tower.homing,
                "aura":tower.aura,"field":tower.field,"beam":tower.beam,
                "b1":modifier_patch(&tower.branch1),"b2":modifier_patch(&tower.branch2)
            })
        })
        .collect();
    let fusions: Vec<_> = params
        .fusions
        .iter()
        .map(|fusion| {
            json!({
                "a":if fusion.a==usize::MAX {-1_i64} else {fusion.a as i64},
                "b":if fusion.b==usize::MAX {-1_i64} else {fusion.b as i64},
                "lv":fusion.level,
                "base":if fusion.base==usize::MAX {-1_i64} else {fusion.base as i64},
                "f":modifier_patch(&fusion.modifiers)
            })
        })
        .collect();
    json!({
        "constants":{"hp":params.constants.hp,"g":params.constants.growth,
            "cnt":params.constants.count,"spd":params.constants.speed,"bty":params.constants.bounty},
        "towers":towers,
        "fusions":fusions,
        "rules":params.rules
    })
}

fn output_tail(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .lines()
        .rev()
        .take(12)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
}

fn source_gates(params_path: &str) -> (bool, Value) {
    let root = env!("CARGO_MANIFEST_DIR").trim_end_matches("/balance-lab");
    let run = |script: &str| {
        Command::new("node")
            .arg(script)
            .current_dir(root)
            .env("EPOCH_PARAMS", params_path)
            .output()
            .unwrap()
    };
    let criteria = run("sim.js");
    let isolation = run("analysis.js");
    let passed = criteria.status.success() && isolation.status.success();
    (
        passed,
        json!({
            "passed":passed,
            "sixCriteria":{"passed":criteria.status.success(),"summary":output_tail(&criteria.stdout)},
            "isolationGuard":{"passed":isolation.status.success(),"summary":output_tail(&isolation.stdout)}
        }),
    )
}

fn defend(args: &[String]) {
    let generations: usize = args.get(2).and_then(|x| x.parse().ok()).unwrap_or(6);
    let candidates: usize = args.get(3).and_then(|x| x.parse().ok()).unwrap_or(24);
    let policy_count: usize = args.get(4).and_then(|x| x.parse().ok()).unwrap_or(48);
    let seed = args.get(5).and_then(|x| x.parse().ok()).unwrap_or(7);
    let calibration: Option<CalibrationPrior> = args
        .get(6)
        .map(|path| serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap());
    let mut rng = Xoshiro::new(seed);
    let mut policies: Vec<_> = (0..policy_count)
        .map(|i| (Genome::random(&mut rng), seed + i as u64 * 104729))
        .collect();
    if let Some(prior) = &calibration {
        let seeded = (policy_count / 4).max(1).min(policy_count);
        for (genome, _) in policies.iter_mut().take(seeded) {
            seed_from_prior(genome, prior);
        }
    }
    let mut best = Params::default();
    let started = Instant::now();
    let mut history = vec![];
    let mut shortlist: Vec<(f64, Params, Value)> = vec![];
    let mut parameter_cma = ParamCma::new();
    for generation in 0..generations {
        // Coupled minimax: rerun a warm-started attacker against the current
        // incumbent before evaluating the defender generation.
        refine_policies(
            &mut policies,
            &best,
            &mut rng,
            seed + generation as u64 * 1_000_003,
        );
        let (incumbent_loss, incumbent_metrics) = parameter_loss(&best, &policies, 60);
        let mut pool: Vec<_> = (0..candidates.saturating_sub(1))
            .map(|_| parameter_cma.sample(&mut rng))
            .collect();
        pool.push(best.clone());
        let cheap = &policies[..policies.len().min(8)];
        let mut scored: Vec<_> = pool
            .into_par_iter()
            .map(|p| {
                if tier0_ok(&p) {
                    let (l, m) = parameter_loss(&p, cheap, 25);
                    (l, p, m)
                } else {
                    (f64::INFINITY, p, json!({"rejectedBy":"tier0"}))
                }
            })
            .collect();
        scored.sort_by(|a, b| a.0.total_cmp(&b.0));
        scored.truncate((candidates / 3).max(2));
        let mut finalists: Vec<_> = scored
            .into_par_iter()
            .enumerate()
            .map(|(candidate_index, (_, p, _))| {
                let mut candidate_policies = policies.clone();
                let mut candidate_rng = Xoshiro::new(
                    seed + generation as u64 * 10_000_019 + candidate_index as u64 * 97_409,
                );
                refine_policies(
                    &mut candidate_policies,
                    &p,
                    &mut candidate_rng,
                    seed + generation as u64 * 20_000_033 + candidate_index as u64 * 193_939,
                );
                let (l, m) = parameter_loss(&p, &candidate_policies, 60);
                (l, p, m)
            })
            .collect();
        finalists.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut cma_samples = finalists
            .iter()
            .map(|(loss, params, _)| (*loss, params.clone()))
            .collect::<Vec<_>>();
        parameter_cma.update(&mut cma_samples);
        shortlist.extend(finalists.iter().take(3).cloned());
        shortlist.sort_by(|a, b| a.0.total_cmp(&b.0));
        shortlist.truncate(6);
        let (generation_loss, generation_metrics) = if finalists[0].0 < incumbent_loss {
            best = finalists[0].1.clone();
            (finalists[0].0, finalists[0].2.clone())
        } else {
            (incumbent_loss, incumbent_metrics)
        };
        history.push(json!({"generation":generation,"loss":generation_loss,"metrics":generation_metrics,"parameterSigma":parameter_cma.sigma,"attackerPool":policies.len(),"tier1Candidates":candidates,"tier2Finalists":finalists.len()}));
        eprintln!("defender generation {generation}: loss {generation_loss:.2}");
    }
    fs::create_dir_all("balance-lab/out").unwrap();
    let searched_params = best.clone();
    let (searched_loss, searched_metrics) = parameter_loss(&searched_params, &policies, 60);
    let candidate_path = format!(
        "{}/balance-lab/out/.candidate-params.json",
        env!("CARGO_MANIFEST_DIR").trim_end_matches("/balance-lab")
    );
    shortlist.push((searched_loss, searched_params, searched_metrics));
    let baseline = Params::default();
    let (baseline_loss, baseline_metrics) = parameter_loss(&baseline, &policies, 60);
    shortlist.push((baseline_loss, baseline, baseline_metrics.clone()));
    for entry in &mut shortlist {
        let (loss, metrics) = parameter_loss(&entry.1, &policies, 60);
        entry.0 = loss;
        entry.2 = metrics;
    }
    shortlist.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut selected = None;
    let mut rejected_candidates = 0usize;
    for (loss, params, metrics) in shortlist {
        fs::write(
            &candidate_path,
            serde_json::to_vec_pretty(&js_params_patch(&params)).unwrap(),
        )
        .unwrap();
        let (passed, gates) = source_gates(&candidate_path);
        if passed {
            selected = Some((loss, params, metrics, gates));
            break;
        }
        rejected_candidates += 1;
    }
    fs::remove_file(&candidate_path).unwrap();
    let (selected_loss, selected_params, selected_metrics, gates) =
        selected.expect("the shipped baseline must pass source-of-truth gates");
    let accepted = serde_json::to_value(js_params_patch(&selected_params)).unwrap()
        != serde_json::to_value(js_params_patch(&Params::default())).unwrap();
    if rejected_candidates > 0 {
        eprintln!(
            "defender rejected {rejected_candidates} lower-loss candidate(s) at source gates"
        );
    }
    let patch = js_params_patch(&selected_params);
    let elapsed = started.elapsed().as_secs_f64();
    let report = json!({"schema":1,"kind":"defender","optimizer":"diagonal-cma-es","minimaxRounds":generations,"tier1MaxWave":25,"tier2MaxWave":60,"seed":seed,"generations":generations,"candidates":candidates,"policies":policy_count,"calibration":calibration.as_ref().map(|p|json!({"sessions":p.sessions,"actions":p.actions})),"elapsedSeconds":elapsed,"threads":rayon::current_num_threads(),"coreHours":elapsed*rayon::current_num_threads() as f64/3600.,"loss":selected_loss,"metrics":selected_metrics,"history":history,"beforeAfter":{"before":{"loss":baseline_loss,"metrics":baseline_metrics},"after":{"loss":selected_loss,"metrics":selected_metrics}},"searchedCandidate":{"loss":searched_loss,"accepted":accepted,"rejectedCandidates":rejected_candidates},"gates":gates,"jsPatch":patch,"candidate":selected_params});
    fs::write(
        "balance-lab/out/defender-report.json",
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    fs::write(
        "balance-lab/out/params.json",
        serde_json::to_vec_pretty(&patch).unwrap(),
    )
    .unwrap();
    fs::write(
        "balance-lab/out/candidate.json",
        serde_json::to_vec_pretty(&selected_params).unwrap(),
    )
    .unwrap();
    println!("balance-lab/out/defender-report.json\nbalance-lab/out/params.json");
}

const PERK_NAMES: [&str; 12] = [
    "Napalm Doctrine",
    "Bounty Reform",
    "Rapid Logistics",
    "Standing Army",
    "War Economy",
    "Scrap Drive",
    "Shock Doctrine",
    "Iron Curtain",
    "Overcharge Rails",
    "Siege Corps",
    "Fission Ammo",
    "Cold Snap",
];
const EFFECT_METRICS: [&str; 17] = [
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
    "tickCapReached",
];
const CLEAR_SECONDS_METRIC: usize = 5;
const MECHANICAL_METRICS: std::ops::Range<usize> = 6..16;
const TICK_CAP_METRIC: usize = 16;
const SPEED_LEVELS: [f64; 5] = [0.65, 0.82, 1.0, 1.18, 1.4];
const RANGE_LEVELS: [f64; 5] = [0.75, 0.9, 1.0, 1.15, 1.3];
const HP_LEVELS: [f64; 3] = [0.85, 1.0, 1.18];
const COUNT_LEVELS: [f64; 3] = [0.75, 1.0, 1.25];
const ACTIVATION_LEVELS: [usize; 5] = [10, 14, 15, 20, 24];

#[derive(Clone)]
struct SensitivityGene {
    genome: Genome,
    trial_seed: u64,
    sell_strategy: bool,
    sell_slot: usize,
    map: usize,
    speed_level: usize,
    range_level: usize,
    hp_level: usize,
    count_level: usize,
    activation_wave: usize,
    focus_tower: usize,
}

#[derive(Clone, Copy)]
struct SensitivityContext {
    homing: bool,
    splash: bool,
    composition: &'static str,
    enemy_speed_factor: f64,
    range_factor: f64,
    hp_factor: f64,
    count_factor: f64,
    map: usize,
    activation_wave: usize,
    rush_proximity: &'static str,
    focus_tower: usize,
    sell_strategy: bool,
    sell_slot: usize,
    sell_scheduled: bool,
    screen_wave: usize,
    reaction_bucket: &'static str,
    reaction_ticks: u32,
    apm: f64,
}

impl SensitivityContext {
    fn json(self) -> Value {
        json!({
            "homing":self.homing,
            "splash":self.splash,
            "composition":self.composition,
            "enemySpeedFactor":self.enemy_speed_factor,
            "rangeFactor":self.range_factor,
            "hpFactor":self.hp_factor,
            "countFactor":self.count_factor,
            "map":self.map,
            "activationWave":self.activation_wave,
            "rushProximity":self.rush_proximity,
            "focusTower":self.focus_tower,
            "sellStrategy":self.sell_strategy,
            "sellSlot":self.sell_slot,
            "sellScheduled":self.sell_scheduled,
            "screenWave":self.screen_wave,
            "reactionBucket":self.reaction_bucket,
            "reactionTicks":self.reaction_ticks,
            "apm":self.apm,
        })
    }

    fn interaction_labels(self) -> Vec<String> {
        vec![
            format!("homing:{}", self.homing),
            format!("splash:{}", self.splash),
            format!("enemySpeed:{:.2}", self.enemy_speed_factor),
            format!("range:{:.2}", self.range_factor),
            format!("map:{}", self.map),
            format!("activationWave:{}", self.activation_wave),
            format!("rushProximity:{}", self.rush_proximity),
            format!("focusTower:{}", self.focus_tower),
            format!("composition:{}", self.composition),
            format!("sellStrategy:{}", self.sell_strategy),
            format!("screenWave:{}", self.screen_wave),
            format!("reaction:{}", self.reaction_bucket),
        ]
    }
}

struct PreparedSensitivity {
    input: Input,
    checkpoint: CounterfactualCheckpoint,
    baseline_reusable: bool,
    context: SensitivityContext,
    interaction_labels: Vec<String>,
    perk_relevant: [bool; 12],
}

#[derive(Clone)]
struct EffectObservation {
    metrics: [f64; EFFECT_METRICS.len()],
    utility: f64,
    outcome_score: f64,
    mechanical_score: f64,
    ordinary_eligible: bool,
    off_capped: bool,
    on_capped: bool,
    off_ticks: usize,
    on_ticks: usize,
}

#[derive(Clone, Copy, Debug)]
enum SearchObjective {
    Benefit,
    Harm,
    Mechanical,
}

impl SearchObjective {
    const ALL: [Self; 3] = [Self::Benefit, Self::Harm, Self::Mechanical];

    fn label(self) -> &'static str {
        match self {
            Self::Benefit => "benefit",
            Self::Harm => "harm",
            Self::Mechanical => "mechanical",
        }
    }
}

fn sensitivity_fitness(observation: &EffectObservation, objective: SearchObjective) -> f64 {
    if !observation.ordinary_eligible {
        return f64::NEG_INFINITY;
    }
    match objective {
        SearchObjective::Benefit => observation.utility * 1_000. + observation.mechanical_score,
        SearchObjective::Harm => -observation.utility * 1_000. + observation.mechanical_score,
        SearchObjective::Mechanical => {
            observation.mechanical_score * 1_000. + observation.outcome_score
        }
    }
}

#[derive(Clone)]
struct Moment {
    n: u64,
    sum: f64,
    sum_sq: f64,
    min: f64,
    max: f64,
}
impl Moment {
    fn new() -> Self {
        Self {
            n: 0,
            sum: 0.,
            sum_sq: 0.,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }
    fn push(&mut self, value: f64) {
        self.n += 1;
        self.sum += value;
        self.sum_sq += value * value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }
    fn merge(&mut self, other: &Self) {
        self.n += other.n;
        self.sum += other.sum;
        self.sum_sq += other.sum_sq;
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }
    fn mean(&self) -> f64 {
        self.sum / self.n.max(1) as f64
    }
    fn ci95(&self) -> (f64, f64) {
        if self.n < 2 {
            return (self.mean(), self.mean());
        }
        let n = self.n as f64;
        let variance = ((self.sum_sq - self.sum * self.sum / n) / (n - 1.)).max(0.);
        let radius = 1.96 * (variance / n).sqrt();
        (self.mean() - radius, self.mean() + radius)
    }
    fn json(&self) -> Value {
        if self.n == 0 {
            return json!({"mean":0.,"ci95":[0.,0.],"min":0.,"max":0.,"samples":0});
        }
        let (low, high) = self.ci95();
        json!({"mean":self.mean(),"ci95":[low,high],"min":self.min,"max":self.max,"samples":self.n})
    }
}

#[derive(Clone)]
struct EffectStats {
    pairs: u64,
    pair_censored: u64,
    off_capped: u64,
    on_capped: u64,
    cap_transition: u64,
    exact_zero: u64,
    outcome_changed: u64,
    mechanical_changed: u64,
    positive: u64,
    negative: u64,
    pathological_stall: u64,
    utility: Moment,
    metrics: Vec<Moment>,
    max_outcome_score: f64,
    max_mechanical_score: f64,
    max_outcome_context: Option<SensitivityContext>,
    max_mechanical_context: Option<SensitivityContext>,
}
impl EffectStats {
    fn new() -> Self {
        Self {
            pairs: 0,
            pair_censored: 0,
            off_capped: 0,
            on_capped: 0,
            cap_transition: 0,
            exact_zero: 0,
            outcome_changed: 0,
            mechanical_changed: 0,
            positive: 0,
            negative: 0,
            pathological_stall: 0,
            utility: Moment::new(),
            metrics: (0..EFFECT_METRICS.len()).map(|_| Moment::new()).collect(),
            max_outcome_score: 0.,
            max_mechanical_score: 0.,
            max_outcome_context: None,
            max_mechanical_context: None,
        }
    }

    fn complete_pairs(&self) -> u64 {
        self.pairs - self.pair_censored
    }

    fn both_capped(&self) -> u64 {
        self.pair_censored - self.cap_transition
    }

    fn push(&mut self, observation: &EffectObservation, context: SensitivityContext) {
        const EPS: f64 = 1e-9;
        self.pairs += 1;
        self.pair_censored += u64::from(observation.off_capped || observation.on_capped);
        self.off_capped += u64::from(observation.off_capped);
        self.on_capped += u64::from(observation.on_capped);
        self.cap_transition += u64::from(observation.off_capped != observation.on_capped);
        self.metrics[TICK_CAP_METRIC].push(observation.metrics[TICK_CAP_METRIC]);

        if !observation.ordinary_eligible {
            return;
        }

        self.utility.push(observation.utility);
        for (stats, value) in self.metrics[..TICK_CAP_METRIC]
            .iter_mut()
            .zip(observation.metrics[..TICK_CAP_METRIC].iter().copied())
        {
            stats.push(value)
        }
        if observation.metrics[..TICK_CAP_METRIC]
            .iter()
            .all(|value| value.abs() <= EPS)
        {
            self.exact_zero += 1
        }
        if observation.metrics[..CLEAR_SECONDS_METRIC + 1]
            .iter()
            .any(|value| value.abs() > EPS)
        {
            self.outcome_changed += 1
        }
        if observation.metrics[MECHANICAL_METRICS]
            .iter()
            .any(|value| value.abs() > EPS)
        {
            self.mechanical_changed += 1
        }
        if observation.utility > 0.1 {
            self.positive += 1
        }
        if observation.utility < -0.1 {
            self.negative += 1
        }
        if observation.metrics[CLEAR_SECONDS_METRIC] > 600.
            && observation.metrics[0].abs() <= EPS
            && observation.metrics[1].abs() <= EPS
            && observation.metrics[2].abs() <= EPS
            && observation.metrics[3].abs() <= EPS
            && observation.metrics[4].abs() <= EPS
        {
            self.pathological_stall += 1
        }
        if observation.outcome_score > self.max_outcome_score {
            self.max_outcome_context = Some(context);
        }
        if observation.mechanical_score > self.max_mechanical_score {
            self.max_mechanical_context = Some(context);
        }
        self.max_outcome_score = self.max_outcome_score.max(observation.outcome_score);
        self.max_mechanical_score = self.max_mechanical_score.max(observation.mechanical_score);
    }
    fn merge(&mut self, other: &Self) {
        self.pairs += other.pairs;
        self.pair_censored += other.pair_censored;
        self.off_capped += other.off_capped;
        self.on_capped += other.on_capped;
        self.cap_transition += other.cap_transition;
        self.exact_zero += other.exact_zero;
        self.outcome_changed += other.outcome_changed;
        self.mechanical_changed += other.mechanical_changed;
        self.positive += other.positive;
        self.negative += other.negative;
        self.pathological_stall += other.pathological_stall;
        self.utility.merge(&other.utility);
        for (left, right) in self.metrics.iter_mut().zip(&other.metrics) {
            left.merge(right)
        }
        if other.max_outcome_score > self.max_outcome_score {
            self.max_outcome_context = other.max_outcome_context;
        }
        if other.max_mechanical_score > self.max_mechanical_score {
            self.max_mechanical_context = other.max_mechanical_context;
        }
        self.max_outcome_score = self.max_outcome_score.max(other.max_outcome_score);
        self.max_mechanical_score = self.max_mechanical_score.max(other.max_mechanical_score);
    }
    fn json(&self) -> Value {
        let complete_pairs = self.complete_pairs();
        let both_capped = self.both_capped();
        let exact_zero_ci = wilson95(self.exact_zero, complete_pairs);
        let outcome_change_ci = wilson95(self.outcome_changed, complete_pairs);
        let mechanical_change_ci = wilson95(self.mechanical_changed, complete_pairs);
        let positive_ci = wilson95(self.positive, complete_pairs);
        let negative_ci = wilson95(self.negative, complete_pairs);
        let pair_censored_ci = wilson95(self.pair_censored, self.pairs);
        let off_capped_ci = wilson95(self.off_capped, self.pairs);
        let on_capped_ci = wilson95(self.on_capped, self.pairs);
        let cap_transition_ci = wilson95(self.cap_transition, self.pairs);
        let both_capped_ci = wilson95(both_capped, self.pairs);
        let metrics = EFFECT_METRICS
            .iter()
            .zip(&self.metrics)
            .map(|(name, stats)| ((*name).to_string(), stats.json()))
            .collect::<serde_json::Map<_, _>>();
        json!({
            "pairs":self.pairs,
            "completePairs":complete_pairs,
            "pairCensored":self.pair_censored,
            "pairCensoredRate":self.pair_censored as f64/self.pairs.max(1) as f64,
            "pairCensoredRateCi95":pair_censored_ci,
            "offCapped":self.off_capped,
            "offCappedRate":self.off_capped as f64/self.pairs.max(1) as f64,
            "offCappedRateCi95":off_capped_ci,
            "onCapped":self.on_capped,
            "onCappedRate":self.on_capped as f64/self.pairs.max(1) as f64,
            "onCappedRateCi95":on_capped_ci,
            "capTransition":self.cap_transition,
            "capTransitionRate":self.cap_transition as f64/self.pairs.max(1) as f64,
            "capTransitionRateCi95":cap_transition_ci,
            "bothCapped":both_capped,
            "bothCappedRate":both_capped as f64/self.pairs.max(1) as f64,
            "bothCappedRateCi95":both_capped_ci,
            "exactZero":self.exact_zero,
            "exactZeroRate":self.exact_zero as f64/complete_pairs.max(1) as f64,
            "exactZeroRateCi95":exact_zero_ci,
            "outcomeChanged":self.outcome_changed,
            "outcomeChangeRate":self.outcome_changed as f64/complete_pairs.max(1) as f64,
            "outcomeChangeRateCi95":outcome_change_ci,
            "mechanicalChanged":self.mechanical_changed,
            "mechanicalChangeRate":self.mechanical_changed as f64/complete_pairs.max(1) as f64,
            "mechanicalChangeRateCi95":mechanical_change_ci,
            "positive":self.positive,
            "positiveRate":self.positive as f64/complete_pairs.max(1) as f64,
            "positiveRateCi95":positive_ci,
            "negative":self.negative,
            "negativeRate":self.negative as f64/complete_pairs.max(1) as f64,
            "negativeRateCi95":negative_ci,
            "pathologicalStall":self.pathological_stall,
            "pathologicalStallRate":self.pathological_stall as f64/complete_pairs.max(1) as f64,
            "pathologicalStallRateCi95":wilson95(self.pathological_stall,complete_pairs),
            "utility":self.utility.json(),
            "metrics":metrics,
            "maxOutcomeScore":self.max_outcome_score,
            "maxMechanicalScore":self.max_mechanical_score,
            "maxOutcomeContext":self.max_outcome_context.map(SensitivityContext::json),
            "maxMechanicalContext":self.max_mechanical_context.map(SensitivityContext::json),
        })
    }
}

fn wilson95(successes: u64, trials: u64) -> [f64; 2] {
    if trials == 0 {
        return [0., 1.];
    }
    let n = trials as f64;
    let p = successes as f64 / n;
    let z = 1.96;
    let denominator = 1. + z * z / n;
    let center = (p + z * z / (2. * n)) / denominator;
    let radius = z * ((p * (1. - p) + z * z / (4. * n)) / n).sqrt() / denominator;
    [(center - radius).max(0.), (center + radius).min(1.)]
}

struct SensitivityAccumulator {
    perks: Vec<EffectStats>,
    perk_strata: Vec<BTreeMap<String, EffectStats>>,
    projectile_interactions: BTreeMap<String, EffectStats>,
    policy_runs: u64,
}
impl SensitivityAccumulator {
    fn new() -> Self {
        Self {
            perks: (0..12).map(|_| EffectStats::new()).collect(),
            perk_strata: (0..12).map(|_| BTreeMap::new()).collect(),
            projectile_interactions: BTreeMap::new(),
            policy_runs: 0,
        }
    }
    fn record(
        &mut self,
        perk: usize,
        observation: &EffectObservation,
        context: SensitivityContext,
        interaction_labels: &[String],
        relevant: bool,
    ) {
        self.perks[perk].push(observation, context);
        for label in [
            format!("composition:{}", context.composition),
            format!(
                "mechanic:{}",
                if relevant { "relevant" } else { "inactive" }
            ),
        ] {
            self.perk_strata[perk]
                .entry(label)
                .or_insert_with(EffectStats::new)
                .push(observation, context);
        }
        if perk == 8 {
            for label in interaction_labels {
                self.projectile_interactions
                    .entry(label.clone())
                    .or_insert_with(EffectStats::new)
                    .push(observation, context);
            }
            let joint = format!("joint:{}", interaction_labels.join("|"));
            self.projectile_interactions
                .entry(joint)
                .or_insert_with(EffectStats::new)
                .push(observation, context);
        }
    }
    fn merge(&mut self, other: &Self) {
        self.policy_runs += other.policy_runs;
        for (left, right) in self.perks.iter_mut().zip(&other.perks) {
            left.merge(right)
        }
        for (perk, strata) in other.perk_strata.iter().enumerate() {
            for (key, stats) in strata {
                self.perk_strata[perk]
                    .entry(key.clone())
                    .or_insert_with(EffectStats::new)
                    .merge(stats)
            }
        }
        for (key, stats) in &other.projectile_interactions {
            self.projectile_interactions
                .entry(key.clone())
                .or_insert_with(EffectStats::new)
                .merge(stats)
        }
    }
}

fn broad_gene(seed: u64, index: usize) -> SensitivityGene {
    let mut rng = Xoshiro::new(seed ^ (index as u64).wrapping_mul(0x9e3779b97f4a7c15));
    let mut genome = Genome::random(&mut rng);
    genome.reaction_ticks = (index % 91) as u32;
    let focus_tower = index % 10;
    for weight in &mut genome.tower_weights {
        *weight -= 0.75
    }
    genome.tower_weights[focus_tower] += 3.;
    if index.is_multiple_of(4) {
        genome.tower_weights[(focus_tower + 1 + index / 10 % 9) % 10] += 2.
    }
    SensitivityGene {
        genome,
        trial_seed: seed.wrapping_add(index as u64 * 104_729),
        sell_strategy: index.is_multiple_of(4),
        sell_slot: index / 4 % 8,
        map: index % 3,
        speed_level: index / 3 % SPEED_LEVELS.len(),
        range_level: index / 15 % RANGE_LEVELS.len(),
        hp_level: index / 75 % HP_LEVELS.len(),
        count_level: index / 225 % COUNT_LEVELS.len(),
        activation_wave: ACTIVATION_LEVELS[index / 675 % ACTIVATION_LEVELS.len()],
        focus_tower,
    }
}

fn sensitivity_params(gene: &SensitivityGene) -> Params {
    let mut params = Params::default();
    params.constants.speed *= SPEED_LEVELS[gene.speed_level];
    params.constants.hp *= HP_LEVELS[gene.hp_level];
    params.constants.count = (params.constants.count * COUNT_LEVELS[gene.count_level]).min(1.);
    for tower in &mut params.towers {
        tower.range *= RANGE_LEVELS[gene.range_level]
    }
    params
}

fn reaction_bucket(ticks: u32) -> &'static str {
    if ticks <= 10 {
        "fast"
    } else if ticks <= 30 {
        "medium"
    } else {
        "slow"
    }
}

fn rush_proximity(activation_wave: usize) -> &'static str {
    match activation_wave % 5 {
        4 => "immediately-before",
        0 => "on-rush",
        _ => "between-rushes",
    }
}

fn prepare_sensitivity(
    gene: &SensitivityGene,
    seed: u64,
    max_wave: usize,
    collect_contexts: bool,
) -> PreparedSensitivity {
    let params = sensitivity_params(gene);
    let (mut policy, checkpoint) = run_policy_with_checkpoint(
        &gene.genome,
        seed,
        gene.map,
        max_wave,
        params.clone(),
        gene.activation_wave,
    );
    let mut baseline_reusable = true;
    if gene.sell_strategy {
        let sell_target = policy
            .actions
            .iter()
            .filter(|action| action.op == "place" && action.wave <= gene.activation_wave)
            .nth(gene.sell_slot)
            .or_else(|| {
                let candidates = policy
                    .actions
                    .iter()
                    .filter(|action| action.op == "place" && action.wave <= gene.activation_wave)
                    .collect::<Vec<_>>();
                candidates
                    .get(gene.sell_slot % candidates.len().max(1))
                    .copied()
            })
            .cloned();
        if let Some(target) = sell_target {
            policy.actions.push(Action {
                wave: gene.activation_wave,
                tick: None,
                op: "sell".into(),
                tower: 0,
                x: target.x,
                y: target.y,
                branch: 0,
                with: None,
            });
            baseline_reusable = false;
        }
    }
    let has_homing = policy
        .tower_counts
        .iter()
        .enumerate()
        .any(|(tower, &count)| count > 0 && params.towers[tower].homing);
    let has_splash = policy
        .tower_counts
        .iter()
        .enumerate()
        .any(|(tower, &count)| {
            count > 0
                && (params.towers[tower].splash > 0.
                    || (policy.branch_counts[tower][0] > 0
                        && params.towers[tower].branch1.sp.is_some())
                    || (policy.branch_counts[tower][1] > 0
                        && params.towers[tower].branch2.sp.is_some()))
        })
        || policy
            .fusion_counts
            .iter()
            .enumerate()
            .any(|(fusion, &count)| count > 0 && params.fusions[fusion].modifiers.sp.is_some());
    let has_burn = policy
        .tower_counts
        .iter()
        .enumerate()
        .any(|(tower, &count)| {
            count > 0
                && (params.towers[tower].burn
                    || (policy.branch_counts[tower][0] > 0
                        && params.towers[tower].branch1.bu.is_some())
                    || (policy.branch_counts[tower][1] > 0
                        && params.towers[tower].branch2.bu.is_some()))
        })
        || policy
            .fusion_counts
            .iter()
            .enumerate()
            .any(|(fusion, &count)| count > 0 && params.fusions[fusion].modifiers.bu.is_some());
    let has_knockback = policy
        .tower_counts
        .iter()
        .enumerate()
        .any(|(tower, &count)| {
            count > 0
                && (params.towers[tower].knockback > 0.
                    || params.towers[tower].branch1.kb.is_some()
                    || params.towers[tower].branch2.kb.is_some())
        })
        || policy
            .fusion_counts
            .iter()
            .enumerate()
            .any(|(fusion, &count)| count > 0 && params.fusions[fusion].modifiers.kb.is_some());
    let has_projectile = policy
        .tower_counts
        .iter()
        .enumerate()
        .any(|(tower, &count)| {
            count > 0
                && !params.towers[tower].aura
                && !params.towers[tower].field
                && !params.towers[tower].beam
        });
    let has_irradiation = policy
        .tower_counts
        .iter()
        .enumerate()
        .any(|(tower, &count)| {
            count > 0
                && (params.towers[tower].aura
                    || params.towers[tower].branch1.ir.is_some()
                    || params.towers[tower].branch2.ir.is_some())
        })
        || policy
            .fusion_counts
            .iter()
            .enumerate()
            .any(|(fusion, &count)| count > 0 && params.fusions[fusion].modifiers.ir.is_some());
    let has_field = policy
        .tower_counts
        .iter()
        .enumerate()
        .any(|(tower, &count)| count > 0 && params.towers[tower].field);
    let tower_kinds = policy
        .tower_counts
        .iter()
        .filter(|&&count| count > 0)
        .count();
    let composition = match tower_kinds {
        0 => "empty",
        1 => "mono",
        2..=3 => "focused-mix",
        _ => "broad-mix",
    };
    let context = SensitivityContext {
        homing: has_homing,
        splash: has_splash,
        composition,
        enemy_speed_factor: SPEED_LEVELS[gene.speed_level],
        range_factor: RANGE_LEVELS[gene.range_level],
        hp_factor: HP_LEVELS[gene.hp_level],
        count_factor: COUNT_LEVELS[gene.count_level],
        map: gene.map,
        activation_wave: gene.activation_wave,
        rush_proximity: rush_proximity(gene.activation_wave),
        focus_tower: gene.focus_tower,
        sell_strategy: gene.sell_strategy,
        sell_slot: gene.sell_slot,
        sell_scheduled: !baseline_reusable,
        screen_wave: max_wave,
        reaction_bucket: reaction_bucket(gene.genome.reaction_ticks),
        reaction_ticks: gene.genome.reaction_ticks,
        apm: gene.genome.apm,
    };
    let interaction_labels = if collect_contexts {
        context.interaction_labels()
    } else {
        vec![]
    };
    let has_tower = policy.tower_counts.iter().any(|&count| count > 0);
    let power_scheduled = policy
        .actions
        .iter()
        .any(|action| action.op == "power" && action.wave >= gene.activation_wave);
    let upgrade_scheduled = policy
        .actions
        .iter()
        .any(|action| action.op == "upgrade" && action.wave >= gene.activation_wave);
    let perk_relevant = [
        has_burn,
        has_tower,
        power_scheduled,
        has_tower,
        !baseline_reusable,
        upgrade_scheduled,
        has_knockback,
        true,
        has_projectile,
        has_splash,
        has_irradiation,
        has_field,
    ];
    PreparedSensitivity {
        input: Input {
            seed,
            map: gene.map,
            max_wave,
            actions: policy.actions,
            params: None,
            initial: None,
            endless: max_wave > 50,
        },
        checkpoint,
        baseline_reusable,
        context,
        interaction_labels,
        perk_relevant,
    }
}

fn mean_latency(telemetry: &Telemetry) -> f64 {
    telemetry.projectile_latency_seconds / telemetry.projectile_impacts.max(1) as f64
}

fn per_combat_second(value: f64, telemetry: &Telemetry) -> f64 {
    value / telemetry.combat_seconds.max(DT)
}

fn overkill_per_kill(summary: &CounterfactualSummary) -> f64 {
    summary.telemetry.overkill_damage / summary.kills.max(1) as f64
}

fn wasted_shot_rate(telemetry: &Telemetry) -> f64 {
    telemetry.wasted_projectiles as f64 / telemetry.projectiles_fired.max(1) as f64
}

fn power_uses(telemetry: &Telemetry) -> f64 {
    telemetry.power_uses.iter().sum::<u64>() as f64
}

fn evaluate_prepared(
    prepared: &PreparedSensitivity,
    perk: usize,
    activation_wave: usize,
) -> EffectObservation {
    let (off, on) = run_counterfactual_summary_pair_from_checkpoint(
        &prepared.input,
        &prepared.checkpoint,
        perk,
        activation_wave,
        prepared.baseline_reusable,
    );
    effect_observation(off, on)
}

fn evaluate_prepared_batch(
    prepared: &PreparedSensitivity,
    perks: &[usize],
    activation_wave: usize,
) -> Vec<(usize, EffectObservation)> {
    run_counterfactual_summary_batch_from_checkpoint(
        &prepared.input,
        &prepared.checkpoint,
        perks,
        activation_wave,
        prepared.baseline_reusable,
    )
    .into_iter()
    .map(|(perk, off, on)| (perk, effect_observation(off, on)))
    .collect()
}

fn effect_observation(off: CounterfactualSummary, on: CounterfactualSummary) -> EffectObservation {
    let off_capped = off.capped;
    let on_capped = on.capped;
    let off_ticks = off.ticks;
    let on_ticks = on.ticks;
    let ordinary_eligible = !off_capped && !on_capped;
    let tick_cap_reached = u8::from(on_capped) as f64 - u8::from(off_capped) as f64;
    let metrics = if ordinary_eligible {
        [
            on.wave as f64 - off.wave as f64,
            on.lives as f64 - off.lives as f64,
            on.gold - off.gold,
            on.telemetry.leaks as f64 - off.telemetry.leaks as f64,
            on.telemetry.leak_damage as f64 - off.telemetry.leak_damage as f64,
            on.seconds - off.seconds,
            on.telemetry.effective_damage - off.telemetry.effective_damage,
            on.telemetry.overkill_damage - off.telemetry.overkill_damage,
            mean_latency(&on.telemetry) - mean_latency(&off.telemetry),
            on.telemetry.wasted_projectiles as f64 - off.telemetry.wasted_projectiles as f64,
            per_combat_second(on.telemetry.effective_damage, &on.telemetry)
                - per_combat_second(off.telemetry.effective_damage, &off.telemetry),
            overkill_per_kill(&on) - overkill_per_kill(&off),
            wasted_shot_rate(&on.telemetry) - wasted_shot_rate(&off.telemetry),
            per_combat_second(on.telemetry.knockback_distance, &on.telemetry)
                - per_combat_second(off.telemetry.knockback_distance, &off.telemetry),
            per_combat_second(on.telemetry.slow_enemy_seconds, &on.telemetry)
                - per_combat_second(off.telemetry.slow_enemy_seconds, &off.telemetry),
            power_uses(&on.telemetry) - power_uses(&off.telemetry),
            tick_cap_reached,
        ]
    } else {
        let mut censored = [0.; EFFECT_METRICS.len()];
        censored[TICK_CAP_METRIC] = tick_cap_reached;
        censored
    };
    let utility = metrics[0] * 100. + metrics[1] * 5. + metrics[2] * 0.02
        - metrics[3] * 2.
        - metrics[4] * 3.
        - metrics[5] * 0.05
        - metrics[9] * 0.1;
    let outcome_score = metrics[0].abs() * 100.
        + metrics[1].abs() * 10.
        + metrics[2].abs() * 0.01
        + metrics[3].abs() * 5.
        + metrics[4].abs() * 10.
        + metrics[5].abs() * 0.1;
    let mechanical_score = metrics[6].abs() * 0.001
        + metrics[7].abs() * 0.001
        + metrics[8].abs() * 10.
        + metrics[9].abs()
        + metrics[10].abs()
        + metrics[11].abs()
        + metrics[12].abs() * 100.
        + metrics[13].abs() * 10.
        + metrics[14].abs() * 10.
        + metrics[15].abs();
    EffectObservation {
        metrics,
        utility,
        outcome_score,
        mechanical_score,
        ordinary_eligible,
        off_capped,
        on_capped,
        off_ticks,
        on_ticks,
    }
}

fn sensitivity_example(
    gene: &SensitivityGene,
    scenario_seed: u64,
    context: SensitivityContext,
    observation: &EffectObservation,
    objective: SearchObjective,
) -> Value {
    debug_assert!(observation.ordinary_eligible);
    let fitness = sensitivity_fitness(observation, objective);
    json!({
        "fitness":fitness,
        "searchObjective":objective.label(),
        "scenarioSeed":scenario_seed.to_string(),
        "screenSeed":scenario_seed.to_string(),
        "screenWave":context.screen_wave,
        "map":context.map,
        "enemySpeedFactor":context.enemy_speed_factor,
        "rangeFactor":context.range_factor,
        "hpFactor":context.hp_factor,
        "countFactor":context.count_factor,
        "activationWave":context.activation_wave,
        "rushProximity":context.rush_proximity,
        "focusTower":context.focus_tower,
        "sellStrategy":context.sell_strategy,
        "sellSlot":context.sell_slot,
        "sellScheduled":context.sell_scheduled,
        "offCapped":observation.off_capped,
        "onCapped":observation.on_capped,
        "offTicks":observation.off_ticks,
        "onTicks":observation.on_ticks,
        "ordinaryMetricsEligible":observation.ordinary_eligible,
        "reactionTicks":context.reaction_ticks,
        "apm":context.apm,
        "context":context.json(),
        "genome":gene.genome.to_vector(),
        "effect":{
            "metrics":EFFECT_METRICS.iter().zip(observation.metrics).collect::<BTreeMap<_,_>>(),
            "utility":observation.utility,
            "outcomeScore":observation.outcome_score,
            "mechanicalScore":observation.mechanical_score,
        }
    })
}

fn mutate_gene(parent: &SensitivityGene, rng: &mut Xoshiro) -> SensitivityGene {
    let mut vector = parent.genome.to_vector();
    for (index, value) in vector.iter_mut().enumerate() {
        let scale = match index {
            39..=41 => 60.,
            43 => 120.,
            44 => 15.,
            45 => 20.,
            _ => 0.22,
        };
        *value += normal(rng) * scale;
    }
    let mut child = parent.clone();
    child.genome = Genome::from_vector(&vector);
    if rng.next_f64() < 0.2 {
        child.map = rng.next_u64() as usize % 3
    }
    if rng.next_f64() < 0.3 {
        child.speed_level = rng.next_u64() as usize % SPEED_LEVELS.len()
    }
    if rng.next_f64() < 0.3 {
        child.range_level = rng.next_u64() as usize % RANGE_LEVELS.len()
    }
    if rng.next_f64() < 0.2 {
        child.hp_level = rng.next_u64() as usize % HP_LEVELS.len()
    }
    if rng.next_f64() < 0.2 {
        child.count_level = rng.next_u64() as usize % COUNT_LEVELS.len()
    }
    if rng.next_f64() < 0.2 {
        child.activation_wave = ACTIVATION_LEVELS[rng.next_u64() as usize % ACTIVATION_LEVELS.len()]
    }
    if rng.next_f64() < 0.25 {
        child.focus_tower = rng.next_u64() as usize % 10;
        child.genome.tower_weights[child.focus_tower] += 2.
    }
    if rng.next_f64() < 0.15 {
        child.trial_seed = rng.next_u64()
    }
    if rng.next_f64() < 0.2 {
        child.sell_strategy = !child.sell_strategy
    }
    if rng.next_f64() < 0.2 {
        child.sell_slot = rng.next_u64() as usize % 8
    }
    child
}

fn classify_perk(broad: &EffectStats, targeted: &EffectStats, full: &EffectStats) -> &'static str {
    if broad.complete_pairs() == 0 {
        return "insufficient complete evidence";
    }
    let outcome_rate_ci = wilson95(broad.outcome_changed, broad.complete_pairs());
    let cap_transition_rate_ci = wilson95(broad.cap_transition, broad.pairs);
    let all_mechanical =
        broad.mechanical_changed + targeted.mechanical_changed + full.mechanical_changed;
    let complete_pairs = broad.complete_pairs() + targeted.complete_pairs() + full.complete_pairs();
    let all_cap_transitions = broad.cap_transition + targeted.cap_transition + full.cap_transition;
    let max_outcome = broad
        .max_outcome_score
        .max(targeted.max_outcome_score)
        .max(full.max_outcome_score);
    let (_, utility_high) = broad.utility.ci95();
    if complete_pairs > 0
        && all_mechanical == 0
        && broad.outcome_changed + targeted.outcome_changed + full.outcome_changed == 0
        && all_cap_transitions == 0
    {
        "mechanically inactive"
    } else if utility_high < -0.1 {
        "harmful"
    } else if outcome_rate_ci[1] < 0.001 && max_outcome < 1. && all_cap_transitions == 0 {
        "redundant"
    } else if (outcome_rate_ci[1] < 0.01 && max_outcome >= 1.)
        || (all_cap_transitions > 0
            && outcome_rate_ci[1] < 0.01
            && cap_transition_rate_ci[1] < 0.01)
    {
        "niche-only"
    } else {
        "meaningful"
    }
}

fn sensitivity_screen_wave(perk: usize, default_wave: usize) -> usize {
    if [8, 10, 11].contains(&perk) {
        default_wave.max(50)
    } else {
        default_wave
    }
}

fn sensitivity_replay(args: &[String]) {
    let path = args
        .get(2)
        .map(String::as_str)
        .unwrap_or("balance-lab/out/sensitivity-report.json");
    let selector = args.get(3).map(String::as_str).unwrap_or("11");
    let field = args
        .get(4)
        .map(String::as_str)
        .unwrap_or("bestFullWaveExample");
    let report: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    let perks = report["perks"]
        .as_array()
        .expect("report perks must be an array");
    let perk = selector
        .parse::<usize>()
        .ok()
        .filter(|&index| index < perks.len())
        .or_else(|| {
            perks
                .iter()
                .position(|entry| entry["name"].as_str() == Some(selector))
        })
        .expect("perk must be a valid id or exact name");
    let example = &perks[perk][field];
    assert!(example.is_object(), "selected report example is missing");
    let factor_index = |levels: &[f64], key: &str| {
        let value = example[key]
            .as_f64()
            .unwrap_or_else(|| panic!("example field {key} must be numeric"));
        levels
            .iter()
            .position(|candidate| (*candidate - value).abs() <= 1e-9)
            .unwrap_or_else(|| panic!("example field {key}={value} is not canonical"))
    };
    let scenario_seed = example["scenarioSeed"]
        .as_str()
        .expect("scenarioSeed must be an exact decimal string")
        .parse::<u64>()
        .unwrap();
    let gene = SensitivityGene {
        genome: Genome::from_vector(
            &serde_json::from_value::<Vec<f64>>(example["genome"].clone()).unwrap(),
        ),
        trial_seed: scenario_seed,
        sell_strategy: example["sellStrategy"].as_bool().unwrap(),
        sell_slot: example["sellSlot"].as_u64().unwrap() as usize,
        map: example["map"].as_u64().unwrap() as usize,
        speed_level: factor_index(&SPEED_LEVELS, "enemySpeedFactor"),
        range_level: factor_index(&RANGE_LEVELS, "rangeFactor"),
        hp_level: factor_index(&HP_LEVELS, "hpFactor"),
        count_level: factor_index(&COUNT_LEVELS, "countFactor"),
        activation_wave: example["activationWave"].as_u64().unwrap() as usize,
        focus_tower: example["focusTower"].as_u64().unwrap() as usize,
    };
    let screen_wave = example["screenWave"].as_u64().unwrap() as usize;
    let prepared = prepare_sensitivity(&gene, scenario_seed, screen_wave, true);
    let scheduled_power_actions = std::array::from_fn::<_, 3, _>(|power| {
        prepared
            .input
            .actions
            .iter()
            .filter(|action| action.op == "power" && action.tower == power)
            .count()
    });
    let (off, on) = run_counterfactual_summary_pair_from_checkpoint(
        &prepared.input,
        &prepared.checkpoint,
        perk,
        gene.activation_wave,
        prepared.baseline_reusable,
    );
    let observation = effect_observation(off.clone(), on.clone());
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "report":path,
            "perk":{"id":perk,"name":PERK_NAMES[perk]},
            "field":field,
            "screenWave":screen_wave,
            "scenarioSeed":scenario_seed.to_string(),
            "scheduledActions":prepared.input.actions.len(),
            "scheduledPowerActions":scheduled_power_actions,
            "off":off,
            "on":on,
            "effect":{
                "metrics":EFFECT_METRICS.iter().zip(observation.metrics).collect::<BTreeMap<_,_>>(),
                "utility":observation.utility,
                "outcomeScore":observation.outcome_score,
                "mechanicalScore":observation.mechanical_score,
                "ordinaryMetricsEligible":observation.ordinary_eligible,
            }
        }))
        .unwrap()
    );
}

fn source_revision() -> String {
    let root = env!("CARGO_MANIFEST_DIR").trim_end_matches("/balance-lab");
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| {
            let revision = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            (!revision.is_empty()).then_some(revision)
        })
        .unwrap_or_else(|| "unknown".to_owned())
}

fn source_dirty() -> bool {
    let root = env!("CARGO_MANIFEST_DIR").trim_end_matches("/balance-lab");
    Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .is_none_or(|output| !output.stdout.is_empty())
}

fn sensitivity(args: &[String]) {
    let broad_scenarios = args
        .get(2)
        .and_then(|x| x.parse().ok())
        .unwrap_or(100_000usize);
    let ga_generations = args.get(3).and_then(|x| x.parse().ok()).unwrap_or(34usize);
    let ga_population = args
        .get(4)
        .and_then(|x| x.parse().ok())
        .unwrap_or(2048usize);
    let seed = args
        .get(5)
        .and_then(|x| x.parse().ok())
        .unwrap_or(20260719u64);
    let max_wave = args.get(6).and_then(|x| x.parse().ok()).unwrap_or(25usize);
    let late_screen_wave = max_wave.max(50);
    let output_path = args
        .get(7)
        .map(String::as_str)
        .unwrap_or("balance-lab/out/sensitivity-report.json");
    assert!(broad_scenarios > 0 && ga_generations > 0 && ga_population >= 2);
    let source_revision_at_start = source_revision();
    let source_dirty_at_start = source_dirty();
    let started = Instant::now();
    let mut broad = SensitivityAccumulator::new();
    let base_perks: Vec<_> = (0..12)
        .filter(|&perk| sensitivity_screen_wave(perk, max_wave) == max_wave)
        .collect();
    let late_perks: Vec<_> = (0..12)
        .filter(|&perk| sensitivity_screen_wave(perk, max_wave) > max_wave)
        .collect();
    let chunk = 2_000usize;
    for start in (0..broad_scenarios).step_by(chunk) {
        let end = (start + chunk).min(broad_scenarios);
        let partial = (start..end)
            .into_par_iter()
            .with_max_len(1)
            .fold(SensitivityAccumulator::new, |mut accumulator, index| {
                let gene = broad_gene(seed, index);
                let scenario_seed = seed.wrapping_add(index as u64 * 104_729);
                let prepared = prepare_sensitivity(&gene, scenario_seed, max_wave, true);
                let late_prepared = (max_wave < late_screen_wave)
                    .then(|| prepare_sensitivity(&gene, scenario_seed, late_screen_wave, true));
                accumulator.policy_runs += 1 + u64::from(late_prepared.is_some());
                for (perk, observation) in
                    evaluate_prepared_batch(&prepared, &base_perks, gene.activation_wave)
                {
                    accumulator.record(
                        perk,
                        &observation,
                        prepared.context,
                        &prepared.interaction_labels,
                        prepared.perk_relevant[perk],
                    );
                }
                if let Some(late_prepared) = &late_prepared {
                    for (perk, observation) in
                        evaluate_prepared_batch(late_prepared, &late_perks, gene.activation_wave)
                    {
                        accumulator.record(
                            perk,
                            &observation,
                            late_prepared.context,
                            &late_prepared.interaction_labels,
                            late_prepared.perk_relevant[perk],
                        );
                    }
                }
                accumulator
            })
            .reduce(SensitivityAccumulator::new, |mut left, right| {
                left.merge(&right);
                left
            });
        broad.merge(&partial);
        eprintln!(
            "sensitivity broad: {end}/{broad_scenarios} scenarios, {} paired comparisons",
            end as u64 * 12
        );
    }

    let mut targeted: Vec<EffectStats> = (0..12).map(|_| EffectStats::new()).collect();
    let mut full_wave: Vec<EffectStats> = (0..12).map(|_| EffectStats::new()).collect();
    let mut targeted_by_objective: Vec<[EffectStats; 3]> = (0..12)
        .map(|_| std::array::from_fn(|_| EffectStats::new()))
        .collect();
    let mut full_wave_by_objective: Vec<[EffectStats; 3]> = (0..12)
        .map(|_| std::array::from_fn(|_| EffectStats::new()))
        .collect();
    let mut best_examples: Vec<[Value; 3]> = (0..12)
        .map(|_| std::array::from_fn(|_| Value::Null))
        .collect();
    let mut best_full_wave_examples: Vec<[Value; 3]> = (0..12)
        .map(|_| std::array::from_fn(|_| Value::Null))
        .collect();
    let mut targeted_policy_runs = 0u64;
    let mut full_policy_runs = 0u64;
    for perk in 0..12 {
        for (objective_index, objective) in SearchObjective::ALL.into_iter().enumerate() {
            let objective_seed = seed
                ^ (perk as u64 + 1).wrapping_mul(0xd1b54a32d192ed03)
                ^ (objective_index as u64 + 1).wrapping_mul(0x94d049bb133111eb);
            let mut rng = Xoshiro::new(objective_seed);
            let mut population: Vec<_> = (0..ga_population)
                .map(|index| {
                    broad_gene(
                        objective_seed,
                        index + (perk * 3 + objective_index) * ga_population,
                    )
                })
                .collect();
            let mut hall_of_fame: Vec<(
                f64,
                SensitivityGene,
                EffectObservation,
                SensitivityContext,
            )> = vec![];
            for generation in 0..ga_generations {
                let mut ranked: Vec<_> = population
                    .into_par_iter()
                    .with_max_len(1)
                    .map(|gene| {
                        let scenario_seed = gene.trial_seed;
                        let prepared = prepare_sensitivity(
                            &gene,
                            scenario_seed,
                            sensitivity_screen_wave(perk, max_wave),
                            false,
                        );
                        let observation = evaluate_prepared(&prepared, perk, gene.activation_wave);
                        let fitness = sensitivity_fitness(&observation, objective);
                        (fitness, gene, observation, prepared.context)
                    })
                    .collect();
                targeted_policy_runs += ranked.len() as u64;
                for (_, _, observation, context) in &ranked {
                    targeted[perk].push(observation, *context);
                    targeted_by_objective[perk][objective_index].push(observation, *context);
                }
                ranked.sort_by(|a, b| b.0.total_cmp(&a.0));
                hall_of_fame.extend(ranked.iter().take(64).cloned());
                hall_of_fame.sort_by(|a, b| b.0.total_cmp(&a.0));
                hall_of_fame.truncate(64);
                eprintln!(
                    "sensitivity target {} {} generation {}/{}: best {:.3}",
                    PERK_NAMES[perk],
                    objective.label(),
                    generation + 1,
                    ga_generations,
                    hall_of_fame[0].0
                );
                let elite_count = (ga_population / 8).max(2).min(ranked.len());
                let elites: Vec<_> = ranked[..elite_count]
                    .iter()
                    .map(|(_, gene, _, _)| gene.clone())
                    .collect();
                population = (0..ga_population)
                    .map(|_| {
                        let parent = &elites[rng.next_u64() as usize % elites.len()];
                        mutate_gene(parent, &mut rng)
                    })
                    .collect();
            }
            let finalists = hall_of_fame.len().min(64);
            let finalist_observations: Vec<_> = hall_of_fame
                .par_iter()
                .take(finalists)
                .enumerate()
                .map(|(index, (_, gene, _, _))| {
                    let scenario_seed = seed
                        .wrapping_add(9_000_000_000)
                        .wrapping_add(perk as u64 * 1_000_000)
                        .wrapping_add(objective_index as u64 * 10_000)
                        .wrapping_add(index as u64);
                    let prepared =
                        prepare_sensitivity(gene, scenario_seed, late_screen_wave, false);
                    let observation = evaluate_prepared(&prepared, perk, gene.activation_wave);
                    let fitness = sensitivity_fitness(&observation, objective);
                    (
                        fitness,
                        scenario_seed,
                        gene.clone(),
                        observation,
                        prepared.context,
                    )
                })
                .collect();
            full_policy_runs += finalist_observations.len() as u64;
            for (_, _, _, observation, context) in &finalist_observations {
                full_wave[perk].push(observation, *context);
                full_wave_by_objective[perk][objective_index].push(observation, *context);
            }
            if let Some(best) = hall_of_fame
                .iter()
                .find(|(_, _, observation, _)| observation.ordinary_eligible)
            {
                best_examples[perk][objective_index] =
                    sensitivity_example(&best.1, best.1.trial_seed, best.3, &best.2, objective);
            }
            if let Some(best_full) = finalist_observations
                .iter()
                .filter(|(_, _, _, observation, _)| observation.ordinary_eligible)
                .max_by(|left, right| left.0.total_cmp(&right.0))
            {
                best_full_wave_examples[perk][objective_index] = sensitivity_example(
                    &best_full.2,
                    best_full.1,
                    best_full.4,
                    &best_full.3,
                    objective,
                );
            }
        }
    }

    let broad_pairs = broad_scenarios as u64 * 12;
    let targeted_pairs =
        ga_generations as u64 * ga_population as u64 * 12 * SearchObjective::ALL.len() as u64;
    let full_pairs = full_policy_runs;
    let total_pairs = broad_pairs + targeted_pairs + full_pairs;
    let complete_pair_count = (0..12)
        .map(|perk| {
            broad.perks[perk].complete_pairs()
                + targeted[perk].complete_pairs()
                + full_wave[perk].complete_pairs()
        })
        .sum::<u64>();
    let censored_pair_count = total_pairs - complete_pair_count;
    let pathological_stall_count = (0..12)
        .map(|perk| {
            broad.perks[perk].pathological_stall
                + targeted[perk].pathological_stall
                + full_wave[perk].pathological_stall
        })
        .sum::<u64>();
    let policy_runs = broad.policy_runs + targeted_policy_runs + full_policy_runs;
    let counterfactual_logical_arms = total_pairs * 2;
    let broad_prefix_groups =
        broad_scenarios as u64 * if max_wave < late_screen_wave { 2 } else { 1 };
    let counterfactual_prefix_groups = broad_prefix_groups + targeted_pairs + full_pairs;
    let total_logical_simulations = policy_runs + counterfactual_logical_arms;
    let elapsed = started.elapsed().as_secs_f64();
    let perks = (0..12)
        .map(|perk| {
            let targeted_objectives = SearchObjective::ALL
                .into_iter()
                .enumerate()
                .map(|(index, objective)| {
                    (
                        objective.label().to_owned(),
                        targeted_by_objective[perk][index].json(),
                    )
                })
                .collect::<serde_json::Map<_, _>>();
            let full_objectives = SearchObjective::ALL
                .into_iter()
                .enumerate()
                .map(|(index, objective)| {
                    (
                        objective.label().to_owned(),
                        full_wave_by_objective[perk][index].json(),
                    )
                })
                .collect::<serde_json::Map<_, _>>();
            let objective_examples = SearchObjective::ALL
                .into_iter()
                .enumerate()
                .map(|(index, objective)| {
                    (
                        objective.label().to_owned(),
                        best_examples[perk][index].clone(),
                    )
                })
                .collect::<serde_json::Map<_, _>>();
            let full_objective_examples = SearchObjective::ALL
                .into_iter()
                .enumerate()
                .map(|(index, objective)| {
                    (
                        objective.label().to_owned(),
                        best_full_wave_examples[perk][index].clone(),
                    )
                })
                .collect::<serde_json::Map<_, _>>();
            let context_strata = broad.perk_strata[perk]
                .iter()
                .map(|(key, stats)| (key.clone(), stats.json()))
                .collect::<serde_json::Map<_, _>>();
            json!({
                "id":perk,
                "name":PERK_NAMES[perk],
                "classification":classify_perk(&broad.perks[perk],&targeted[perk],&full_wave[perk]),
                "broad":broad.perks[perk].json(),
                "contextStrata":context_strata,
                "targeted":targeted[perk].json(),
                "targetedByObjective":targeted_objectives,
                "fullWaveValidation":full_wave[perk].json(),
                "fullWaveByObjective":full_objectives,
                "bestExamplesByObjective":objective_examples,
                "bestFullWaveExamplesByObjective":full_objective_examples,
                "bestAdversarialExample":best_examples[perk][2].clone(),
                "bestFullWaveExample":best_full_wave_examples[perk][2].clone(),
            })
        })
        .collect::<Vec<_>>();
    let interactions = broad
        .projectile_interactions
        .iter()
        .map(|(key, stats)| (key.clone(), stats.json()))
        .collect::<serde_json::Map<_, _>>();
    let source_revision_at_completion = source_revision();
    let source_dirty_at_completion = source_dirty();
    let source_stable_across_run = source_revision_at_start == source_revision_at_completion
        && source_dirty_at_start == source_dirty_at_completion;
    let release_gates = json!({
        "minimumThreeMillionPairs":total_pairs>=3_000_000,
        "completePairRateAtLeast99_5Percent":complete_pair_count as f64/total_pairs as f64>=0.995,
        "noPathologicalStalls":pathological_stall_count==0,
        "separateBenefitHarmMechanicalSearches":true,
        "commonRandomNumbers":true,
        "exactPairedInterventions":true,
        "relevantContextStrata":broad.perk_strata.iter().all(|strata|strata.contains_key("mechanic:relevant")),
    });
    let release_gate_pass = release_gates
        .as_object()
        .unwrap()
        .values()
        .all(|value| value.as_bool() == Some(true));
    let report = json!({
        "schema":6,
        "kind":"perk-sensitivity",
        "seed":seed.to_string(),
        "sourceRevision":source_revision_at_start,
        "sourceDirty":source_dirty_at_start,
        "sourceRevisionAtCompletion":source_revision_at_completion,
        "sourceDirtyAtCompletion":source_dirty_at_completion,
        "sourceStableAcrossRun":source_stable_across_run,
        "config":{
            "broadScenarios":broad_scenarios,
            "gaGenerations":ga_generations,
            "gaPopulation":ga_population,
            "gaObjectives":["benefit","harm","mechanical"],
            "gaElitesReevaluated":false,
            "gaCensorAware":true,
            "gaCappedCandidatesRankBelowComplete":true,
            "screenMaxWave":max_wave,
            "lateScreenWave":late_screen_wave,
            "lateScreenPerks":[PERK_NAMES[8],PERK_NAMES[10],PERK_NAMES[11]],
            "fullWaveFinalistsPerPerk":full_pairs/12,
            "corpusDimensions":{
                "maps":[0,1,2],
                "enemySpeedFactors":SPEED_LEVELS,
                "towerRangeFactors":RANGE_LEVELS,
                "enemyHpFactors":HP_LEVELS,
                "enemyCountFactors":COUNT_LEVELS,
                "activationWaves":ACTIVATION_LEVELS,
                "focusTowers":10,
                "policyGenomeDimensions":46,
                "policyVariation":["placements","tower priorities","branches","merges","fusions","doctrines","powerups","early calls","reaction delay","APM","sell timing and target"]
            }
        },
        "counts":{"pairedComparisons":total_pairs,"completePairedComparisons":complete_pair_count,"censoredPairedComparisons":censored_pair_count,"counterfactualLogicalArms":counterfactual_logical_arms,"counterfactualPrefixGroups":counterfactual_prefix_groups,"policyGenerationRuns":policy_runs,"totalLogicalSimulations":total_logical_simulations,"broadPairs":broad_pairs,"targetedPairs":targeted_pairs,"fullWavePairs":full_pairs},
        "releaseGates":{"pass":release_gate_pass,"checks":release_gates,"pathologicalStallPairs":pathological_stall_count},
        "execution":{
            "countUnit":"logical-simulation-arm-output",
            "sharedPreInterventionPrefixes":true,
            "broadPrefixesSharedAcrossPerks":true,
            "policyCheckpointsReused":true,
            "naturalPolicyBaselineArmsReused":true,
            "compactSensitivityResults":true,
            "censoredClearTimeExcluded":true,
            "censoredOrdinaryMetricsExcluded":true,
            "completePairMetricsOnly":true,
            "gaCensorAware":true,
            "gaCappedCandidatesRankBelowComplete":true,
            "tickCap":EXECUTION_TICK_CAP,
            "countSemantics":"Logical arm outputs; common pre-intervention ticks execute once, an already-executed natural policy arm is reused when it is exactly identical, and capped arms remain explicitly censored.",
            "ordinaryMetricSemantics":"Outcome, mechanical, utility, rate, maximum, and best-example evidence uses only pairs where both arms finish before the tick cap."
        },
        "tickCapMetadata":{
            "ticksPerArm":EXECUTION_TICK_CAP,
            "dtSeconds":DT,
            "simulatedSecondsAtCap":EXECUTION_TICK_CAP as f64*DT,
            "metric":"tickCapReached",
            "interpretation":"diagnostic-only"
        },
        "elapsedSeconds":elapsed,
        "threads":rayon::current_num_threads(),
        "logicalSimulationsPerSecond":total_logical_simulations as f64/elapsed.max(f64::MIN_POSITIVE),
        "effectConvention":"Eligible ordinary deltas are perk-on minus perk-off under identical seed, map, params, and action queue. tickCapReached is the diagnostic on-capped minus off-capped indicator.",
        "inferenceCaveat":"Broad-phase ordinary confidence intervals describe uncertainty under the declared seed-driven scenario generator and represented parameter ranges, conditional on both arms finishing before the tick cap. Cap rates quantify the excluded region separately. Adaptive targeted and full-wave confidence intervals are descriptive because the genetic search selected those samples.",
        "censoringCaveat":"When either arm reaches the tick cap, every ordinary outcome, mechanical, utility, rate, maximum, and best-example contribution from that pair is excluded. This also excludes equal-horizon both-capped telemetry so it is not mixed with terminal completed-run metrics. tickCapReached and the cap counts remain diagnostic only and do not enter utility, ordinary effect scores, GA fitness, or benefit/harm direction.",
        "classificationCaveat":"Classifications use complete-pair ordinary evidence. A one-arm cap transition proves execution changed, so it prevents inactive or redundant labels and can supply niche-only activity evidence, but it is never assigned a benefit, harm, or effect magnitude. Both-capped pairs supply no classification evidence. mechanically inactive remains an empirical corpus result, not a mathematical proof over an infinite state space.",
        "classificationRules":{
            "insufficient complete evidence":"the broad phase has no pair where both arms finish before the tick cap",
            "mechanically inactive":"at least one complete pair exists, no measured complete-pair outcome or mechanical delta appears in broad, targeted, or full-wave phases, and no one-arm cap transition occurs",
            "harmful":"upper 95% confidence bound on complete-pair broad mean utility is below -0.1",
            "redundant":"upper 95% Wilson bound on complete-pair broad outcome-change rate is below 0.1%, the maximum eligible adversarial outcome score is below 1, and no one-arm cap transition occurs",
            "niche-only":"complete-pair broad outcome evidence is below 1% but eligible adversarial search finds an outcome score of at least 1, or one-arm cap transitions provide similarly rare direction-unknown activity evidence",
            "meaningful":"does not satisfy the stricter inactive, redundant, niche-only, or harmful rules"
        },
        "perks":perks,
        "projectileSpeedInteractions":interactions,
    });
    if let Some(parent) = std::path::Path::new(output_path).parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(output_path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    println!("{}", serde_json::to_string_pretty(&json!({"output":output_path,"counts":report["counts"],"elapsedSeconds":elapsed,"classifications":perks.iter().map(|perk|json!({"name":perk["name"],"classification":perk["classification"]})).collect::<Vec<_>>()})).unwrap());
}

fn dashboard(path: Option<&String>) {
    let p = path
        .map(String::as_str)
        .unwrap_or("balance-lab/out/report.json");
    let data: Value = serde_json::from_str(&fs::read_to_string(p).unwrap()).unwrap();
    let payload = serde_json::to_string(&data).unwrap().replace("</", "<\\/");
    let html = include_str!("../dashboard.html").replace("__DATA__", &payload);
    fs::create_dir_all("balance-lab/out").unwrap();
    fs::write("balance-lab/out/index.html", html).unwrap();
    println!("balance-lab/out/index.html")
}
fn bench(args: &[String]) {
    let n = args.get(2).and_then(|x| x.parse().ok()).unwrap_or(96);
    let mut rng = Xoshiro::new(20260718);
    let inputs: Vec<_> = (0..n)
        .map(|i| {
            let seed = 20260718 + i as u64 * 104729;
            let genome = Genome::random(&mut rng);
            let policy = run_policy(&genome, seed, (seed % 3) as usize, 50, Params::default());
            Input {
                seed,
                map: (seed % 3) as usize,
                max_wave: 50,
                actions: policy.actions,
                params: None,
                initial: None,
                endless: false,
            }
        })
        .collect();
    let rust_started = Instant::now();
    let rust_runs: Vec<_> = inputs
        .par_iter()
        .map(|input| run(input, Params::default()))
        .collect();
    let rust_seconds = rust_started.elapsed().as_secs_f64();
    let root = env!("CARGO_MANIFEST_DIR").trim_end_matches("/balance-lab");
    let js_started = Instant::now();
    let mut child = Command::new("node")
        .arg(format!("{root}/oracle.js"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();
    serde_json::to_writer(child.stdin.take().unwrap(), &inputs).unwrap();
    assert!(child.wait().unwrap().success(), "JS benchmark failed");
    let js_seconds = js_started.elapsed().as_secs_f64();
    let threads = rayon::current_num_threads();
    let rust_gps = n as f64 / rust_seconds;
    let js_gps = n as f64 / js_seconds;
    let commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned());
    let report = json!({
        "schema":1,"kind":"benchmark","commit":commit,"cases":n,"maxWave":50,
        "waveTraces":rust_runs.iter().map(|run|run.trace.len()).sum::<usize>(),
        "threads":threads,"rustSeconds":rust_seconds,"rustGamesPerSecond":rust_gps,
        "rustGamesPerSecondPerThread":rust_gps/threads as f64,"jsSeconds":js_seconds,
        "jsGamesPerSecond":js_gps,"wallClockSpeedup":rust_gps/js_gps,
        "estimatedPerCoreSpeedup":rust_gps/threads as f64/js_gps
    });
    fs::create_dir_all("balance-lab/out").unwrap();
    fs::write(
        "balance-lab/out/bench-report.json",
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("{}", serde_json::to_string_pretty(&report).unwrap())
}

#[cfg(test)]
mod sensitivity_tests {
    use super::*;

    fn summary(seconds: f64, capped: bool, ticks: usize) -> CounterfactualSummary {
        let telemetry = Telemetry {
            combat_seconds: seconds,
            ..Telemetry::default()
        };
        CounterfactualSummary {
            wave: 20,
            lives: 10,
            gold: 100.,
            kills: 0,
            seconds,
            capped,
            ticks,
            telemetry,
        }
    }

    fn context() -> SensitivityContext {
        SensitivityContext {
            homing: false,
            splash: false,
            composition: "empty",
            enemy_speed_factor: 1.,
            range_factor: 1.,
            hp_factor: 1.,
            count_factor: 1.,
            map: 0,
            activation_wave: 10,
            rush_proximity: "on-rush",
            focus_tower: 0,
            sell_strategy: false,
            sell_slot: 0,
            sell_scheduled: false,
            screen_wave: 25,
            reaction_bucket: "fast",
            reaction_ticks: 0,
            apm: 60.,
        }
    }

    #[test]
    fn complete_pair_retains_clear_seconds_delta() {
        let observation = effect_observation(summary(12.5, false, 375), summary(9.25, false, 278));
        assert_eq!(observation.metrics[CLEAR_SECONDS_METRIC], -3.25);
        assert_eq!(observation.metrics[TICK_CAP_METRIC], 0.);
        assert!(observation.ordinary_eligible);
        assert!(!observation.off_capped);
        assert!(!observation.on_capped);
    }

    #[test]
    fn effective_damage_is_search_evidence_but_not_utility() {
        let off = summary(12.5, false, 375);
        let mut on = off.clone();
        on.telemetry.effective_damage = 1_000_000.;
        let observation = effect_observation(off, on);

        assert_eq!(observation.utility, 0.);
        assert_eq!(observation.outcome_score, 0.);
        assert!(observation.mechanical_score > 1_000.);
        assert_eq!(
            sensitivity_fitness(&observation, SearchObjective::Mechanical),
            observation.mechanical_score * 1_000.
        );
    }

    #[test]
    fn huge_censored_telemetry_cannot_affect_ordinary_stats_maxima_or_fitness() {
        let off = summary(12.5, false, 375);
        let mut on = summary(100_000., true, EXECUTION_TICK_CAP);
        on.wave = 10_000;
        on.lives = -10_000;
        on.gold = 1e15;
        on.telemetry.leaks = 1_000_000_000;
        on.telemetry.leak_damage = i64::MAX / 2;
        on.telemetry.effective_damage = 1e30;
        on.telemetry.overkill_damage = 1e30;
        on.telemetry.projectile_impacts = 1;
        on.telemetry.projectile_latency_seconds = 1e20;
        on.telemetry.wasted_projectiles = 1_000_000_000;
        let censored = effect_observation(off, on);

        assert!(!censored.ordinary_eligible);
        assert!(
            censored.metrics[..TICK_CAP_METRIC]
                .iter()
                .all(|value| *value == 0.)
        );
        assert_eq!(censored.metrics[TICK_CAP_METRIC], 1.);
        assert_eq!(censored.utility, 0.);
        assert_eq!(censored.outcome_score, 0.);
        assert_eq!(censored.mechanical_score, 0.);
        assert_eq!(
            sensitivity_fitness(&censored, SearchObjective::Mechanical),
            f64::NEG_INFINITY
        );

        let mut stats = EffectStats::new();
        stats.push(&censored, context());
        assert_eq!(stats.pairs, 1);
        assert_eq!(stats.complete_pairs(), 0);
        assert_eq!(stats.pair_censored, 1);
        assert_eq!(stats.on_capped, 1);
        assert_eq!(stats.cap_transition, 1);
        assert_eq!(stats.utility.n, 0);
        assert!(
            stats.metrics[..TICK_CAP_METRIC]
                .iter()
                .all(|moment| moment.n == 0)
        );
        assert_eq!(stats.metrics[TICK_CAP_METRIC].n, 1);
        assert_eq!(stats.outcome_changed, 0);
        assert_eq!(stats.mechanical_changed, 0);
        assert_eq!(stats.max_outcome_score, 0.);
        assert_eq!(stats.max_mechanical_score, 0.);
        assert!(stats.max_outcome_context.is_none());
        assert!(stats.max_mechanical_context.is_none());
    }

    #[test]
    fn complete_pair_counts_and_rates_exclude_censored_pairs() {
        let exact_zero = effect_observation(summary(12.5, false, 375), summary(12.5, false, 375));
        let off = summary(12.5, false, 375);
        let mut on = off.clone();
        on.gold += 10.;
        on.telemetry.effective_damage = 500.;
        let changed = effect_observation(off, on);
        let capped = effect_observation(
            summary(12.5, false, 375),
            summary(100_000., true, EXECUTION_TICK_CAP),
        );

        let mut stats = EffectStats::new();
        stats.push(&exact_zero, context());
        stats.push(&changed, context());
        stats.push(&capped, context());

        let report = stats.json();
        assert_eq!(report["pairs"], 3);
        assert_eq!(report["completePairs"], 2);
        assert_eq!(report["pairCensored"], 1);
        assert_eq!(report["capTransition"], 1);
        assert_eq!(report["bothCapped"], 0);
        assert_eq!(report["exactZero"], 1);
        assert_eq!(report["exactZeroRate"], 0.5);
        assert_eq!(report["outcomeChanged"], 1);
        assert_eq!(report["outcomeChangeRate"], 0.5);
        assert_eq!(report["mechanicalChanged"], 1);
        assert_eq!(report["mechanicalChangeRate"], 0.5);
        assert_eq!(report["positive"], 1);
        assert_eq!(report["positiveRate"], 0.5);
        assert_eq!(report["negative"], 0);
        assert_eq!(report["utility"]["samples"], 2);
        assert_eq!(report["metrics"]["effectiveDamage"]["samples"], 2);
        assert_eq!(report["metrics"]["tickCapReached"]["samples"], 3);
    }

    #[test]
    fn all_censored_stats_serialize_as_finite_zero_sample_data() {
        let transition = effect_observation(
            summary(12.5, false, 375),
            summary(100_000., true, EXECUTION_TICK_CAP),
        );
        let both_capped = effect_observation(
            summary(100_000., true, EXECUTION_TICK_CAP),
            summary(100_000., true, EXECUTION_TICK_CAP),
        );
        let mut stats = EffectStats::new();
        stats.push(&transition, context());
        stats.push(&both_capped, context());

        let report = stats.json();
        assert_eq!(report["pairs"], 2);
        assert_eq!(report["completePairs"], 0);
        assert_eq!(report["pairCensored"], 2);
        assert_eq!(report["capTransition"], 1);
        assert_eq!(report["bothCapped"], 1);
        for metric in &EFFECT_METRICS[..TICK_CAP_METRIC] {
            let moment = &report["metrics"][metric];
            assert_eq!(moment["samples"], 0);
            assert_eq!(moment["mean"], 0.);
            assert_eq!(moment["min"], 0.);
            assert_eq!(moment["max"], 0.);
            assert_eq!(moment["ci95"], json!([0., 0.]));
        }
        assert_eq!(report["utility"]["samples"], 0);
        assert_eq!(report["metrics"]["tickCapReached"]["samples"], 2);
        assert!(serde_json::to_vec(&report).is_ok());
    }

    #[test]
    fn all_censored_broad_phase_is_not_given_an_effect_classification() {
        let censored = effect_observation(
            summary(100_000., true, EXECUTION_TICK_CAP),
            summary(100_000., true, EXECUTION_TICK_CAP),
        );
        let mut broad = EffectStats::new();
        broad.push(&censored, context());

        assert_eq!(
            classify_perk(&broad, &EffectStats::new(), &EffectStats::new()),
            "insufficient complete evidence"
        );
    }

    #[test]
    fn rare_cap_transition_is_activity_evidence_without_a_utility_direction() {
        let exact_zero = effect_observation(summary(12.5, false, 375), summary(12.5, false, 375));
        let transition = effect_observation(
            summary(12.5, false, 375),
            summary(100_000., true, EXECUTION_TICK_CAP),
        );
        let mut broad = EffectStats::new();
        for _ in 0..1_000 {
            broad.push(&exact_zero, context());
        }
        broad.push(&transition, context());

        assert_eq!(broad.utility.n, 1_000);
        assert_eq!(broad.cap_transition, 1);
        assert_eq!(
            classify_perk(&broad, &EffectStats::new(), &EffectStats::new()),
            "niche-only"
        );
    }

    #[test]
    fn harmful_complete_pair_evidence_precedes_redundant_and_cap_niche() {
        let off = summary(12.5, false, 375);
        let mut on = off.clone();
        on.telemetry.wasted_projectiles = 100;
        let harmful = effect_observation(off, on);
        let transition = effect_observation(
            summary(12.5, false, 375),
            summary(100_000., true, EXECUTION_TICK_CAP),
        );
        let mut broad = EffectStats::new();
        for _ in 0..4_000 {
            broad.push(&harmful, context());
        }

        assert!(broad.utility.ci95().1 < -0.1);
        assert!(wilson95(broad.outcome_changed, broad.complete_pairs())[1] < 0.001);
        assert_eq!(
            classify_perk(&broad, &EffectStats::new(), &EffectStats::new()),
            "harmful"
        );

        broad.push(&transition, context());
        assert_eq!(broad.cap_transition, 1);
        assert_eq!(
            classify_perk(&broad, &EffectStats::new(), &EffectStats::new()),
            "harmful"
        );
    }
}
