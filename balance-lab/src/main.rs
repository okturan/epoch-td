use epoch_core::*;
use rayon::prelude::*;
use serde_json::{Value, json};
use std::{
    env, fs,
    io::{self, Read},
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
                serde_json::to_string(&run(&input, Params::default())).unwrap()
            )
        }
        Some("golden") => golden(args.get(2)),
        Some("attack") => attack(&args),
        Some("dashboard") => dashboard(args.get(2)),
        Some("bench") => bench(),
        _ => help(),
    }
}
fn help() {
    eprintln!(
        "epoch-lab <run [case.json]|golden [cases.json]|attack [generations population seed]|dashboard [report.json]|bench>"
    )
}

fn golden(path: Option<&String>) {
    let inputs: Vec<Input> = serde_json::from_str(&read_input(path)).unwrap();
    let rust: Vec<_> = inputs.iter().map(|i| run(i, Params::default())).collect();
    let root = env!("CARGO_MANIFEST_DIR").trim_end_matches("/balance-lab");
    let mut child = Command::new("node")
        .arg(format!("{root}/oracle.js"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    serde_json::to_writer(child.stdin.take().unwrap(), &inputs).unwrap();
    let out = child.wait_with_output().unwrap();
    if !out.status.success() {
        panic!("JS oracle failed")
    };
    let js: Vec<Value> = serde_json::from_slice(&out.stdout).unwrap();
    let mut differences = vec![];
    for (i, (r, j)) in rust.iter().zip(js.iter()).enumerate() {
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
            if rr.lives != jj["lives"].as_i64().unwrap() as i32
                || rr.kills != jj["kills"].as_u64().unwrap()
                || gold > 1e-4
                || seconds > 1e-4
            {
                differences.push(format!("case {i} wave {}: rust lives={} gold={} kills={} t={}, js lives={} gold={} kills={} t={}",w+1,rr.lives,rr.gold,rr.kills,rr.seconds,jj["lives"],jj["gold"],jj["kills"],jj["seconds"]));
                break;
            }
        }
    }
    if differences.is_empty() {
        println!(
            "golden: PASS ({} cases, {} wave traces)",
            inputs.len(),
            rust.iter().map(|r| r.trace.len()).sum::<usize>()
        )
    } else {
        for d in differences.iter().take(20) {
            eprintln!("{d}")
        }
        panic!("golden: FAIL ({} cases differ)", differences.len())
    }
}

fn scripted(genome: &Genome, seed: u64, max_wave: usize) -> Input {
    let cells = placement_cells((seed % 3) as usize, genome.placement);
    let mut actions = vec![];
    let mut used = 0;
    for wave in 0..max_wave {
        let unlocked = (wave / 5).min(9);
        let tower = (0..=unlocked)
            .max_by(|&a, &b| genome.tower_weights[a].total_cmp(&genome.tower_weights[b]))
            .unwrap();
        if wave % 2 == 0 && used < cells.len() {
            let (x, y) = cells[used];
            used += 1;
            actions.push(Action {
                wave,
                op: "place".into(),
                tower,
                x,
                y,
                branch: 0,
                with: None,
            })
        }
        if genome.upgrade_weight > 0. && used > 0 {
            let (x, y) = cells[(wave / 3).min(used - 1)];
            actions.push(Action {
                wave,
                op: "upgrade".into(),
                tower: 0,
                x,
                y,
                branch: 0,
                with: None,
            });
            if wave % 5 == 4 {
                actions.push(Action {
                    wave,
                    op: "branch".into(),
                    tower: 0,
                    x,
                    y,
                    branch: if genome.branch_bias > 0. { 1 } else { 2 },
                    with: None,
                })
            }
        }
    }
    Input {
        seed,
        map: (seed % 3) as usize,
        max_wave,
        actions,
    }
}
fn mutate(g: &Genome, r: &mut Xoshiro) -> Genome {
    let mut n = g.clone();
    for w in &mut n.tower_weights {
        if r.next_f64() < 0.35 {
            *w += (r.next_f64() - 0.5) * 0.7
        }
    }
    for x in &mut n.placement {
        if r.next_f64() < 0.4 {
            *x += (r.next_f64() - 0.5) * 0.3
        }
    }
    for x in &mut n.fusion_weights {
        if r.next_f64() < 0.4 {
            *x += (r.next_f64() - 0.5) * 0.5
        }
    }
    n.upgrade_weight += (r.next_f64() - 0.5) * 0.3;
    n.branch_bias += (r.next_f64() - 0.5) * 0.3;
    n.merge_weight += (r.next_f64() - 0.5) * 0.3;
    n
}
fn attack(args: &[String]) {
    let generations = args.get(2).and_then(|x| x.parse().ok()).unwrap_or(20);
    let population = args.get(3).and_then(|x| x.parse().ok()).unwrap_or(256);
    let seed = args.get(4).and_then(|x| x.parse().ok()).unwrap_or(1);
    let mut rng = Xoshiro::new(seed);
    let mut genomes: Vec<_> = (0..population).map(|_| Genome::random(&mut rng)).collect();
    let started = Instant::now();
    let mut archive = Archive::new();
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
                (g.clone(), s, pr, fitness)
            })
            .collect();
        for (g, s, pr, fitness) in evaluated {
            let mono = (0..10)
                .max_by(|&a, &b| g.tower_weights[a].total_cmp(&g.tower_weights[b]))
                .unwrap();
            let e = Elite {
                fitness,
                wave: pr.run.result.wave,
                lives: pr.run.result.lives,
                mono_tower: mono,
                no_merge: pr.merges == 0,
                fusion: pr.fusion_counts.iter().position(|&n| n > 0),
                genome: g,
                seed: s,
                actions: pr.actions,
            };
            archive
                .entry((mono, e.no_merge, e.fusion))
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
        history
            .push(json!({"generation":generation,"bestFitness":best,"viableCells":archive.len()}));
        eprintln!(
            "generation {generation}: best {best:0.1}, cells {}",
            archive.len()
        );
        let elites: Vec<_> = archive.values().cloned().collect();
        genomes = (0..population)
            .map(|i| mutate(&elites[i % elites.len()].genome, &mut rng))
            .collect();
    }
    let report = json!({"schema":1,"kind":"attack","seed":seed,"generations":generations,"population":population,"elapsedSeconds":started.elapsed().as_secs_f64(),"games":generations*population,"gamesPerSecond":generations as f64*population as f64/started.elapsed().as_secs_f64(),"history":history,"archive":archive.values().collect::<Vec<_>>()});
    fs::create_dir_all("balance-lab/out").unwrap();
    fs::write(
        "balance-lab/out/report.json",
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("balance-lab/out/report.json")
}
fn dashboard(path: Option<&String>) {
    let p = path
        .map(String::as_str)
        .unwrap_or("balance-lab/out/report.json");
    let data: Value = serde_json::from_str(&fs::read_to_string(p).unwrap()).unwrap();
    let payload = serde_json::to_string(&data).unwrap().replace("</", "<\\/");
    let html = format!(
        r#"<!doctype html><meta charset=utf-8><title>EPOCH Balance Lab</title><style>body{{font:14px system-ui;background:#15151b;color:#ddd;max-width:1100px;margin:30px auto}}h1{{color:#ffd870}}.cards{{display:flex;gap:12px}}.card{{background:#24242e;padding:16px;border-radius:9px;flex:1}}svg{{background:#202029;border-radius:8px}}table{{width:100%;border-collapse:collapse}}td,th{{padding:7px;border-bottom:1px solid #333;text-align:left}}</style><h1>EPOCH Balance Lab</h1><div class=cards id=cards></div><h2>Exploitability / best fitness</h2><svg id=trend width=1050 height=260></svg><h2>MAP-Elites archive</h2><div id=heat></div><h2>Strongest strategies</h2><table id=table></table><script>const D={payload};cards.innerHTML=`<div class=card><b>${{D.games.toLocaleString()}}</b><br>games</div><div class=card><b>${{D.gamesPerSecond.toFixed(0)}}</b><br>games/sec</div><div class=card><b>${{D.archive.length}}</b><br>viable niches</div>`;let h=D.history,m=Math.max(...h.map(x=>x.bestFitness)),pts=h.map((x,i)=>`${{20+i*1010/Math.max(1,h.length-1)}},${{235-x.bestFitness/m*210}}`).join(' ');trend.innerHTML=`<polyline fill=none stroke=#ffd870 stroke-width=3 points="${{pts}}"/>`;heat.innerHTML=D.archive.map(e=>`<span title="fitness ${{e.fitness.toFixed(0)}}" style="display:inline-block;margin:3px;padding:18px;background:hsl(${{e.wave*5}},55%,38%)">T${{e.mono_tower+1}} · W${{e.wave}}</span>`).join('');table.innerHTML='<tr><th>Niche</th><th>Wave</th><th>Lives</th><th>Fitness</th><th>Seed</th></tr>'+D.archive.sort((a,b)=>b.fitness-a.fitness).map(e=>`<tr><td>mono tower ${{e.mono_tower+1}}</td><td>${{e.wave}}</td><td>${{e.lives}}</td><td>${{e.fitness.toFixed(1)}}</td><td>${{e.seed}}</td></tr>`).join('')</script>"#
    );
    fs::create_dir_all("balance-lab/out").unwrap();
    fs::write("balance-lab/out/index.html", html).unwrap();
    println!("balance-lab/out/index.html")
}
fn bench() {
    let input = Input {
        seed: 1,
        map: 0,
        max_wave: 25,
        actions: vec![],
    };
    let n = 500;
    let t = Instant::now();
    (0..n).into_par_iter().for_each(|_| {
        let _ = run(&input, Params::default());
    });
    let s = t.elapsed().as_secs_f64();
    println!(
        "{n} partial games in {s:0.3}s = {:0.1} games/s",
        n as f64 / s
    )
}
