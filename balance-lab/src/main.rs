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
        _ => help(),
    }
}
fn help() {
    eprintln!(
        "epoch-lab <run [case.json]|batch [jobs.json]|calibrate [traces.json]|golden [cases.json]|policy-golden [cases seed]|attack [generations population seed calibration.json]|defend [generations candidates policies seed calibration.json]|dashboard [report.json]|bench>"
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
