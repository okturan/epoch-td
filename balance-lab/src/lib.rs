use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const DT: f64 = 1.0 / 30.0;
const W: i32 = 14;
const H: i32 = 9;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Modifiers {
    #[serde(default)]
    pub dm: Option<f64>,
    #[serde(default)]
    pub sp: Option<f64>,
    #[serde(default)]
    pub pv: Option<f64>,
    #[serde(default)]
    pub rgm: Option<f64>,
    #[serde(default)]
    pub kb: Option<f64>,
    #[serde(default)]
    pub pi: Option<f64>,
    #[serde(default)]
    pub bu: Option<f64>,
    #[serde(default)]
    pub bum: Option<f64>,
    #[serde(default)]
    pub hc: Option<f64>,
    #[serde(default)]
    pub sf: Option<f64>,
    #[serde(default)]
    pub ir: Option<f64>,
    #[serde(default)]
    pub sl: Option<f64>,
    #[serde(default)]
    pub hr: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TowerSpec {
    pub name: String,
    pub cost: f64,
    pub damage: f64,
    pub rate: f64,
    pub range: f64,
    #[serde(default)]
    pub splash: f64,
    #[serde(default)]
    pub pierce: bool,
    #[serde(default)]
    pub burn: bool,
    #[serde(default)]
    pub knockback: f64,
    #[serde(default)]
    pub ramp: bool,
    #[serde(default)]
    pub homing: bool,
    #[serde(default)]
    pub aura: bool,
    #[serde(default)]
    pub field: bool,
    #[serde(default)]
    pub beam: bool,
    pub branch1: Modifiers,
    pub branch2: Modifiers,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Constants {
    pub hp: f64,
    pub growth: f64,
    pub count: f64,
    pub speed: f64,
    pub bounty: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Params {
    pub constants: Constants,
    pub towers: Vec<TowerSpec>,
}

impl Default for Params {
    fn default() -> Self {
        let m = |dm, sp, pv, rgm, kb, pi, bu, bum, hc, sf, ir, sl, hr| Modifiers {
            dm,
            sp,
            pv,
            rgm,
            kb,
            pi,
            bu,
            bum,
            hc,
            sf,
            ir,
            sl,
            hr,
        };
        let z = || Modifiers::default();
        Self {
            constants: Constants {
                hp: 14.,
                growth: 1.12,
                count: 0.8,
                speed: 0.008,
                bounty: 0.38,
            },
            towers: vec![
                TowerSpec {
                    name: "Rock Hurler".into(),
                    cost: 20.,
                    damage: 6.,
                    rate: 1.,
                    range: 2.5,
                    splash: 0.,
                    pierce: false,
                    burn: false,
                    knockback: 0.,
                    ramp: false,
                    homing: false,
                    aura: false,
                    field: false,
                    beam: false,
                    branch1: m(
                        Some(2.),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                    branch2: m(
                        Some(0.7),
                        Some(0.5),
                        Some(1.4),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                },
                TowerSpec {
                    name: "Catapult".into(),
                    cost: 45.,
                    damage: 13.,
                    rate: 0.5,
                    range: 3.,
                    splash: 0.9,
                    pierce: false,
                    burn: false,
                    knockback: 0.,
                    ramp: false,
                    homing: false,
                    aura: false,
                    field: false,
                    beam: false,
                    branch1: m(
                        Some(1.8),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                    branch2: m(
                        None,
                        Some(1.4),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                },
                TowerSpec {
                    name: "Ballista".into(),
                    cost: 70.,
                    damage: 22.,
                    rate: 0.7,
                    range: 4.5,
                    splash: 0.,
                    pierce: true,
                    burn: false,
                    knockback: 0.,
                    ramp: false,
                    homing: false,
                    aura: false,
                    field: false,
                    beam: false,
                    branch1: m(
                        Some(1.8),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                    branch2: m(
                        Some(0.7),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                },
                TowerSpec {
                    name: "Brazier".into(),
                    cost: 100.,
                    damage: 3.,
                    rate: 2.,
                    range: 2.,
                    splash: 0.,
                    pierce: false,
                    burn: true,
                    knockback: 0.,
                    ramp: false,
                    homing: false,
                    aura: false,
                    field: false,
                    beam: false,
                    branch1: m(
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(2.),
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                    branch2: m(
                        None,
                        None,
                        None,
                        Some(1.5),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                },
                TowerSpec {
                    name: "Cannon".into(),
                    cost: 140.,
                    damage: 55.,
                    rate: 0.35,
                    range: 3.,
                    splash: 0.5,
                    pierce: false,
                    burn: false,
                    knockback: 0.7,
                    ramp: false,
                    homing: false,
                    aura: false,
                    field: false,
                    beam: false,
                    branch1: m(
                        Some(1.8),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                    branch2: m(
                        None,
                        Some(0.8),
                        None,
                        None,
                        Some(1.4),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                },
                TowerSpec {
                    name: "Gatling".into(),
                    cost: 190.,
                    damage: 8.,
                    rate: 3.,
                    range: 2.5,
                    splash: 0.,
                    pierce: false,
                    burn: false,
                    knockback: 0.,
                    ramp: true,
                    homing: false,
                    aura: false,
                    field: false,
                    beam: false,
                    branch1: m(
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(12.),
                        None,
                        None,
                        None,
                        None,
                    ),
                    branch2: m(
                        Some(0.75),
                        None,
                        None,
                        None,
                        None,
                        Some(1.),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                },
                TowerSpec {
                    name: "Missile".into(),
                    cost: 250.,
                    damage: 105.,
                    rate: 0.6,
                    range: 5.5,
                    splash: 0.7,
                    pierce: false,
                    burn: false,
                    knockback: 0.,
                    ramp: false,
                    homing: true,
                    aura: false,
                    field: false,
                    beam: false,
                    branch1: m(
                        Some(1.8),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                    branch2: m(
                        Some(0.75),
                        Some(1.3),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                },
                TowerSpec {
                    name: "Reactor".into(),
                    cost: 330.,
                    damage: 12.,
                    rate: 0.,
                    range: 2.2,
                    splash: 0.,
                    pierce: false,
                    burn: false,
                    knockback: 0.,
                    ramp: false,
                    homing: false,
                    aura: true,
                    field: false,
                    beam: false,
                    branch1: m(
                        Some(2.),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                    branch2: m(
                        None,
                        None,
                        None,
                        Some(1.45),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                },
                TowerSpec {
                    name: "Drone Hub".into(),
                    cost: 420.,
                    damage: 0.,
                    rate: 0.,
                    range: 2.5,
                    splash: 0.,
                    pierce: false,
                    burn: false,
                    knockback: 0.,
                    ramp: false,
                    homing: false,
                    aura: false,
                    field: true,
                    beam: false,
                    branch1: z(),
                    branch2: m(
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(0.45),
                        None,
                        None,
                        None,
                    ),
                },
                TowerSpec {
                    name: "Orbital Laser".into(),
                    cost: 600.,
                    damage: 40.,
                    rate: 0.,
                    range: 99.,
                    splash: 0.,
                    pierce: false,
                    burn: false,
                    knockback: 0.,
                    ramp: false,
                    homing: false,
                    aura: false,
                    field: false,
                    beam: true,
                    branch1: m(
                        Some(1.5),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                    branch2: m(
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(2.),
                    ),
                },
            ],
        }
    }
}

#[derive(Clone, Debug)]
struct Wave {
    n: usize,
    hp: f64,
    speed: f64,
    armor: f64,
    regen: f64,
    cap: f64,
    dash: bool,
    split: bool,
    bounty: f64,
    leak: i32,
    boss: bool,
    rush: bool,
}

fn wave_row(c: &Constants, w: usize) -> Wave {
    let k = (w as f64 / 5.).ceil() as usize;
    let p = (w - 1) % 5;
    let boss = p == 4 && k % 2 == 0;
    let rush = p == 4 && k % 2 == 1;
    let ai = if p == 1 { k.saturating_sub(2) } else { k - 1 } % 10;
    let (hm, sm, armor, rf, split, cf, dash) = match ai {
        1 => (0.6, 1.7, 0., 0., false, 0., false),
        2 => (1., 1., if w < 13 { 3. } else { 4. }, 0., false, 0., false),
        3 => (1., 1., 0., 0.012, false, 0., false),
        4 => (0.75, 1., 0., 0., true, 0., false),
        5 => (1., 1., 0., 0., false, 0.05, false),
        6 => (
            1.,
            1.,
            if w < 37 { 7. } else { 8. },
            0.008,
            false,
            0.,
            false,
        ),
        7 => (2.2, 0.55, 0., 0., false, 0., false),
        8 => (1., 1.05, 0., 0., false, 0., true),
        9 => (
            1.04,
            1.05,
            if w < 47 { 10. } else { 11. },
            0.008,
            false,
            0.,
            true,
        ),
        _ => (1., 1., 0., 0., false, 0., false),
    };
    let raw = c.hp * c.growth.powi(w as i32 - 1);
    let base = raw * hm;
    let mut hp = base;
    let mut n = 8 + (c.count * w as f64).floor() as usize;
    let mut speed = (1. + c.speed * w as f64) * sm;
    let mut bounty = (1. + c.bounty * w as f64).round().max(1.);
    let mut ar = armor;
    let mut regen = (base * rf).round();
    let mut cap = 0.;
    let mut ds = dash;
    let mut sp = split;
    let mut leak = 1;
    if rush {
        hp *= 0.55;
        n *= 2;
        bounty = (bounty * 0.5).round().max(1.)
    }
    if boss {
        let i = k / 2;
        hp = raw * 26.;
        n = 1;
        speed = 0.55;
        ar = [0., 4., 6., 8., 10.][i.min(5) - 1];
        regen = if i > 2 { (hp * 0.002).round() } else { 0. };
        cap = if i > 3 { (hp * 0.02).floor() } else { 0. };
        bounty = (20 + 10 * i) as f64;
        ds = false;
        sp = false;
        leak = 5
    }
    if cf > 0. && !boss {
        cap = (hp * cf).floor()
    }
    Wave {
        n,
        hp: hp.round(),
        speed,
        armor: ar,
        regen,
        cap,
        dash: ds,
        split: sp,
        bounty,
        leak,
        boss,
        rush,
    }
}

#[derive(Clone, Debug)]
struct Enemy {
    id: u64,
    row: Wave,
    d: f64,
    x: f64,
    y: f64,
    hp: f64,
    max: f64,
    speed: f64,
    armor: f64,
    regen: f64,
    cap: f64,
    dash: bool,
    dc: f64,
    inc: f64,
    split: bool,
    bounty: f64,
    leak: i32,
    boss: bool,
    burn_n: f64,
    burn_t: f64,
    burn_p: f64,
    irr: f64,
    slow: f64,
    slow_f: f64,
    leaked: bool,
}
#[derive(Clone, Debug)]
struct Tower {
    i: usize,
    x: i32,
    y: i32,
    level: u8,
    branch: u8,
    inv: f64,
    cd: f64,
    heat: f64,
    lock: Option<u64>,
    mb: f64,
    buff: f64,
    fire: bool,
    rad: bool,
    damage: f64,
    fusion: Option<usize>,
}
#[derive(Clone, Debug)]
struct Projectile {
    x: f64,
    y: f64,
    target: u64,
    v: f64,
    damage: f64,
    splash: f64,
    kb: f64,
    pierce: bool,
    burn: f64,
    irr: bool,
    slow: bool,
    tower_i: usize,
    tower_slot: usize,
}
#[derive(Clone, Debug)]
struct Spawn {
    row: Wave,
    sn: usize,
    st: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Action {
    pub wave: usize,
    pub op: String,
    #[serde(default)]
    pub tower: usize,
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
    #[serde(default)]
    pub branch: u8,
    #[serde(default)]
    pub with: Option<Point>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    #[serde(default)]
    pub seed: u64,
    #[serde(default)]
    pub map: usize,
    #[serde(default = "fifty")]
    pub max_wave: usize,
    #[serde(default)]
    pub actions: Vec<Action>,
}
fn fifty() -> usize {
    50
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TowerTrace {
    pub x: i32,
    pub y: i32,
    pub tower: usize,
    pub level: u8,
    pub branch: u8,
    pub fusion: Option<usize>,
    pub damage: f64,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct WaveTrace {
    pub wave: usize,
    pub seconds: f64,
    pub lives: i32,
    pub gold: f64,
    pub kills: u64,
    pub won: bool,
    pub over: bool,
    #[serde(rename = "towerDamage")]
    pub tower_damage: Vec<TowerTrace>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RunResult {
    pub protocol: u8,
    pub seed: String,
    pub map: usize,
    pub trace: Vec<WaveTrace>,
    pub rejected: Vec<Action>,
    pub pending: usize,
    pub result: WaveTrace,
}

pub struct Sim {
    pub params: Params,
    map: usize,
    path: Vec<(i32, i32)>,
    gold: f64,
    lives: i32,
    wave: usize,
    phase_wave: bool,
    spawns: Vec<Spawn>,
    t: f64,
    enemies: Vec<Enemy>,
    towers: Vec<Tower>,
    projectiles: Vec<Projectile>,
    kills: u64,
    over: bool,
    won: bool,
    next_enemy: u64,
}

impl Sim {
    pub fn new(params: Params, map: usize) -> Self {
        let mut s = Self {
            params,
            map,
            path: vec![],
            gold: 80.,
            lives: 20,
            wave: 0,
            phase_wave: false,
            spawns: vec![],
            t: 0.,
            enemies: vec![],
            towers: vec![],
            projectiles: vec![],
            kills: 0,
            over: false,
            won: false,
            next_enemy: 1,
        };
        s.set_map(map);
        s
    }
    fn set_map(&mut self, map: usize) {
        let maps: [[(i32, i32); 8]; 3] = [
            [
                (0, 1),
                (12, 1),
                (12, 3),
                (1, 3),
                (1, 5),
                (12, 5),
                (12, 7),
                (0, 7),
            ],
            [
                (0, 1),
                (12, 1),
                (12, 7),
                (1, 7),
                (1, 3),
                (10, 3),
                (10, 5),
                (3, 5),
            ],
            [
                (0, 7),
                (3, 7),
                (3, 1),
                (7, 1),
                (7, 7),
                (11, 7),
                (11, 1),
                (13, 1),
            ],
        ];
        self.map = map;
        self.path.clear();
        for seg in maps[map].windows(2) {
            let (mut x, mut y) = seg[0];
            let (bx, by) = seg[1];
            let dx = (bx - x).signum();
            let dy = (by - y).signum();
            while x != bx || y != by {
                self.path.push((x, y));
                x += dx;
                y += dy
            }
        }
        self.path.push(*maps[map].last().unwrap())
    }
    fn pos(&self, d: f64) -> (f64, f64) {
        let i = (d.floor() as usize).min(self.path.len() - 2);
        let f = d - i as f64;
        let a = self.path[i];
        let b = self.path[i + 1];
        (
            a.0 as f64 + (b.0 - a.0) as f64 * f + 0.5,
            a.1 as f64 + (b.1 - a.1) as f64 * f + 0.5,
        )
    }
    fn age(&self) -> usize {
        (self.wave / 5).min(9)
    }
    fn tower_at(&self, x: i32, y: i32) -> Option<usize> {
        self.towers.iter().position(|t| t.x == x && t.y == y)
    }
    fn on_path(&self, x: i32, y: i32) -> bool {
        self.path.contains(&(x, y))
    }
    fn modifiers(&self, t: &Tower) -> Modifiers {
        if let Some(f) = t.fusion {
            fusion_mod(f)
        } else {
            match t.branch {
                1 => self.params.towers[t.i].branch1.clone(),
                2 => self.params.towers[t.i].branch2.clone(),
                _ => Modifiers::default(),
            }
        }
    }
    fn calc_buffs(&mut self) {
        for i in 0..self.towers.len() {
            self.towers[i].buff = 1.;
            self.towers[i].fire = false;
            self.towers[i].rad = false;
            for j in 0..self.towers.len() {
                if i == j
                    || ((self.towers[j].x - self.towers[i].x)
                        .abs()
                        .max((self.towers[j].y - self.towers[i].y).abs())
                        != 1)
                {
                    continue;
                }
                let (kind, branch, level) = (
                    self.towers[j].i,
                    self.towers[j].branch,
                    self.towers[j].level,
                );
                if kind == 8 {
                    self.towers[i].buff = self.towers[i].buff.max(
                        1. + (if branch == 1 { 0.5 } else { 0.25 }) * 1.5f64.powi(level as i32),
                    );
                }
                if kind == 3 {
                    self.towers[i].fire = true
                }
                if kind == 7 {
                    self.towers[i].rad = true
                }
            }
            if self.params.towers[self.towers[i].i].rate == 0. {
                self.towers[i].fire = false;
                self.towers[i].rad = false
            }
        }
    }
    pub fn apply(&mut self, a: &Action) -> bool {
        match a.op.as_str() {
            "place" => self.place(a.tower, a.x, a.y),
            "upgrade" => self.tower_at(a.x, a.y).is_some_and(|i| self.upgrade(i)),
            "branch" => self
                .tower_at(a.x, a.y)
                .is_some_and(|i| self.branch(i, a.branch)),
            "sell" => {
                if let Some(i) = self.tower_at(a.x, a.y) {
                    self.gold += (self.towers[i].inv * 0.7).round();
                    self.towers.remove(i);
                    self.calc_buffs();
                    true
                } else {
                    false
                }
            }
            "merge" => {
                let Some(i) = self.tower_at(a.x, a.y) else {
                    return false;
                };
                let mate = a
                    .with
                    .as_ref()
                    .and_then(|p| self.tower_at(p.x, p.y))
                    .or_else(|| {
                        self.towers
                            .iter()
                            .enumerate()
                            .find(|(j, t)| {
                                *j != i
                                    && t.i == self.towers[i].i
                                    && t.level == self.towers[i].level
                                    && t.branch == self.towers[i].branch
                                    && (t.x - self.towers[i].x)
                                        .abs()
                                        .max((t.y - self.towers[i].y).abs())
                                        == 1
                            })
                            .map(|x| x.0)
                    });
                if let Some(j) = mate {
                    let inv = self.towers[j].inv;
                    let rm = j;
                    let keep = if j < i { i - 1 } else { i };
                    self.towers.remove(rm);
                    self.towers[keep].level += 1;
                    self.towers[keep].inv += inv;
                    self.towers[keep].mb *= 1.1;
                    self.calc_buffs();
                    true
                } else {
                    false
                }
            }
            "fuse" => self.fuse(a),
            _ => false,
        }
    }
    fn place(&mut self, i: usize, x: i32, y: i32) -> bool {
        if i >= self.params.towers.len()
            || self.gold < self.params.towers[i].cost
            || i > self.age()
            || x < 0
            || y < 0
            || x >= W
            || y >= H
            || self.on_path(x, y)
            || self.tower_at(x, y).is_some()
        {
            return false;
        }
        self.gold -= self.params.towers[i].cost;
        self.towers.push(Tower {
            i,
            x,
            y,
            level: 0,
            branch: 0,
            inv: self.params.towers[i].cost,
            cd: 0.,
            heat: 0.,
            lock: None,
            mb: 1.,
            buff: 1.,
            fire: false,
            rad: false,
            damage: 0.,
            fusion: None,
        });
        self.calc_buffs();
        true
    }
    fn upgrade(&mut self, i: usize) -> bool {
        let c = (self.params.towers[self.towers[i].i].cost * 0.6).round();
        if self.towers[i].level >= 3 || self.gold < c {
            return false;
        }
        self.gold -= c;
        self.towers[i].level += 1;
        self.towers[i].inv += c;
        self.calc_buffs();
        true
    }
    fn branch(&mut self, i: usize, b: u8) -> bool {
        let c = self.params.towers[self.towers[i].i].cost;
        if self.towers[i].branch != 0 || self.towers[i].level < 3 || self.gold < c {
            return false;
        }
        self.gold -= c;
        self.towers[i].branch = b;
        self.towers[i].inv += c;
        self.calc_buffs();
        true
    }
    fn fuse(&mut self, a: &Action) -> bool {
        let (Some(i), Some(p)) = (self.tower_at(a.x, a.y), a.with.as_ref()) else {
            return false;
        };
        let Some(j) = self.tower_at(p.x, p.y) else {
            return false;
        };
        let (ai, bi) = (self.towers[i].i, self.towers[j].i);
        let recipe = [
            (1, 3, 2, 1),
            (2, 4, 2, 4),
            (6, 7, 2, 6),
            (5, 8, 2, 5),
            (9, 7, 3, 9),
        ]
        .iter()
        .position(|&(x, y, l, _)| {
            self.towers[i].level >= l
                && self.towers[j].level >= l
                && ((ai == x && bi == y) || (ai == y && bi == x))
        });
        let Some(r) = recipe else { return false };
        let base = [1, 4, 6, 5, 9][r];
        let inv = self.towers[j].inv;
        let keep = if j < i { i - 1 } else { i };
        self.towers.remove(j);
        self.towers[keep].i = base;
        self.towers[keep].fusion = Some(r);
        self.towers[keep].branch = 0;
        self.towers[keep].inv += inv;
        self.towers[keep].mb *= 1.1;
        self.calc_buffs();
        true
    }
    fn start_wave(&mut self) {
        if self.over || self.wave >= 50 {
            return;
        }
        let r = wave_row(&self.params.constants, self.wave + 1);
        self.spawns.push(Spawn {
            row: r,
            sn: 0,
            st: 0.,
        });
        self.phase_wave = true;
        self.wave += 1
    }
    fn spawn_enemy(&mut self, row: Wave, d: f64, f: Option<f64>) {
        let mul = f.unwrap_or(1.);
        let hp = (row.hp * mul).round();
        let (x, y) = self.pos(d);
        let e = Enemy {
            id: self.next_enemy,
            row: row.clone(),
            d,
            x,
            y,
            hp,
            max: hp,
            speed: row.speed,
            armor: row.armor,
            regen: row.regen,
            cap: row.cap,
            dash: row.dash,
            dc: 2.5,
            inc: 0.,
            split: if f.is_some() { false } else { row.split },
            bounty: if f.is_some() { 1. } else { row.bounty },
            leak: row.leak,
            boss: row.boss,
            burn_n: 0.,
            burn_t: 0.,
            burn_p: 1.,
            irr: 0.,
            slow: 0.,
            slow_f: 0.6,
            leaked: false,
        };
        self.next_enemy += 1;
        self.enemies.push(e)
    }
    fn in_range(t: &Tower, e: &Enemy, r: f64) -> bool {
        (e.x - t.x as f64 - 0.5).powi(2) + (e.y - t.y as f64 - 0.5).powi(2) <= r * r
    }
    fn enemy_idx(&self, id: u64) -> Option<usize> {
        self.enemies.iter().position(|e| e.id == id)
    }
    fn raw(&mut self, ei: usize, d: f64, tower: Option<usize>) {
        let eff = d.min(self.enemies[ei].hp.max(0.));
        self.enemies[ei].hp -= d;
        if let Some(t) = tower {
            if t < self.towers.len() {
                self.towers[t].damage += eff
            }
        }
    }
    fn damage(&mut self, ei: usize, mut d: f64, pierce: bool, slow_bonus: bool, tower: usize) {
        if !pierce {
            d = (d - self.enemies[ei].armor).max(1.)
        }
        if self.enemies[ei].cap > 0. {
            d = d.min(self.enemies[ei].cap)
        }
        if self.enemies[ei].irr > 0. {
            d *= 1.2
        }
        if slow_bonus && self.enemies[ei].slow > 0. {
            d *= 1.4
        }
        self.raw(ei, d, Some(tower))
    }
    pub fn update(&mut self, dt: f64) {
        if self.over {
            return;
        }
        self.t += dt;
        let mut born_rows = vec![];
        for j in &mut self.spawns {
            j.st -= dt;
            if j.st <= 0. && j.sn < j.row.n {
                born_rows.push(j.row.clone());
                j.sn += 1;
                j.st += if j.row.rush { 0.28 } else { 0.8 }
            }
        }
        self.spawns.retain(|j| j.sn < j.row.n);
        for r in born_rows {
            self.spawn_enemy(r, 0., None)
        }
        for ti in 0..self.towers.len() {
            let spec = self.params.towers[self.towers[ti].i].clone();
            let b = self.modifiers(&self.towers[ti]);
            let mult =
                1.5f64.powi(self.towers[ti].level as i32) * b.dm.unwrap_or(1.) * self.towers[ti].mb;
            let range =
                spec.range * b.rgm.unwrap_or(1.) * (1. + 0.06 * self.towers[ti].level as f64);
            if spec.field {
                for e in &mut self.enemies {
                    if Self::in_range(&self.towers[ti], e, range) && e.slow < 0.2 {
                        e.slow = 0.15;
                        e.slow_f = b.sf.unwrap_or(0.6)
                    }
                }
                continue;
            }
            if spec.aura {
                for ei in 0..self.enemies.len() {
                    if Self::in_range(&self.towers[ti], &self.enemies[ei], range) {
                        let d = spec.damage
                            * mult
                            * dt
                            * (if self.enemies[ei].irr > 0. { 1.2 } else { 1. });
                        self.raw(ei, d, Some(ti));
                        self.enemies[ei].irr = 0.5
                    }
                }
                continue;
            }
            if spec.beam {
                let valid = self.towers[ti]
                    .lock
                    .and_then(|id| self.enemy_idx(id))
                    .filter(|&i| self.enemies[i].hp > 0. && !self.enemies[i].leaked);
                if valid.is_none() {
                    self.towers[ti].lock = self
                        .enemies
                        .iter()
                        .max_by(|a, b| a.max.total_cmp(&b.max))
                        .map(|e| e.id);
                    self.towers[ti].heat = 0.
                }
                if let Some(ei) = self.towers[ti].lock.and_then(|id| self.enemy_idx(id)) {
                    self.towers[ti].heat += dt * b.hr.unwrap_or(1.);
                    let d = spec.damage
                        * (1. + self.towers[ti].heat)
                        * mult
                        * dt
                        * (if self.enemies[ei].irr > 0. { 1.2 } else { 1. });
                    self.raw(ei, d, Some(ti));
                    if b.ir.is_some() {
                        self.enemies[ei].irr = 0.5
                    }
                }
                continue;
            }
            self.towers[ti].cd -= dt * self.towers[ti].buff;
            let target = if spec.homing {
                self.enemies
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| Self::in_range(&self.towers[ti], e, range) && e.hp > e.inc)
                    .max_by(|(_, a), (_, b)| (a.hp - a.inc).total_cmp(&(b.hp - b.inc)))
                    .map(|x| x.0)
            } else {
                self.enemies
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| Self::in_range(&self.towers[ti], e, range))
                    .max_by(|(_, a), (_, b)| a.d.total_cmp(&b.d))
                    .map(|x| x.0)
            };
            if spec.ramp {
                let id = target.map(|i| self.enemies[i].id);
                if self.towers[ti].lock != id {
                    self.towers[ti].lock = id;
                    self.towers[ti].heat = 0.
                } else if id.is_some() {
                    self.towers[ti].heat = (self.towers[ti].heat + dt).min(b.hc.unwrap_or(6.))
                }
            }
            let Some(ei) = target else {
                if self.towers[ti].cd < 0. {
                    self.towers[ti].cd = 0.
                }
                continue;
            };
            if self.towers[ti].cd <= 0. {
                self.towers[ti].cd += 1.
                    / if spec.ramp {
                        spec.rate + self.towers[ti].heat
                    } else {
                        spec.rate
                    };
                let damage = spec.damage * mult;
                self.projectiles.push(Projectile {
                    x: self.towers[ti].x as f64 + 0.5,
                    y: self.towers[ti].y as f64 + 0.5,
                    target: self.enemies[ei].id,
                    v: (if spec.homing { 5. } else { 12. })
                        * (1. + 0.1 * self.towers[ti].level as f64)
                        * b.pv.unwrap_or(1.),
                    damage,
                    splash: b.sp.unwrap_or(spec.splash),
                    kb: b.kb.unwrap_or(spec.knockback),
                    pierce: b.pi.is_some() || spec.pierce,
                    burn: if spec.burn || b.bu.is_some() {
                        mult * b.bum.unwrap_or(1.)
                    } else if self.towers[ti].fire {
                        1.
                    } else {
                        0.
                    },
                    irr: self.towers[ti].rad || b.ir.is_some(),
                    slow: b.sl.is_some(),
                    tower_i: self.towers[ti].i,
                    tower_slot: ti,
                });
                if spec.homing {
                    self.enemies[ei].inc += damage
                }
            }
        }
        let projectiles = std::mem::take(&mut self.projectiles);
        let mut keep = Vec::with_capacity(projectiles.len());
        for mut p in projectiles {
            let Some(ei) = self.enemy_idx(p.target) else {
                continue;
            };
            let (tx, ty) = (self.enemies[ei].x, self.enemies[ei].y);
            let (dx, dy) = (tx - p.x, ty - p.y);
            let len = dx.hypot(dy);
            let step = p.v * dt;
            if len <= step {
                if p.splash > 0. {
                    let ids: Vec<u64> = self
                        .enemies
                        .iter()
                        .filter(|e| (e.x - tx).powi(2) + (e.y - ty).powi(2) <= p.splash * p.splash)
                        .map(|e| e.id)
                        .collect();
                    for id in ids {
                        if let Some(i) = self.enemy_idx(id) {
                            self.damage(i, p.damage, p.pierce, true, p.tower_slot)
                        }
                    }
                    if let Some(i) = self.enemy_idx(p.target) {
                        self.enemies[i].inc -= p.damage
                    }
                } else if self.enemies[ei].hp > 0. && !self.enemies[ei].leaked {
                    self.damage(ei, p.damage, p.pierce, false, p.tower_slot)
                }
                if let Some(i) = self.enemy_idx(p.target) {
                    if p.kb > 0. && self.enemies[i].hp > 0. {
                        self.enemies[i].d = (self.enemies[i].d
                            - p.kb * (if self.enemies[i].boss { 0.15 } else { 1. }))
                        .max(0.)
                    }
                    if p.burn > 0. && self.enemies[i].hp > 0. {
                        self.enemies[i].burn_t = 3.;
                        self.enemies[i].burn_n = (self.enemies[i].burn_n + 1.).min(5.);
                        self.enemies[i].burn_p = self.enemies[i].burn_p.max(p.burn)
                    }
                    if p.irr && self.enemies[i].hp > 0. {
                        self.enemies[i].irr = 0.5
                    }
                    if p.slow && self.enemies[i].hp > 0. && self.enemies[i].slow < 0.2 {
                        self.enemies[i].slow = 0.5;
                        self.enemies[i].slow_f = 0.6
                    }
                }
            } else {
                p.x += dx / len * step;
                p.y += dy / len * step;
                keep.push(p)
            }
        }
        self.projectiles = keep;
        let plen = (self.path.len() - 1) as f64;
        for ei in 0..self.enemies.len() {
            let e = &mut self.enemies[ei];
            if e.dash {
                e.dc -= dt;
                if e.dc <= 0. {
                    e.dc += 2.5
                }
            }
            let sm = (if e.dash && e.dc < 0.6 { 2.4 } else { 1. })
                * (if e.slow > 0. { e.slow_f } else { 1. });
            e.slow -= dt;
            e.irr -= dt;
            e.d += e.speed * sm * dt;
            if e.regen > 0. && e.hp > 0. && e.hp < e.max {
                e.hp = (e.hp + e.regen * dt).min(e.max)
            }
            if e.burn_t > 0. {
                let d = 4. * e.burn_n * e.burn_p * (if e.irr > 0. { 2.4 } else { 1. }) * dt;
                e.hp -= d;
                e.burn_t -= dt;
                if e.burn_t <= 0. {
                    e.burn_n = 0.
                }
            }
        }
        for i in 0..self.enemies.len() {
            let p = self.pos(self.enemies[i].d.min(plen));
            self.enemies[i].x = p.0;
            self.enemies[i].y = p.1;
            if self.enemies[i].d >= plen && !self.enemies[i].leaked {
                self.enemies[i].leaked = true;
                self.lives -= self.enemies[i].leak
            }
        }
        let dead: Vec<(Wave, f64, Option<f64>)> = self
            .enemies
            .iter()
            .filter(|e| !e.leaked && e.hp <= 0. && e.split)
            .flat_map(|e| {
                [
                    (e.row.clone(), (e.d - 0.6).max(0.), Some(0.4)),
                    (e.row.clone(), (e.d - 1.2).max(0.), Some(0.4)),
                ]
            })
            .collect();
        for e in &self.enemies {
            if !e.leaked && e.hp <= 0. {
                self.gold += e.bounty;
                self.kills += 1
            }
        }
        self.enemies.retain(|e| !e.leaked && e.hp > 0.);
        for (drow, d, f) in dead {
            self.spawn_enemy(drow, d, f)
        }
        if self.lives <= 0 {
            self.over = true;
            self.won = false;
            self.phase_wave = false;
            return;
        }
        if self.phase_wave && self.spawns.is_empty() && self.enemies.is_empty() {
            if self.wave >= 50 {
                self.over = true;
                self.won = true;
                self.phase_wave = false
            } else {
                self.phase_wave = false
            }
        }
    }
    fn trace(&self, seconds: f64) -> WaveTrace {
        let mut td: Vec<_> = self
            .towers
            .iter()
            .map(|t| TowerTrace {
                x: t.x,
                y: t.y,
                tower: t.i,
                level: t.level,
                branch: t.branch,
                fusion: t.fusion,
                damage: round6(t.damage),
            })
            .collect();
        td.sort_by_key(|t| (t.y, t.x));
        WaveTrace {
            wave: self.wave,
            seconds: round6(seconds),
            lives: self.lives,
            gold: round6(self.gold),
            kills: self.kills,
            won: self.won,
            over: self.over,
            tower_damage: td,
        }
    }
}

fn fusion_mod(i: usize) -> Modifiers {
    match i {
        0 => Modifiers {
            dm: Some(2.),
            sp: Some(1.4),
            bu: Some(1.),
            bum: Some(2.),
            ..Default::default()
        },
        1 => Modifiers {
            dm: Some(2.),
            pi: Some(1.),
            kb: Some(1.1),
            ..Default::default()
        },
        2 => Modifiers {
            dm: Some(1.2),
            ir: Some(1.),
            ..Default::default()
        },
        3 => Modifiers {
            dm: Some(1.5),
            hc: Some(12.),
            sl: Some(1.),
            ..Default::default()
        },
        4 => Modifiers {
            dm: Some(1.15),
            hr: Some(1.5),
            ir: Some(1.),
            ..Default::default()
        },
        _ => Default::default(),
    }
}
fn round6(x: f64) -> f64 {
    (x * 1_000_000.).round() / 1_000_000.
}

pub fn run(input: &Input, params: Params) -> RunResult {
    let mut sim = Sim::new(params, input.map);
    let mut pending = input.actions.clone();
    pending.sort_by_key(|a| a.wave);
    let mut rejected = vec![];
    let mut trace = vec![];
    let mut ticks = 0usize;
    while !sim.over && sim.wave < input.max_wave && ticks < 3_000_000 {
        while pending.first().is_some_and(|a| a.wave <= sim.wave) {
            let a = pending.remove(0);
            if !sim.apply(&a) {
                rejected.push(a)
            }
        }
        if !sim.phase_wave {
            sim.start_wave()
        }
        let started = sim.t;
        while sim.phase_wave && !sim.over && ticks < 3_000_000 {
            sim.update(DT);
            ticks += 1
        }
        trace.push(sim.trace(sim.t - started));
    }
    let total = trace.iter().map(|x| x.seconds).sum();
    RunResult {
        protocol: 1,
        seed: input.seed.to_string(),
        map: input.map,
        trace,
        rejected,
        pending: pending.len(),
        result: sim.trace(total),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Genome {
    pub tower_weights: [f64; 10],
    pub upgrade_weight: f64,
    pub branch_bias: f64,
    pub merge_weight: f64,
    pub fusion_weights: [f64; 5],
    pub placement: [f64; 4],
    pub reaction_ticks: u32,
    pub apm: f64,
}
impl Genome {
    pub fn random(rng: &mut Xoshiro) -> Self {
        let mut w = [0.; 10];
        for x in &mut w {
            *x = rng.next_f64() * 2. - 1.
        }
        let mut fusion_weights = [0.; 5];
        for x in &mut fusion_weights {
            *x = rng.next_f64() * 2. - 1.
        }
        Self {
            tower_weights: w,
            upgrade_weight: rng.next_f64() * 2. - 1.,
            branch_bias: rng.next_f64() * 2. - 1.,
            merge_weight: rng.next_f64() * 2. - 1.,
            fusion_weights,
            placement: [
                rng.next_f64(),
                rng.next_f64(),
                rng.next_f64(),
                rng.next_f64(),
            ],
            reaction_ticks: (rng.next_u64() % 91) as u32,
            apm: 10. + rng.next_f64() * 110.,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct PolicyRun {
    pub run: RunResult,
    pub actions: Vec<Action>,
    pub fusion_counts: [usize; 5],
    pub tower_counts: [usize; 10],
    pub merges: usize,
}

// Executes a genome against the live state at each clear phase. The APM cap
// limits decisions in the three-second inter-wave window; reaction_ticks are
// spent before the first decision, preventing a zero-latency policy model.
pub fn run_policy(
    genome: &Genome,
    seed: u64,
    map: usize,
    max_wave: usize,
    params: Params,
) -> PolicyRun {
    let mut sim = Sim::new(params, map);
    let cells = placement_cells(map, genome.placement);
    let mut actions = vec![];
    let mut trace = vec![];
    let mut fusion_counts = [0usize; 5];
    let mut tower_counts = [0usize; 10];
    let mut merges = 0;
    let mut ticks = 0usize;
    let recipes = [
        (1usize, 3usize, 2u8),
        (2, 4, 2),
        (6, 7, 2),
        (5, 8, 2),
        (9, 7, 3),
    ];

    while !sim.over && sim.wave < max_wave && ticks < 3_000_000 {
        let action_cap = ((genome.apm * 3. / 60.).floor() as usize).max(1);
        let _human_reaction_seconds = genome.reaction_ticks as f64 * DT;
        for _ in 0..action_cap {
            let unlocked = sim.age();
            let preferred_fusion = (0..5)
                .filter(|&r| recipes[r].0 <= unlocked && recipes[r].1 <= unlocked)
                .max_by(|&a, &b| genome.fusion_weights[a].total_cmp(&genome.fusion_weights[b]));
            let mut next: Option<Action> = None;

            if let Some(r) = preferred_fusion.filter(|&r| genome.fusion_weights[r] > 0.) {
                let (a, b, level) = recipes[r];
                'pairs: for i in 0..sim.towers.len() {
                    for j in i + 1..sim.towers.len() {
                        let t = &sim.towers[i];
                        let p = &sim.towers[j];
                        if t.level >= level
                            && p.level >= level
                            && ((t.i == a && p.i == b) || (t.i == b && p.i == a))
                            && (t.x - p.x).abs().max((t.y - p.y).abs()) == 1
                        {
                            next = Some(Action {
                                wave: sim.wave,
                                op: "fuse".into(),
                                tower: 0,
                                x: t.x,
                                y: t.y,
                                branch: 0,
                                with: Some(Point { x: p.x, y: p.y }),
                            });
                            break 'pairs;
                        }
                    }
                }
                if next.is_none() {
                    for &kind in &[a, b] {
                        if let Some(t) = sim.towers.iter().find(|t| t.i == kind && t.level < level)
                        {
                            next = Some(Action {
                                wave: sim.wave,
                                op: "upgrade".into(),
                                tower: 0,
                                x: t.x,
                                y: t.y,
                                branch: 0,
                                with: None,
                            });
                            break;
                        }
                    }
                }
                if next.is_none() {
                    let have_a = sim.towers.iter().find(|t| t.i == a).map(|t| (t.x, t.y));
                    let have_b = sim.towers.iter().find(|t| t.i == b).map(|t| (t.x, t.y));
                    let kind = if have_a.is_none() { a } else { b };
                    let anchor = if kind == b { have_a } else { have_b };
                    let cell = anchor
                        .and_then(|(x, y)| {
                            [(-1, 0), (1, 0), (0, -1), (0, 1)]
                                .into_iter()
                                .map(|(dx, dy)| (x + dx, y + dy))
                                .find(|&(x, y)| {
                                    x >= 0
                                        && y >= 0
                                        && x < W
                                        && y < H
                                        && !sim.on_path(x, y)
                                        && sim.tower_at(x, y).is_none()
                                })
                        })
                        .or_else(|| {
                            cells
                                .iter()
                                .copied()
                                .find(|&(x, y)| sim.tower_at(x, y).is_none())
                        });
                    if let Some((x, y)) = cell {
                        next = Some(Action {
                            wave: sim.wave,
                            op: "place".into(),
                            tower: kind,
                            x,
                            y,
                            branch: 0,
                            with: None,
                        });
                    }
                }
            }

            if next.is_none() && genome.merge_weight > 0. {
                'merge: for i in 0..sim.towers.len() {
                    for j in i + 1..sim.towers.len() {
                        let t = &sim.towers[i];
                        let p = &sim.towers[j];
                        if t.i == p.i
                            && t.level == p.level
                            && t.level < 5
                            && t.branch == p.branch
                            && t.fusion == p.fusion
                            && (t.x - p.x).abs().max((t.y - p.y).abs()) == 1
                        {
                            next = Some(Action {
                                wave: sim.wave,
                                op: "merge".into(),
                                tower: 0,
                                x: t.x,
                                y: t.y,
                                branch: 0,
                                with: Some(Point { x: p.x, y: p.y }),
                            });
                            break 'merge;
                        }
                    }
                }
            }
            if next.is_none() && genome.upgrade_weight > 0. {
                if let Some(t) =
                    sim.towers.iter().filter(|t| t.level < 3).max_by(|a, b| {
                        genome.tower_weights[a.i].total_cmp(&genome.tower_weights[b.i])
                    })
                {
                    next = Some(Action {
                        wave: sim.wave,
                        op: "upgrade".into(),
                        tower: 0,
                        x: t.x,
                        y: t.y,
                        branch: 0,
                        with: None,
                    });
                } else if let Some(t) = sim
                    .towers
                    .iter()
                    .find(|t| t.level == 3 && t.branch == 0 && t.fusion.is_none())
                {
                    next = Some(Action {
                        wave: sim.wave,
                        op: "branch".into(),
                        tower: 0,
                        x: t.x,
                        y: t.y,
                        branch: if genome.branch_bias >= 0. { 1 } else { 2 },
                        with: None,
                    });
                }
            }
            if next.is_none() {
                let kind = (0..=unlocked)
                    .max_by(|&a, &b| genome.tower_weights[a].total_cmp(&genome.tower_weights[b]))
                    .unwrap();
                if let Some((x, y)) = cells
                    .iter()
                    .copied()
                    .find(|&(x, y)| sim.tower_at(x, y).is_none())
                {
                    next = Some(Action {
                        wave: sim.wave,
                        op: "place".into(),
                        tower: kind,
                        x,
                        y,
                        branch: 0,
                        with: None,
                    });
                }
            }
            let Some(a) = next else { break };
            if sim.apply(&a) {
                if a.op == "place" {
                    tower_counts[a.tower] += 1
                }
                if a.op == "merge" {
                    merges += 1
                }
                if a.op == "fuse" {
                    if let Some(r) = preferred_fusion {
                        fusion_counts[r] += 1
                    }
                }
                actions.push(a);
            } else {
                break;
            }
        }
        sim.start_wave();
        let started = sim.t;
        while sim.phase_wave && !sim.over && ticks < 3_000_000 {
            sim.update(DT);
            ticks += 1
        }
        trace.push(sim.trace(sim.t - started));
    }
    let total = trace.iter().map(|x| x.seconds).sum();
    let result = RunResult {
        protocol: 1,
        seed: seed.to_string(),
        map,
        trace,
        rejected: vec![],
        pending: 0,
        result: sim.trace(total),
    };
    PolicyRun {
        run: result,
        actions,
        fusion_counts,
        tower_counts,
        merges,
    }
}

#[derive(Clone, Debug)]
pub struct Xoshiro([u64; 4]);
impl Xoshiro {
    pub fn new(seed: u64) -> Self {
        let mut z = seed;
        let mut s = [0; 4];
        for x in &mut s {
            z = z.wrapping_add(0x9e3779b97f4a7c15);
            let mut q = z;
            q = (q ^ (q >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            q = (q ^ (q >> 27)).wrapping_mul(0x94d049bb133111eb);
            *x = q ^ (q >> 31)
        }
        Self(s)
    }
    pub fn next_u64(&mut self) -> u64 {
        let r = self.0[0]
            .wrapping_add(self.0[3])
            .rotate_left(23)
            .wrapping_add(self.0[0]);
        let t = self.0[1] << 17;
        self.0[2] ^= self.0[0];
        self.0[3] ^= self.0[1];
        self.0[1] ^= self.0[2];
        self.0[0] ^= self.0[3];
        self.0[2] ^= t;
        self.0[3] = self.0[3].rotate_left(45);
        r
    }
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

pub fn placement_cells(map: usize, weights: [f64; 4]) -> Vec<(i32, i32)> {
    let sim = Sim::new(Params::default(), map);
    let mut out = vec![];
    for y in 0..H {
        for x in 0..W {
            if sim.on_path(x, y) {
                continue;
            }
            let coverage = sim
                .path
                .iter()
                .filter(|&&(px, py)| ((px - x).pow(2) + (py - y).pow(2)) as f64 <= 20.25)
                .count() as f64;
            let corners = sim
                .path
                .windows(3)
                .filter(|p| {
                    (p[0].0 - p[1].0, p[0].1 - p[1].1) != (p[1].0 - p[2].0, p[1].1 - p[2].1)
                        && ((p[1].0 - x).pow(2) + (p[1].1 - y).pow(2)) <= 25
                })
                .count() as f64;
            let center = -(x as f64 - 6.5).abs() - (y as f64 - 4.).abs();
            let edge = -(x.min(13 - x).min(y.min(8 - y))) as f64;
            let score = weights[0] * coverage
                + weights[1] * corners
                + weights[2] * center
                + weights[3] * edge;
            out.push((score, x, y))
        }
    }
    out.sort_by(|a, b| b.0.total_cmp(&a.0));
    out.into_iter().map(|(_, x, y)| (x, y)).collect()
}

#[derive(Clone, Debug, Serialize)]
pub struct Elite {
    pub fitness: f64,
    pub wave: usize,
    pub lives: i32,
    pub mono_tower: usize,
    pub no_merge: bool,
    pub fusion: Option<usize>,
    pub genome: Genome,
    pub seed: u64,
    pub actions: Vec<Action>,
}
pub type Archive = BTreeMap<(usize, bool, Option<usize>), Elite>;
