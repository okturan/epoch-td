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
        _ => help(),
    }
}
fn help() {
    eprintln!(
        "epoch-lab <run [case.json]|batch [jobs.json]|calibrate [traces.json]|golden [cases.json]|policy-golden [cases seed]|attack [generations population seed calibration.json]|defend [generations candidates policies seed calibration.json]|sensitivity [broad-scenarios ga-generations ga-population seed max-wave]|dashboard [report.json]|bench>"
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
const EFFECT_METRICS: [&str; 10] = [
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
}

#[derive(Clone)]
struct EffectObservation {
    metrics: [f64; 10],
    utility: f64,
    outcome_score: f64,
    mechanical_score: f64,
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
        let (low, high) = self.ci95();
        json!({"mean":self.mean(),"ci95":[low,high],"min":self.min,"max":self.max,"samples":self.n})
    }
}

#[derive(Clone)]
struct EffectStats {
    pairs: u64,
    exact_zero: u64,
    outcome_changed: u64,
    mechanical_changed: u64,
    positive: u64,
    negative: u64,
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
            exact_zero: 0,
            outcome_changed: 0,
            mechanical_changed: 0,
            positive: 0,
            negative: 0,
            utility: Moment::new(),
            metrics: (0..EFFECT_METRICS.len()).map(|_| Moment::new()).collect(),
            max_outcome_score: 0.,
            max_mechanical_score: 0.,
            max_outcome_context: None,
            max_mechanical_context: None,
        }
    }
    fn push(&mut self, observation: &EffectObservation, context: SensitivityContext) {
        const EPS: f64 = 1e-9;
        self.pairs += 1;
        self.utility.push(observation.utility);
        for (stats, value) in self.metrics.iter_mut().zip(observation.metrics) {
            stats.push(value)
        }
        if observation.metrics.iter().all(|value| value.abs() <= EPS) {
            self.exact_zero += 1
        }
        if observation.metrics[..6]
            .iter()
            .any(|value| value.abs() > EPS)
        {
            self.outcome_changed += 1
        }
        if observation.metrics[6..]
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
        self.exact_zero += other.exact_zero;
        self.outcome_changed += other.outcome_changed;
        self.mechanical_changed += other.mechanical_changed;
        self.positive += other.positive;
        self.negative += other.negative;
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
        let outcome_change_ci = wilson95(self.outcome_changed, self.pairs);
        let mechanical_change_ci = wilson95(self.mechanical_changed, self.pairs);
        let metrics = EFFECT_METRICS
            .iter()
            .zip(&self.metrics)
            .map(|(name, stats)| ((*name).to_string(), stats.json()))
            .collect::<serde_json::Map<_, _>>();
        json!({
            "pairs":self.pairs,
            "exactZeroRate":self.exact_zero as f64/self.pairs.max(1) as f64,
            "outcomeChangeRate":self.outcome_changed as f64/self.pairs.max(1) as f64,
            "outcomeChangeRateCi95":outcome_change_ci,
            "mechanicalChangeRate":self.mechanical_changed as f64/self.pairs.max(1) as f64,
            "mechanicalChangeRateCi95":mechanical_change_ci,
            "positiveRate":self.positive as f64/self.pairs.max(1) as f64,
            "negativeRate":self.negative as f64/self.pairs.max(1) as f64,
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
    projectile_interactions: BTreeMap<String, EffectStats>,
    policy_runs: u64,
}
impl SensitivityAccumulator {
    fn new() -> Self {
        Self {
            perks: (0..12).map(|_| EffectStats::new()).collect(),
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
    ) {
        self.perks[perk].push(observation, context);
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
    }
}

fn mean_latency(telemetry: &Telemetry) -> f64 {
    telemetry.projectile_latency_seconds / telemetry.projectile_impacts.max(1) as f64
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
    let metrics = [
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
    ];
    let utility = metrics[0] * 100. + metrics[1] * 5. + metrics[2] * 0.02
        - metrics[3] * 2.
        - metrics[4] * 3.
        - metrics[5] * 0.05
        + metrics[6] * 0.001
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
        + metrics[9].abs();
    EffectObservation {
        metrics,
        utility,
        outcome_score,
        mechanical_score,
    }
}

fn sensitivity_example(
    gene: &SensitivityGene,
    scenario_seed: u64,
    context: SensitivityContext,
    observation: &EffectObservation,
) -> Value {
    let fitness = observation.outcome_score * 1_000. + observation.mechanical_score;
    json!({
        "fitness":fitness,
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
    let outcome_rate_ci = wilson95(broad.outcome_changed, broad.pairs);
    let all_mechanical =
        broad.mechanical_changed + targeted.mechanical_changed + full.mechanical_changed;
    let max_outcome = broad
        .max_outcome_score
        .max(targeted.max_outcome_score)
        .max(full.max_outcome_score);
    let (_, utility_high) = broad.utility.ci95();
    if all_mechanical == 0
        && broad.outcome_changed + targeted.outcome_changed + full.outcome_changed == 0
    {
        "mechanically inactive"
    } else if outcome_rate_ci[1] < 0.001 && max_outcome < 1. {
        "redundant"
    } else if outcome_rate_ci[1] < 0.01 && max_outcome >= 1. {
        "niche-only"
    } else if utility_high < -0.1 {
        "harmful"
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
    assert!(broad_scenarios > 0 && ga_generations > 0 && ga_population >= 2);
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
    let mut best_examples = vec![Value::Null; 12];
    let mut best_full_wave_examples = vec![Value::Null; 12];
    let mut targeted_policy_runs = 0u64;
    let mut full_policy_runs = 0u64;
    for perk in 0..12 {
        let mut rng = Xoshiro::new(seed ^ (perk as u64 + 1).wrapping_mul(0xd1b54a32d192ed03));
        let mut population: Vec<_> = (0..ga_population)
            .map(|index| broad_gene(seed ^ perk as u64, index + perk * ga_population))
            .collect();
        let mut hall_of_fame: Vec<(f64, SensitivityGene, EffectObservation, SensitivityContext)> =
            vec![];
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
                    let fitness = observation.outcome_score * 1_000. + observation.mechanical_score;
                    (fitness, gene, observation, prepared.context)
                })
                .collect();
            targeted_policy_runs += ranked.len() as u64;
            for (_, _, observation, context) in &ranked {
                targeted[perk].push(observation, *context)
            }
            ranked.sort_by(|a, b| b.0.total_cmp(&a.0));
            hall_of_fame.extend(ranked.iter().take(64).cloned());
            hall_of_fame.sort_by(|a, b| b.0.total_cmp(&a.0));
            hall_of_fame.truncate(64);
            eprintln!(
                "sensitivity target {} generation {}/{}: best {:.3}",
                PERK_NAMES[perk],
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
                    .wrapping_add(perk as u64 * 10_000)
                    .wrapping_add(index as u64);
                let prepared = prepare_sensitivity(gene, scenario_seed, late_screen_wave, false);
                let observation = evaluate_prepared(&prepared, perk, gene.activation_wave);
                let fitness = observation.outcome_score * 1_000. + observation.mechanical_score;
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
        }
        let best = &hall_of_fame[0];
        best_examples[perk] = sensitivity_example(&best.1, best.1.trial_seed, best.3, &best.2);
        let best_full = finalist_observations
            .iter()
            .max_by(|left, right| left.0.total_cmp(&right.0))
            .unwrap();
        best_full_wave_examples[perk] =
            sensitivity_example(&best_full.2, best_full.1, best_full.4, &best_full.3);
    }

    let broad_pairs = broad_scenarios as u64 * 12;
    let targeted_pairs = ga_generations as u64 * ga_population as u64 * 12;
    let full_pairs = full_policy_runs;
    let total_pairs = broad_pairs + targeted_pairs + full_pairs;
    let policy_runs = broad.policy_runs + targeted_policy_runs + full_policy_runs;
    let counterfactual_logical_arms = total_pairs * 2;
    let broad_prefix_groups =
        broad_scenarios as u64 * if max_wave < late_screen_wave { 2 } else { 1 };
    let counterfactual_prefix_groups = broad_prefix_groups + targeted_pairs + full_pairs;
    let total_logical_simulations = policy_runs + counterfactual_logical_arms;
    let elapsed = started.elapsed().as_secs_f64();
    let perks = (0..12)
        .map(|perk| {
            json!({
                "id":perk,
                "name":PERK_NAMES[perk],
                "classification":classify_perk(&broad.perks[perk],&targeted[perk],&full_wave[perk]),
                "broad":broad.perks[perk].json(),
                "targeted":targeted[perk].json(),
                "fullWaveValidation":full_wave[perk].json(),
                "bestAdversarialExample":best_examples[perk],
                "bestFullWaveExample":best_full_wave_examples[perk],
            })
        })
        .collect::<Vec<_>>();
    let interactions = broad
        .projectile_interactions
        .iter()
        .map(|(key, stats)| (key.clone(), stats.json()))
        .collect::<serde_json::Map<_, _>>();
    let report = json!({
        "schema":3,
        "kind":"perk-sensitivity",
        "seed":seed.to_string(),
        "config":{
            "broadScenarios":broad_scenarios,
            "gaGenerations":ga_generations,
            "gaPopulation":ga_population,
            "gaElitesReevaluated":false,
            "screenMaxWave":max_wave,
            "lateScreenWave":late_screen_wave,
            "lateScreenPerks":[PERK_NAMES[8],PERK_NAMES[10],PERK_NAMES[11]],
            "fullWaveFinalistsPerPerk":ga_population.min(64),
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
        "counts":{"pairedComparisons":total_pairs,"counterfactualLogicalArms":counterfactual_logical_arms,"counterfactualPrefixGroups":counterfactual_prefix_groups,"policyGenerationRuns":policy_runs,"totalLogicalSimulations":total_logical_simulations,"broadPairs":broad_pairs,"targetedPairs":targeted_pairs,"fullWavePairs":full_pairs},
        "execution":{
            "countUnit":"logical-complete-simulation-output",
            "sharedPreInterventionPrefixes":true,
            "broadPrefixesSharedAcrossPerks":true,
            "policyCheckpointsReused":true,
            "naturalPolicyBaselineArmsReused":true,
            "compactSensitivityResults":true,
            "countSemantics":"Logical complete outputs; common pre-intervention ticks execute once, and an already-executed natural policy arm is reused when it is exactly identical."
        },
        "elapsedSeconds":elapsed,
        "threads":rayon::current_num_threads(),
        "logicalSimulationsPerSecond":total_logical_simulations as f64/elapsed.max(f64::MIN_POSITIVE),
        "effectConvention":"all deltas are perk-on minus perk-off under identical seed, map, params, and action queue",
        "inferenceCaveat":"Broad-phase confidence intervals are inferential for the fixed stratified corpus; adaptive targeted and full-wave confidence intervals are descriptive because the genetic search selected those samples.",
        "classificationCaveat":"mechanically inactive means no observed effect in this executed corpus and adversarial search, not a mathematical proof over an infinite state space",
        "classificationRules":{
            "mechanically inactive":"no measured outcome or mechanical delta in broad, targeted, or full-wave phases",
            "redundant":"upper 95% Wilson bound on broad outcome-change rate is below 0.1% and adversarial outcome score is below 1",
            "niche-only":"upper 95% Wilson bound on broad outcome-change rate is below 1% but adversarial search finds an outcome score of at least 1",
            "harmful":"upper 95% confidence bound on broad mean utility is below -0.1",
            "meaningful":"does not satisfy the stricter inactive, redundant, niche-only, or harmful rules"
        },
        "perks":perks,
        "projectileSpeedInteractions":interactions,
    });
    fs::create_dir_all("balance-lab/out").unwrap();
    fs::write(
        "balance-lab/out/sensitivity-report.json",
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("{}", serde_json::to_string_pretty(&json!({"output":"balance-lab/out/sensitivity-report.json","counts":report["counts"],"elapsedSeconds":elapsed,"classifications":perks.iter().map(|perk|json!({"name":perk["name"],"classification":perk["classification"]})).collect::<Vec<_>>()})).unwrap());
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
