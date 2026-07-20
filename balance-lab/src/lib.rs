use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const DT: f64 = 1.0 / 30.0;
pub const EXECUTION_TICK_CAP: usize = 3_000_000;
const W: i32 = 14;
const H: i32 = 9;
const LEVEL_MULTIPLIERS: [f64; 6] = [1., 1.5, 2.25, 3.375, 5.0625, 7.59375];

fn level_multiplier(level: u8) -> f64 {
    LEVEL_MULTIPLIERS
        .get(level as usize)
        .copied()
        .unwrap_or_else(|| 1.5f64.powi(level as i32))
}

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
    #[serde(default)]
    pub rm: Option<f64>,
    #[serde(default)]
    pub bf: Option<f64>,
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
pub struct FusionSpec {
    pub a: usize,
    pub b: usize,
    pub level: u8,
    pub base: usize,
    pub modifiers: Modifiers,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Rules {
    pub upgrade_cost: f64,
    pub sell_refund: f64,
    pub doctrine_upgrade_cost: f64,
    pub doctrine_sell_refund: f64,
    pub doctrine_bounty: f64,
    pub doctrine_range: f64,
    pub doctrine_cooldown: f64,
    pub doctrine_knockback: f64,
    pub doctrine_boss_knockback: f64,
    pub doctrine_lives: i32,
    pub doctrine_projectile_speed: f64,
    pub doctrine_splash: f64,
    pub doctrine_irradiate: f64,
    pub doctrine_slow: f64,
    pub burn_cap: f64,
    pub doctrine_burn_cap: f64,
    pub power_cooldowns: [f64; 3],
    pub power_ages: [usize; 3],
    pub meteor_damage: f64,
    pub meteor_radius: f64,
    pub stasis_duration: f64,
    pub stasis_factor: f64,
    pub overdrive_duration: f64,
    pub overdrive_multiplier: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Params {
    pub constants: Constants,
    pub towers: Vec<TowerSpec>,
    pub fusions: Vec<FusionSpec>,
    pub rules: Rules,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ConstantsPatch {
    pub hp: Option<f64>,
    #[serde(rename = "g")]
    pub growth: Option<f64>,
    #[serde(rename = "cnt")]
    pub count: Option<f64>,
    #[serde(rename = "spd")]
    pub speed: Option<f64>,
    #[serde(rename = "bty")]
    pub bounty: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TowerPatch {
    #[serde(rename = "c")]
    pub cost: Option<f64>,
    #[serde(rename = "d")]
    pub damage: Option<f64>,
    #[serde(rename = "r")]
    pub rate: Option<f64>,
    #[serde(rename = "rg")]
    pub range: Option<f64>,
    #[serde(rename = "sp")]
    pub splash: Option<f64>,
    #[serde(rename = "pi")]
    pub pierce: Option<bool>,
    #[serde(rename = "bu")]
    pub burn: Option<bool>,
    #[serde(rename = "kb")]
    pub knockback: Option<f64>,
    pub ramp: Option<bool>,
    #[serde(rename = "home")]
    pub homing: Option<bool>,
    pub aura: Option<bool>,
    pub field: Option<bool>,
    pub beam: Option<bool>,
    #[serde(rename = "b1")]
    pub branch1: Option<Modifiers>,
    #[serde(rename = "b2")]
    pub branch2: Option<Modifiers>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FusionPatch {
    pub a: Option<usize>,
    pub b: Option<usize>,
    #[serde(rename = "lv")]
    pub level: Option<u8>,
    pub base: Option<usize>,
    #[serde(rename = "f")]
    pub modifiers: Option<Modifiers>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ParamsPatch {
    pub constants: Option<ConstantsPatch>,
    pub towers: Option<Vec<TowerPatch>>,
    pub fusions: Option<Vec<FusionPatch>>,
    pub rules: Option<Rules>,
}

impl ParamsPatch {
    pub fn apply(&self) -> Params {
        let mut params = Params::default();
        if let Some(patch) = &self.constants {
            if let Some(value) = patch.hp {
                params.constants.hp = value
            }
            if let Some(value) = patch.growth {
                params.constants.growth = value
            }
            if let Some(value) = patch.count {
                params.constants.count = value
            }
            if let Some(value) = patch.speed {
                params.constants.speed = value
            }
            if let Some(value) = patch.bounty {
                params.constants.bounty = value
            }
        }
        if let Some(towers) = &self.towers {
            for (tower, patch) in params.towers.iter_mut().zip(towers) {
                if let Some(value) = patch.cost {
                    tower.cost = value
                }
                if let Some(value) = patch.damage {
                    tower.damage = value
                }
                if let Some(value) = patch.rate {
                    tower.rate = value
                }
                if let Some(value) = patch.range {
                    tower.range = value
                }
                if let Some(value) = patch.splash {
                    tower.splash = value
                }
                if let Some(value) = patch.pierce {
                    tower.pierce = value
                }
                if let Some(value) = patch.burn {
                    tower.burn = value
                }
                if let Some(value) = patch.knockback {
                    tower.knockback = value
                }
                if let Some(value) = patch.ramp {
                    tower.ramp = value
                }
                if let Some(value) = patch.homing {
                    tower.homing = value
                }
                if let Some(value) = patch.aura {
                    tower.aura = value
                }
                if let Some(value) = patch.field {
                    tower.field = value
                }
                if let Some(value) = patch.beam {
                    tower.beam = value
                }
                if let Some(value) = &patch.branch1 {
                    merge_modifiers(&mut tower.branch1, value)
                }
                if let Some(value) = &patch.branch2 {
                    merge_modifiers(&mut tower.branch2, value)
                }
            }
        }
        if let Some(fusions) = &self.fusions {
            for (fusion, patch) in params.fusions.iter_mut().zip(fusions) {
                if let Some(value) = patch.a {
                    fusion.a = value
                }
                if let Some(value) = patch.b {
                    fusion.b = value
                }
                if let Some(value) = patch.level {
                    fusion.level = value
                }
                if let Some(value) = patch.base {
                    fusion.base = value
                }
                if let Some(value) = &patch.modifiers {
                    merge_modifiers(&mut fusion.modifiers, value)
                }
            }
        }
        if let Some(rules) = &self.rules {
            params.rules = rules.clone();
        }
        params
    }
}

fn merge_modifiers(target: &mut Modifiers, patch: &Modifiers) {
    macro_rules! merge {
        ($($field:ident),+ $(,)?) => {$(
            if patch.$field.is_some() { target.$field = patch.$field; }
        )+};
    }
    merge!(dm, sp, pv, rgm, kb, pi, bu, bum, hc, sf, ir, sl, hr, rm, bf);
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
            rm: None,
            bf: None,
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
                    branch2: {
                        let mut branch = m(
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
                        );
                        branch.rm = Some(2.);
                        branch
                    },
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
            fusions: vec![
                FusionSpec {
                    a: 1,
                    b: 3,
                    level: 2,
                    base: 1,
                    modifiers: Modifiers {
                        dm: Some(2.),
                        sp: Some(1.4),
                        bu: Some(1.),
                        bum: Some(2.),
                        ..Default::default()
                    },
                },
                FusionSpec {
                    a: 2,
                    b: 4,
                    level: 2,
                    base: 4,
                    modifiers: Modifiers {
                        dm: Some(2.),
                        pi: Some(1.),
                        kb: Some(1.1),
                        ..Default::default()
                    },
                },
                FusionSpec {
                    a: 6,
                    b: 7,
                    level: 2,
                    base: 6,
                    modifiers: Modifiers {
                        dm: Some(1.2),
                        ir: Some(1.),
                        ..Default::default()
                    },
                },
                FusionSpec {
                    a: 5,
                    b: 8,
                    level: 2,
                    base: 5,
                    modifiers: Modifiers {
                        dm: Some(1.5),
                        hc: Some(12.),
                        sl: Some(1.),
                        ..Default::default()
                    },
                },
                FusionSpec {
                    a: 9,
                    b: 7,
                    level: 3,
                    base: 9,
                    modifiers: Modifiers {
                        dm: Some(1.15),
                        hr: Some(1.5),
                        ir: Some(1.),
                        ..Default::default()
                    },
                },
                FusionSpec {
                    a: usize::MAX,
                    b: usize::MAX,
                    level: 5,
                    base: usize::MAX,
                    modifiers: Modifiers {
                        dm: Some(1.6),
                        rgm: Some(1.15),
                        ..Default::default()
                    },
                },
            ],
            rules: Rules {
                upgrade_cost: 0.6,
                sell_refund: 0.7,
                doctrine_upgrade_cost: 0.5,
                doctrine_sell_refund: 0.85,
                doctrine_bounty: 1.,
                doctrine_range: 1.08,
                doctrine_cooldown: 0.7,
                doctrine_knockback: 1.5,
                doctrine_boss_knockback: 0.3,
                doctrine_lives: 3,
                doctrine_projectile_speed: 1.3,
                doctrine_splash: 1.2,
                doctrine_irradiate: 1.,
                doctrine_slow: 0.5,
                burn_cap: 5.,
                doctrine_burn_cap: 7.,
                power_cooldowns: [45., 30., 40.],
                power_ages: [1, 4, 7],
                meteor_damage: 3.,
                meteor_radius: 1.5,
                stasis_duration: 2.5,
                stasis_factor: 0.15,
                overdrive_duration: 8.,
                overdrive_multiplier: 1.5,
            },
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
    let boss = p == 4 && k.is_multiple_of(2);
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
#[derive(Clone, Default)]
struct EnemySoa {
    id: Vec<u64>,
    row: Vec<Wave>,
    d: Vec<f64>,
    x: Vec<f64>,
    y: Vec<f64>,
    hp: Vec<f64>,
    max: Vec<f64>,
    speed: Vec<f64>,
    armor: Vec<f64>,
    regen: Vec<f64>,
    cap: Vec<f64>,
    dash: Vec<u8>,
    dc: Vec<f64>,
    inc: Vec<f64>,
    split: Vec<u8>,
    bounty: Vec<f64>,
    leak: Vec<i32>,
    boss: Vec<u8>,
    burn_n: Vec<f64>,
    burn_t: Vec<f64>,
    burn_p: Vec<f64>,
    irr: Vec<f64>,
    slow: Vec<f64>,
    slow_f: Vec<f64>,
    leaked: Vec<u8>,
}
impl EnemySoa {
    fn with_capacity(n: usize) -> Self {
        let mut s = Self::default();
        s.reserve(n);
        s
    }
    fn reserve(&mut self, n: usize) {
        macro_rules! r{($($f:ident),*)=>{$(self.$f.reserve(n);)*}}
        r!(
            id, row, d, x, y, hp, max, speed, armor, regen, cap, dash, dc, inc, split, bounty,
            leak, boss, burn_n, burn_t, burn_p, irr, slow, slow_f, leaked
        );
    }
    fn len(&self) -> usize {
        self.id.len()
    }
    fn is_empty(&self) -> bool {
        self.id.is_empty()
    }
    fn push(&mut self, e: Enemy) {
        macro_rules! p{($($f:ident),*)=>{$(self.$f.push(e.$f);)*}}
        p!(
            id, row, d, x, y, hp, max, speed, armor, regen, cap, dc, inc, bounty, leak, burn_n,
            burn_t, burn_p, irr, slow, slow_f
        );
        self.dash.push(u8::from(e.dash));
        self.split.push(u8::from(e.split));
        self.boss.push(u8::from(e.boss));
        self.leaked.push(u8::from(e.leaked));
    }
    fn retain_alive(&mut self) {
        let mut w = 0;
        for i in 0..self.len() {
            if self.leaked[i] == 0 && self.hp[i] > 0. {
                if i != w {
                    self.id[w] = self.id[i];
                    self.row[w] = self.row[i].clone();
                    macro_rules! c{($($f:ident),*)=>{$(self.$f[w]=self.$f[i];)*}}
                    c!(
                        d, x, y, hp, max, speed, armor, regen, cap, dash, dc, inc, split, bounty,
                        leak, boss, burn_n, burn_t, burn_p, irr, slow, slow_f, leaked
                    );
                }
                w += 1
            }
        }
        macro_rules! t{($($f:ident),*)=>{$(self.$f.truncate(w);)*}}
        t!(
            id, row, d, x, y, hp, max, speed, armor, regen, cap, dash, dc, inc, split, bounty,
            leak, boss, burn_n, burn_t, burn_p, irr, slow, slow_f, leaked
        );
    }
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
    target_x: f64,
    target_y: f64,
    v: f64,
    damage: f64,
    splash: f64,
    kb: f64,
    pierce: bool,
    burn: f64,
    irr: bool,
    slow: bool,
    tower_slot: usize,
    born_at: f64,
}
#[derive(Clone, Default)]
struct ProjectileSoa {
    x: Vec<f64>,
    y: Vec<f64>,
    target: Vec<u64>,
    target_x: Vec<f64>,
    target_y: Vec<f64>,
    v: Vec<f64>,
    damage: Vec<f64>,
    splash: Vec<f64>,
    kb: Vec<f64>,
    pierce: Vec<u8>,
    burn: Vec<f64>,
    irr: Vec<u8>,
    slow: Vec<u8>,
    tower_slot: Vec<usize>,
    born_at: Vec<f64>,
}
impl ProjectileSoa {
    fn with_capacity(n: usize) -> Self {
        let mut s = Self::default();
        s.reserve(n);
        s
    }
    fn reserve(&mut self, n: usize) {
        macro_rules! r{($($f:ident),*)=>{$(self.$f.reserve(n);)*}}
        r!(
            x, y, target, target_x, target_y, v, damage, splash, kb, pierce, burn, irr, slow,
            tower_slot, born_at
        );
    }
    fn len(&self) -> usize {
        self.x.len()
    }
    fn push(&mut self, p: Projectile) {
        macro_rules! q{($($f:ident),*)=>{$(self.$f.push(p.$f);)*}}
        q!(
            x, y, target, target_x, target_y, v, damage, splash, kb, burn, tower_slot, born_at
        );
        self.pierce.push(u8::from(p.pierce));
        self.irr.push(u8::from(p.irr));
        self.slow.push(u8::from(p.slow));
    }
    fn get(&self, i: usize) -> Projectile {
        Projectile {
            x: self.x[i],
            y: self.y[i],
            target: self.target[i],
            target_x: self.target_x[i],
            target_y: self.target_y[i],
            v: self.v[i],
            damage: self.damage[i],
            splash: self.splash[i],
            kb: self.kb[i],
            pierce: self.pierce[i] != 0,
            burn: self.burn[i],
            irr: self.irr[i] != 0,
            slow: self.slow[i] != 0,
            tower_slot: self.tower_slot[i],
            born_at: self.born_at[i],
        }
    }
    fn set(&mut self, i: usize, p: Projectile) {
        macro_rules! s{($($f:ident),*)=>{$(self.$f[i]=p.$f;)*}}
        s!(
            x, y, target, target_x, target_y, v, damage, splash, kb, burn, tower_slot, born_at
        );
        self.pierce[i] = u8::from(p.pierce);
        self.irr[i] = u8::from(p.irr);
        self.slow[i] = u8::from(p.slow);
    }
    fn truncate(&mut self, n: usize) {
        macro_rules! t{($($f:ident),*)=>{$(self.$f.truncate(n);)*}}
        t!(
            x, y, target, target_x, target_y, v, damage, splash, kb, pierce, burn, irr, slow,
            tower_slot, born_at
        );
    }
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
    #[serde(default)]
    pub tick: Option<u32>,
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
    #[serde(default)]
    pub params: Option<ParamsPatch>,
    #[serde(default)]
    pub initial: Option<InitialState>,
    #[serde(default)]
    pub endless: bool,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitialState {
    pub gold: Option<f64>,
    pub lives: Option<i32>,
    pub wave: Option<usize>,
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Telemetry {
    pub leaks: u64,
    pub leak_damage: i64,
    pub effective_damage: f64,
    pub overkill_damage: f64,
    pub projectiles_fired: u64,
    pub projectile_impacts: u64,
    pub projectile_latency_seconds: f64,
    pub wasted_projectiles: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct PerkIntervention {
    pub perk: usize,
    pub enabled: bool,
    pub activation_wave: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CounterfactualRun {
    pub run: RunResult,
    pub telemetry: Telemetry,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CounterfactualSummary {
    pub wave: usize,
    pub lives: i32,
    pub gold: f64,
    pub seconds: f64,
    pub ticks: usize,
    pub capped: bool,
    pub telemetry: Telemetry,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugTick {
    pub tick: usize,
    pub gold: f64,
    pub lives: i32,
    pub kills: u64,
    pub enemies: Vec<(f64, f64, f64, f64)>,
    pub projectiles: Vec<(f64, f64, usize, usize)>,
    pub tower_damage: Vec<f64>,
}

#[derive(Clone)]
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
    enemies: EnemySoa,
    towers: Vec<Tower>,
    projectiles: ProjectileSoa,
    born_scratch: Vec<Wave>,
    split_scratch: Vec<(Wave, f64, Option<f64>)>,
    kills: u64,
    over: bool,
    won: bool,
    next_enemy: u64,
    relics: [bool; 12],
    perk_overrides: [Option<bool>; 12],
    pick: Vec<usize>,
    power_cooldowns: [f64; 3],
    overdrive: f64,
    endless: bool,
    telemetry: Telemetry,
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
            spawns: Vec::with_capacity(4),
            t: 0.,
            enemies: EnemySoa::with_capacity(256),
            towers: Vec::with_capacity(32),
            projectiles: ProjectileSoa::with_capacity(1024),
            born_scratch: Vec::with_capacity(4),
            split_scratch: Vec::with_capacity(64),
            kills: 0,
            over: false,
            won: false,
            next_enemy: 1,
            relics: [false; 12],
            perk_overrides: [None; 12],
            pick: vec![],
            power_cooldowns: [0.; 3],
            overdrive: 0.,
            endless: false,
            telemetry: Telemetry::default(),
        };
        s.set_map(map);
        s
    }
    fn fork(&self) -> Self {
        let mut fork = self.clone();
        fork.born_scratch.reserve(4);
        fork.split_scratch.reserve(16);
        fork
    }
    fn apply_initial(&mut self, initial: Option<&InitialState>) {
        let Some(initial) = initial else { return };
        if let Some(value) = initial.gold {
            self.gold = value
        }
        if let Some(value) = initial.lives {
            self.lives = value
        }
        if let Some(value) = initial.wave {
            self.wave = value
        }
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
    fn modifiers(&self, t: &Tower) -> Option<&Modifiers> {
        if let Some(f) = t.fusion {
            Some(&self.params.fusions[f].modifiers)
        } else {
            match t.branch {
                1 => Some(&self.params.towers[t.i].branch1),
                2 => Some(&self.params.towers[t.i].branch2),
                _ => None,
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
                    self.towers[i].buff = self.towers[i]
                        .buff
                        .max(1. + (if branch == 1 { 0.5 } else { 0.25 }) * level_multiplier(level));
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
                    self.gold += (self.towers[i].inv
                        * if self.perk_active(4) {
                            self.params.rules.doctrine_sell_refund
                        } else {
                            self.params.rules.sell_refund
                        })
                    .round();
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
            "relic" => self.choose_relic_id(a.tower),
            "power" => self.power(a.tower, a.x, a.y),
            "early" => self.early_call(),
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
        let c = (self.params.towers[self.towers[i].i].cost
            * if self.perk_active(5) {
                self.params.rules.doctrine_upgrade_cost
            } else {
                self.params.rules.upgrade_cost
            })
        .round();
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
        if self.towers[i].fusion.is_some() || self.towers[j].fusion.is_some() {
            return false;
        }
        let (ai, bi) = (self.towers[i].i, self.towers[j].i);
        let recipe = self.params.fusions.iter().position(|fusion| {
            let ascendant = fusion.a == usize::MAX
                && ai == bi
                && self.towers[i].branch == self.towers[j].branch;
            self.towers[i].level >= fusion.level
                && self.towers[j].level >= fusion.level
                && (ascendant
                    || ((ai == fusion.a && bi == fusion.b) || (ai == fusion.b && bi == fusion.a)))
        });
        let Some(r) = recipe else { return false };
        let base = self.params.fusions[r].base;
        let inv = self.towers[j].inv;
        let keep = if j < i { i - 1 } else { i };
        self.towers.remove(j);
        if base != usize::MAX {
            self.towers[keep].i = base;
        }
        self.towers[keep].fusion = Some(r);
        self.towers[keep].branch = 0;
        self.towers[keep].inv += inv;
        self.towers[keep].mb *= 1.1;
        self.calc_buffs();
        true
    }
    fn start_wave(&mut self) {
        if self.over || !self.pick.is_empty() || (self.wave >= 50 && !self.endless) {
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
    fn early_call(&mut self) -> bool {
        if !self.phase_wave
            || self.over
            || !self.pick.is_empty()
            || self.wave >= 50
            || self.spawns.len() > 1
        {
            return false;
        }
        self.gold += (2. + self.wave as f64 * 0.5).round();
        let row = wave_row(&self.params.constants, self.wave + 1);
        self.spawns.push(Spawn { row, sn: 0, st: 0. });
        self.wave += 1;
        true
    }
    fn choose_relic_id(&mut self, relic: usize) -> bool {
        if !self.pick.contains(&relic) {
            return false;
        }
        self.relics[relic] = true;
        if relic == 7 && self.perk_active(7) {
            self.lives += self.params.rules.doctrine_lives;
        }
        self.pick.clear();
        true
    }
    fn choose_relic(&mut self, seed: u64) {
        if self.pick.is_empty() {
            return;
        }
        let choice = (seed.wrapping_add(self.wave as u64) as usize) % self.pick.len();
        let relic = self.pick[choice];
        self.choose_relic_id(relic);
    }
    fn power(&mut self, power: usize, x: i32, y: i32) -> bool {
        let age = self.wave.saturating_sub(usize::from(self.phase_wave)) / 5;
        if power >= 3
            || self.power_cooldowns[power] > 0.
            || age < self.params.rules.power_ages[power]
        {
            return false;
        }
        self.power_cooldowns[power] = self.params.rules.power_cooldowns[power]
            * if self.perk_active(2) {
                self.params.rules.doctrine_cooldown
            } else {
                1.
            };
        match power {
            0 => {
                let damage = self.params.rules.meteor_damage
                    * self.params.constants.hp
                    * self.params.constants.growth.powi(self.wave as i32);
                let radius_sq = self.params.rules.meteor_radius * self.params.rules.meteor_radius;
                for i in 0..self.enemies.len() {
                    let dx = self.enemies.x[i] - x as f64;
                    let dy = self.enemies.y[i] - y as f64;
                    if dx * dx + dy * dy <= radius_sq {
                        self.enemies.hp[i] -= damage;
                    }
                }
            }
            1 => {
                for i in 0..self.enemies.len() {
                    self.enemies.slow[i] = self.params.rules.stasis_duration;
                    self.enemies.slow_f[i] = self.params.rules.stasis_factor;
                }
            }
            2 => self.overdrive = self.params.rules.overdrive_duration,
            _ => unreachable!(),
        }
        true
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
    fn in_range(t: &Tower, enemies: &EnemySoa, i: usize, r: f64) -> bool {
        let dx = enemies.x[i] - t.x as f64 - 0.5;
        let dy = enemies.y[i] - t.y as f64 - 0.5;
        dx * dx + dy * dy <= r * r
    }
    fn enemy_idx(&self, id: u64) -> Option<usize> {
        self.enemies
            .id
            .iter()
            .position(|&candidate| candidate == id)
    }
    fn raw(&mut self, ei: usize, d: f64, tower: Option<usize>) {
        let eff = d.min(self.enemies.hp[ei].max(0.));
        self.telemetry.effective_damage += eff;
        self.telemetry.overkill_damage += (d - eff).max(0.);
        self.enemies.hp[ei] -= d;
        if let Some(t) = tower
            && t < self.towers.len()
        {
            self.towers[t].damage += eff
        }
    }
    fn perk_active(&self, perk: usize) -> bool {
        self.perk_overrides[perk].unwrap_or(self.relics[perk])
    }
    fn apply_perk_intervention(&mut self, intervention: PerkIntervention) {
        if self.wave < intervention.activation_wave {
            return;
        }
        let was_active = self.perk_active(intervention.perk);
        self.perk_overrides[intervention.perk] = Some(intervention.enabled);
        let is_active = self.perk_active(intervention.perk);
        if intervention.perk == 7 && !was_active && is_active {
            self.lives += self.params.rules.doctrine_lives;
        }
    }
    fn damage(&mut self, ei: usize, mut d: f64, pierce: bool, slow_bonus: bool, tower: usize) {
        if !pierce {
            d = (d - self.enemies.armor[ei]).max(1.)
        }
        if self.enemies.cap[ei] > 0. {
            d = d.min(self.enemies.cap[ei])
        }
        if self.enemies.irr[ei] > 0. {
            d *= 1.2
        }
        if slow_bonus && self.enemies.slow[ei] > 0. {
            d *= 1.4
        }
        self.raw(ei, d, Some(tower))
    }
    pub fn update(&mut self, dt: f64) {
        if self.over {
            return;
        }
        self.t += dt;
        self.overdrive = (self.overdrive - dt).max(0.);
        for cooldown in &mut self.power_cooldowns {
            *cooldown = (*cooldown - dt).max(0.);
        }
        self.born_scratch.clear();
        for j in &mut self.spawns {
            j.st -= dt;
            if j.st <= 0. && j.sn < j.row.n {
                self.born_scratch.push(j.row.clone());
                j.sn += 1;
                j.st += if j.row.rush { 0.28 } else { 0.8 }
            }
        }
        self.spawns.retain(|j| j.sn < j.row.n);
        let mut born_rows = std::mem::take(&mut self.born_scratch);
        for r in born_rows.drain(..) {
            self.spawn_enemy(r, 0., None)
        }
        self.born_scratch = born_rows;
        for ti in 0..self.towers.len() {
            let spec = &self.params.towers[self.towers[ti].i];
            let (spec_ramp, spec_aura, spec_field, spec_beam) =
                (spec.ramp, spec.aura, spec.field, spec.beam);
            if !spec_aura && !spec_field && !spec_beam {
                self.towers[ti].cd -= dt
                    * self.towers[ti].buff
                    * if self.overdrive > 0. {
                        self.params.rules.overdrive_multiplier
                    } else {
                        1.
                    };
                // Non-ramping towers cannot change any state through targeting
                // until they are ready to fire. Avoid all modifier and range
                // work while their cooldown is still positive.
                if !spec_ramp && self.towers[ti].cd > 0. {
                    continue;
                }
            }
            let (
                spec_damage,
                spec_rate,
                spec_range,
                spec_splash,
                spec_pierce,
                spec_burn,
                spec_knockback,
                spec_homing,
            ) = (
                spec.damage,
                spec.rate,
                spec.range,
                spec.splash,
                spec.pierce,
                spec.burn,
                spec.knockback,
                spec.homing,
            );
            let (b_dm, b_sp, b_pv, b_rgm, b_kb, b_pi, b_bu, b_bum, b_hc, b_sf, b_ir, b_sl, b_hr) =
                self.modifiers(&self.towers[ti]).map_or(
                    (
                        None, None, None, None, None, None, None, None, None, None, None, None,
                        None,
                    ),
                    |b| {
                        (
                            b.dm, b.sp, b.pv, b.rgm, b.kb, b.pi, b.bu, b.bum, b.hc, b.sf, b.ir,
                            b.sl, b.hr,
                        )
                    },
                );
            let mult =
                level_multiplier(self.towers[ti].level) * b_dm.unwrap_or(1.) * self.towers[ti].mb;
            let range = spec_range
                * b_rgm.unwrap_or(1.)
                * (1. + 0.06 * self.towers[ti].level as f64)
                * if self.perk_active(3) {
                    self.params.rules.doctrine_range
                } else {
                    1.
                };
            if spec_field {
                for ei in 0..self.enemies.len() {
                    if Self::in_range(&self.towers[ti], &self.enemies, ei, range)
                        && self.enemies.slow[ei] < 0.2
                    {
                        self.enemies.slow[ei] = 0.15;
                        self.enemies.slow_f[ei] = b_sf.unwrap_or(if self.perk_active(11) {
                            self.params.rules.doctrine_slow
                        } else {
                            0.6
                        })
                    }
                }
                continue;
            }
            if spec_aura {
                for ei in 0..self.enemies.len() {
                    if Self::in_range(&self.towers[ti], &self.enemies, ei, range) {
                        let d = spec_damage
                            * mult
                            * dt
                            * if self.overdrive > 0. {
                                self.params.rules.overdrive_multiplier
                            } else {
                                1.
                            }
                            * (if self.enemies.irr[ei] > 0. { 1.2 } else { 1. });
                        self.raw(ei, d, Some(ti));
                        self.enemies.irr[ei] = if self.perk_active(10) {
                            self.params.rules.doctrine_irradiate
                        } else {
                            0.5
                        }
                    }
                }
                continue;
            }
            if spec_beam {
                let valid = self.towers[ti]
                    .lock
                    .and_then(|id| self.enemy_idx(id))
                    .filter(|&i| self.enemies.hp[i] > 0. && self.enemies.leaked[i] == 0);
                if valid.is_none() {
                    let mut chosen: Option<usize> = None;
                    for i in 0..self.enemies.len() {
                        if chosen.is_none_or(|old| self.enemies.max[i] > self.enemies.max[old]) {
                            chosen = Some(i);
                        }
                    }
                    self.towers[ti].lock = chosen.map(|i| self.enemies.id[i]);
                    self.towers[ti].heat = 0.
                }
                if let Some(ei) = self.towers[ti].lock.and_then(|id| self.enemy_idx(id)) {
                    self.towers[ti].heat += dt * b_hr.unwrap_or(1.);
                    let d = spec_damage
                        * (1. + self.towers[ti].heat)
                        * mult
                        * dt
                        * if self.overdrive > 0. {
                            self.params.rules.overdrive_multiplier
                        } else {
                            1.
                        }
                        * (if self.enemies.irr[ei] > 0. { 1.2 } else { 1. });
                    self.raw(ei, d, Some(ti));
                    if b_ir.is_some() {
                        self.enemies.irr[ei] = if self.perk_active(10) {
                            self.params.rules.doctrine_irradiate
                        } else {
                            0.5
                        }
                    }
                }
                continue;
            }
            let mut target: Option<usize> = None;
            for i in 0..self.enemies.len() {
                if !Self::in_range(&self.towers[ti], &self.enemies, i, range)
                    || (spec_homing && self.enemies.hp[i] <= self.enemies.inc[i])
                {
                    continue;
                }
                let better = target.is_none_or(|old| {
                    if spec_homing {
                        self.enemies.hp[i] - self.enemies.inc[i]
                            > self.enemies.hp[old] - self.enemies.inc[old] + 1e-9
                    } else {
                        self.enemies.d[i] > self.enemies.d[old]
                    }
                });
                if better {
                    target = Some(i);
                }
            }
            if spec_ramp {
                let id = target.map(|i| self.enemies.id[i]);
                if self.towers[ti].lock != id {
                    self.towers[ti].lock = id;
                    self.towers[ti].heat = 0.
                } else if id.is_some() {
                    self.towers[ti].heat = (self.towers[ti].heat + dt).min(b_hc.unwrap_or(6.))
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
                    / if spec_ramp {
                        spec_rate + self.towers[ti].heat
                    } else {
                        spec_rate
                    };
                let damage = spec_damage * mult;
                self.projectiles.push(Projectile {
                    x: self.towers[ti].x as f64 + 0.5,
                    y: self.towers[ti].y as f64 + 0.5,
                    target: self.enemies.id[ei],
                    target_x: self.enemies.x[ei],
                    target_y: self.enemies.y[ei],
                    v: (if spec_homing { 5. } else { 12. })
                        * (1. + 0.1 * self.towers[ti].level as f64)
                        * b_pv.unwrap_or(1.)
                        * if self.perk_active(8) {
                            self.params.rules.doctrine_projectile_speed
                        } else {
                            1.
                        },
                    damage,
                    splash: b_sp.unwrap_or(spec_splash),
                    kb: b_kb.unwrap_or(spec_knockback),
                    pierce: b_pi.is_some() || spec_pierce,
                    burn: if spec_burn || b_bu.is_some() {
                        mult * b_bum.unwrap_or(1.)
                    } else if self.towers[ti].fire {
                        1.
                    } else {
                        0.
                    },
                    irr: self.towers[ti].rad || b_ir.is_some(),
                    slow: b_sl.is_some(),
                    tower_slot: ti,
                    born_at: self.t,
                });
                self.telemetry.projectiles_fired += 1;
                if spec_homing {
                    self.enemies.inc[ei] += damage
                }
            }
        }
        let processing_len = self.projectiles.len();
        let mut write_projectile = 0usize;
        for projectile_index in 0..processing_len {
            let mut p = self.projectiles.get(projectile_index);
            let target_index = self.enemy_idx(p.target);
            if let Some(ei) = target_index {
                p.target_x = self.enemies.x[ei];
                p.target_y = self.enemies.y[ei];
            }
            let (tx, ty) = (p.target_x, p.target_y);
            let (dx, dy) = (tx - p.x, ty - p.y);
            // Match V8 Math.hypot's scaled two-term summation. Small rounding
            // differences here change which simultaneous projectile gets the
            // final hit and eventually cause oracle drift.
            let hi = dx.abs().max(dy.abs());
            let len = if hi == 0. {
                0.
            } else {
                let ratio = dx.abs().min(dy.abs()) / hi;
                hi * (1. + ratio * ratio).sqrt()
            };
            let step = p.v * dt;
            if len <= step {
                self.telemetry.projectile_impacts += 1;
                self.telemetry.projectile_latency_seconds += self.t - p.born_at;
                let mut effective_targets = 0usize;
                if p.splash > 0. {
                    let splash = p.splash
                        * if self.perk_active(9) {
                            self.params.rules.doctrine_splash
                        } else {
                            1.
                        };
                    for i in 0..self.enemies.len() {
                        let dx = self.enemies.x[i] - tx;
                        let dy = self.enemies.y[i] - ty;
                        if dx * dx + dy * dy <= splash * splash {
                            if self.enemies.hp[i] > 0. && self.enemies.leaked[i] == 0 {
                                effective_targets += 1;
                            }
                            self.damage(i, p.damage, p.pierce, true, p.tower_slot)
                        }
                    }
                    if let Some(i) = target_index {
                        self.enemies.inc[i] -= p.damage
                    }
                } else if let Some(ei) =
                    target_index.filter(|&i| self.enemies.hp[i] > 0. && self.enemies.leaked[i] == 0)
                {
                    effective_targets = 1;
                    self.damage(ei, p.damage, p.pierce, false, p.tower_slot)
                }
                if effective_targets == 0 {
                    self.telemetry.wasted_projectiles += 1;
                }
                if let Some(i) = target_index {
                    if p.kb > 0. && self.enemies.hp[i] > 0. {
                        self.enemies.d[i] = (self.enemies.d[i]
                            - p.kb
                                * if self.perk_active(6) {
                                    self.params.rules.doctrine_knockback
                                } else {
                                    1.
                                }
                                * (if self.enemies.boss[i] != 0 {
                                    if self.perk_active(6) {
                                        self.params.rules.doctrine_boss_knockback
                                    } else {
                                        0.15
                                    }
                                } else {
                                    1.
                                }))
                        .max(0.)
                    }
                    if p.burn > 0. && self.enemies.hp[i] > 0. {
                        self.enemies.burn_t[i] = 3.;
                        self.enemies.burn_n[i] =
                            (self.enemies.burn_n[i] + 1.).min(if self.perk_active(0) {
                                self.params.rules.doctrine_burn_cap
                            } else {
                                self.params.rules.burn_cap
                            });
                        self.enemies.burn_p[i] = self.enemies.burn_p[i].max(p.burn)
                    }
                    if p.irr && self.enemies.hp[i] > 0. {
                        self.enemies.irr[i] = if self.perk_active(10) {
                            self.params.rules.doctrine_irradiate
                        } else {
                            0.5
                        }
                    }
                    if p.slow && self.enemies.hp[i] > 0. && self.enemies.slow[i] < 0.2 {
                        self.enemies.slow[i] = 0.5;
                        self.enemies.slow_f[i] = 0.6
                    }
                }
            } else {
                p.x += dx / len * step;
                p.y += dy / len * step;
                self.projectiles.set(write_projectile, p);
                write_projectile += 1;
            }
        }
        self.projectiles.truncate(write_projectile);
        let plen = (self.path.len() - 1) as f64;
        for ei in 0..self.enemies.len() {
            if self.enemies.dash[ei] != 0 {
                self.enemies.dc[ei] -= dt;
                if self.enemies.dc[ei] <= 0. {
                    self.enemies.dc[ei] += 2.5
                }
            }
            let sm = (if self.enemies.dash[ei] != 0 && self.enemies.dc[ei] < 0.6 {
                2.4
            } else {
                1.
            }) * (if self.enemies.slow[ei] > 0. {
                self.enemies.slow_f[ei]
            } else {
                1.
            });
            self.enemies.slow[ei] -= dt;
            self.enemies.irr[ei] -= dt;
            self.enemies.d[ei] += self.enemies.speed[ei] * sm * dt;
            if self.enemies.regen[ei] > 0.
                && self.enemies.hp[ei] > 0.
                && self.enemies.hp[ei] < self.enemies.max[ei]
            {
                self.enemies.hp[ei] =
                    (self.enemies.hp[ei] + self.enemies.regen[ei] * dt).min(self.enemies.max[ei])
            }
            if self.enemies.burn_t[ei] > 0. {
                let d = 4.
                    * self.enemies.burn_n[ei]
                    * self.enemies.burn_p[ei]
                    * (if self.enemies.irr[ei] > 0. { 2.4 } else { 1. })
                    * dt;
                let eff = d.min(self.enemies.hp[ei].max(0.));
                self.telemetry.effective_damage += eff;
                self.telemetry.overkill_damage += (d - eff).max(0.);
                self.enemies.hp[ei] -= d;
                self.enemies.burn_t[ei] -= dt;
                if self.enemies.burn_t[ei] <= 0. {
                    self.enemies.burn_n[ei] = 0.
                }
            }
            let p = self.pos(self.enemies.d[ei].min(plen));
            self.enemies.x[ei] = p.0;
            self.enemies.y[ei] = p.1;
            if self.enemies.d[ei] >= plen && self.enemies.leaked[ei] == 0 {
                self.enemies.leaked[ei] = 1;
                self.lives -= self.enemies.leak[ei];
                self.telemetry.leaks += 1;
                self.telemetry.leak_damage += i64::from(self.enemies.leak[ei]);
            }
        }
        // A JS projectile owns a reference to its target object. Update the
        // retained snapshot after enemy movement so, if that enemy is removed
        // below, the projectile still flies to the object's true final point.
        let enemies = &self.enemies;
        for projectile in 0..self.projectiles.len() {
            if let Some(i) = enemies
                .id
                .iter()
                .position(|&id| id == self.projectiles.target[projectile])
            {
                self.projectiles.target_x[projectile] = enemies.x[i];
                self.projectiles.target_y[projectile] = enemies.y[i];
            }
        }
        let mut dead = std::mem::take(&mut self.split_scratch);
        dead.clear();
        let mut removed = false;
        for i in 0..self.enemies.len() {
            if self.enemies.leaked[i] == 0 && self.enemies.hp[i] <= 0. {
                if self.enemies.split[i] != 0 {
                    dead.push((
                        self.enemies.row[i].clone(),
                        (self.enemies.d[i] - 0.6).max(0.),
                        Some(0.4),
                    ));
                    dead.push((
                        self.enemies.row[i].clone(),
                        (self.enemies.d[i] - 1.2).max(0.),
                        Some(0.4),
                    ));
                }
                self.gold += self.enemies.bounty[i]
                    + if self.perk_active(1) {
                        self.params.rules.doctrine_bounty
                    } else {
                        0.
                    };
                self.kills += 1;
                removed = true;
            } else if self.enemies.leaked[i] != 0 {
                removed = true;
            }
        }
        if removed {
            self.enemies.retain_alive();
        }
        for (drow, d, f) in dead.drain(..) {
            self.spawn_enemy(drow, d, f)
        }
        self.split_scratch = dead;
        if self.lives <= 0 {
            self.over = true;
            self.won = false;
            self.phase_wave = false;
            return;
        }
        if self.phase_wave && self.spawns.is_empty() && self.enemies.is_empty() {
            if self.wave >= 50 && !self.endless {
                self.over = true;
                self.won = true;
                self.phase_wave = false
            } else {
                self.phase_wave = false;
                if self.wave.is_multiple_of(10) {
                    let mut sd = self.wave * 7 + self.map * 3;
                    let available = self.relics.iter().filter(|&&x| !x).count();
                    while self.pick.len() < 3 && self.pick.len() < available {
                        let r = sd % 12;
                        sd += 1;
                        if !self.relics[r] && !self.pick.contains(&r) {
                            self.pick.push(r);
                        }
                    }
                }
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

fn round6(x: f64) -> f64 {
    (x * 1_000_000.).round() / 1_000_000.
}

fn apply_matching_actions(
    sim: &mut Sim,
    pending: &mut Vec<Action>,
    rejected: &mut Vec<Action>,
    mut due: impl FnMut(&Action) -> bool,
) {
    let mut index = 0;
    while index < pending.len() {
        if due(&pending[index]) {
            let action = pending.remove(index);
            if !sim.apply(&action) {
                rejected.push(action);
            }
        } else {
            index += 1;
        }
    }
}

pub fn run(input: &Input, params: Params) -> RunResult {
    run_counterfactual(input, params, None).run
}

struct ExecutionState {
    sim: Sim,
    pending: Vec<Action>,
    rejected: Vec<Action>,
    trace: Vec<WaveTrace>,
    total_seconds: f64,
    collect_trace: bool,
    ticks: usize,
}

pub struct CounterfactualCheckpoint {
    state: ExecutionState,
    applied_actions: usize,
    checkpoint_relics: [bool; 12],
    final_relics: [bool; 12],
    baseline: Option<CounterfactualSummary>,
}

impl CounterfactualCheckpoint {
    fn state_for_input(&self, input: &Input) -> ExecutionState {
        let mut state = self.state.fork();
        state.pending = input.actions[self.applied_actions.min(input.actions.len())..].to_vec();
        state
            .pending
            .sort_by_key(|action| (action.wave, action.tick.unwrap_or(0)));
        state
    }

    fn baseline_arm(&self, perk: usize) -> Option<(bool, CounterfactualSummary)> {
        let enabled = if self.checkpoint_relics[perk] {
            true
        } else if !self.final_relics[perk] {
            false
        } else {
            return None;
        };
        Some((enabled, self.baseline.as_ref()?.clone()))
    }
}

impl ExecutionState {
    fn new(input: &Input, params: Params) -> Self {
        Self::with_trace(input, params, true)
    }

    fn compact(input: &Input, params: Params) -> Self {
        Self::with_trace(input, params, false)
    }

    fn with_trace(input: &Input, params: Params, collect_trace: bool) -> Self {
        let mut sim = Sim::new(params, input.map);
        sim.endless = input.endless;
        sim.apply_initial(input.initial.as_ref());
        let mut pending = input.actions.clone();
        pending.sort_by_key(|action| (action.wave, action.tick.unwrap_or(0)));
        Self {
            sim,
            pending,
            rejected: vec![],
            trace: vec![],
            total_seconds: 0.,
            collect_trace,
            ticks: 0,
        }
    }

    fn fork(&self) -> Self {
        Self {
            sim: self.sim.fork(),
            pending: self.pending.clone(),
            rejected: self.rejected.clone(),
            trace: self.trace.clone(),
            total_seconds: self.total_seconds,
            collect_trace: self.collect_trace,
            ticks: self.ticks,
        }
    }
}

fn advance_execution(
    state: &mut ExecutionState,
    input: &Input,
    intervention: Option<PerkIntervention>,
    stop_before_wave: Option<usize>,
) {
    while !state.sim.over && state.sim.wave < input.max_wave && state.ticks < EXECUTION_TICK_CAP {
        if stop_before_wave.is_some_and(|wave| state.sim.wave >= wave) {
            break;
        }
        let current_wave = state.sim.wave;
        if let Some(intervention) = intervention {
            state.sim.apply_perk_intervention(intervention);
        }
        apply_matching_actions(
            &mut state.sim,
            &mut state.pending,
            &mut state.rejected,
            |action| action.wave <= current_wave && action.tick.is_none() && action.op == "relic",
        );
        if !state.sim.pick.is_empty() {
            state.sim.choose_relic(input.seed);
        }
        if let Some(intervention) = intervention {
            state.sim.apply_perk_intervention(intervention);
        }
        apply_matching_actions(
            &mut state.sim,
            &mut state.pending,
            &mut state.rejected,
            |action| action.wave <= current_wave && action.tick.is_none() && action.op != "relic",
        );
        if !state.sim.phase_wave {
            state.sim.start_wave()
        }
        let started = state.sim.t;
        let mut wave_tick = 0u32;
        while state.sim.phase_wave && !state.sim.over && state.ticks < EXECUTION_TICK_CAP {
            let current_wave = state.sim.wave;
            apply_matching_actions(
                &mut state.sim,
                &mut state.pending,
                &mut state.rejected,
                |action| {
                    action.wave <= current_wave && action.tick.is_some_and(|tick| tick <= wave_tick)
                },
            );
            state.sim.update(DT);
            state.ticks += 1;
            wave_tick += 1;
        }
        let seconds = state.sim.t - started;
        state.total_seconds += round6(seconds);
        if state.collect_trace {
            state.trace.push(state.sim.trace(seconds));
        }
    }
}

fn finish_execution(state: ExecutionState, input: &Input) -> CounterfactualRun {
    let run = RunResult {
        protocol: 1,
        seed: input.seed.to_string(),
        map: input.map,
        trace: state.trace,
        rejected: state.rejected,
        pending: state.pending.len(),
        result: state.sim.trace(state.total_seconds),
    };
    let mut telemetry = state.sim.telemetry;
    telemetry.wasted_projectiles += state.sim.projectiles.len() as u64;
    CounterfactualRun { run, telemetry }
}

fn finish_summary(state: ExecutionState, max_wave: usize) -> CounterfactualSummary {
    summarize(&state.sim, state.total_seconds, state.ticks, max_wave)
}

fn summarize(
    sim: &Sim,
    total_seconds: f64,
    ticks: usize,
    max_wave: usize,
) -> CounterfactualSummary {
    let mut telemetry = sim.telemetry.clone();
    telemetry.wasted_projectiles += sim.projectiles.len() as u64;
    let completed = sim.over || (sim.wave >= max_wave && !sim.phase_wave);
    CounterfactualSummary {
        wave: sim.wave,
        lives: sim.lives,
        gold: round6(sim.gold),
        seconds: round6(total_seconds),
        ticks,
        capped: ticks >= EXECUTION_TICK_CAP && !completed,
        telemetry,
    }
}

pub fn run_counterfactual(
    input: &Input,
    params: Params,
    intervention: Option<PerkIntervention>,
) -> CounterfactualRun {
    let mut state = ExecutionState::new(input, params);
    advance_execution(&mut state, input, intervention, None);
    finish_execution(state, input)
}

pub fn run_counterfactual_pair(
    input: &Input,
    params: Params,
    perk: usize,
    activation_wave: usize,
) -> (CounterfactualRun, CounterfactualRun) {
    let mut prefix = ExecutionState::new(input, params);
    advance_execution(&mut prefix, input, None, Some(activation_wave));
    let mut off = prefix.fork();
    let mut on = prefix;
    advance_execution(
        &mut off,
        input,
        Some(PerkIntervention {
            perk,
            enabled: false,
            activation_wave,
        }),
        None,
    );
    advance_execution(
        &mut on,
        input,
        Some(PerkIntervention {
            perk,
            enabled: true,
            activation_wave,
        }),
        None,
    );
    (finish_execution(off, input), finish_execution(on, input))
}

pub fn run_counterfactual_batch(
    input: &Input,
    params: Params,
    perks: &[usize],
    activation_wave: usize,
) -> Vec<(usize, CounterfactualRun, CounterfactualRun)> {
    let mut prefix = ExecutionState::new(input, params);
    advance_execution(&mut prefix, input, None, Some(activation_wave));
    perks
        .iter()
        .map(|&perk| {
            let mut off = prefix.fork();
            let mut on = prefix.fork();
            advance_execution(
                &mut off,
                input,
                Some(PerkIntervention {
                    perk,
                    enabled: false,
                    activation_wave,
                }),
                None,
            );
            advance_execution(
                &mut on,
                input,
                Some(PerkIntervention {
                    perk,
                    enabled: true,
                    activation_wave,
                }),
                None,
            );
            (
                perk,
                finish_execution(off, input),
                finish_execution(on, input),
            )
        })
        .collect()
}

pub fn run_counterfactual_summary_pair(
    input: &Input,
    params: Params,
    perk: usize,
    activation_wave: usize,
) -> (CounterfactualSummary, CounterfactualSummary) {
    let mut prefix = ExecutionState::compact(input, params);
    advance_execution(&mut prefix, input, None, Some(activation_wave));
    let mut off = prefix.fork();
    let mut on = prefix;
    advance_execution(
        &mut off,
        input,
        Some(PerkIntervention {
            perk,
            enabled: false,
            activation_wave,
        }),
        None,
    );
    advance_execution(
        &mut on,
        input,
        Some(PerkIntervention {
            perk,
            enabled: true,
            activation_wave,
        }),
        None,
    );
    (
        finish_summary(off, input.max_wave),
        finish_summary(on, input.max_wave),
    )
}

pub fn run_counterfactual_summary_batch(
    input: &Input,
    params: Params,
    perks: &[usize],
    activation_wave: usize,
) -> Vec<(usize, CounterfactualSummary, CounterfactualSummary)> {
    let mut prefix = ExecutionState::compact(input, params);
    advance_execution(&mut prefix, input, None, Some(activation_wave));
    perks
        .iter()
        .map(|&perk| {
            let mut off = prefix.fork();
            let mut on = prefix.fork();
            advance_execution(
                &mut off,
                input,
                Some(PerkIntervention {
                    perk,
                    enabled: false,
                    activation_wave,
                }),
                None,
            );
            advance_execution(
                &mut on,
                input,
                Some(PerkIntervention {
                    perk,
                    enabled: true,
                    activation_wave,
                }),
                None,
            );
            (
                perk,
                finish_summary(off, input.max_wave),
                finish_summary(on, input.max_wave),
            )
        })
        .collect()
}

pub fn run_counterfactual_summary_pair_from_checkpoint(
    input: &Input,
    checkpoint: &CounterfactualCheckpoint,
    perk: usize,
    activation_wave: usize,
    reuse_baseline: bool,
) -> (CounterfactualSummary, CounterfactualSummary) {
    let prefix = checkpoint.state_for_input(input);
    if reuse_baseline && let Some((enabled, baseline)) = checkpoint.baseline_arm(perk) {
        let mut counterfactual = prefix;
        advance_execution(
            &mut counterfactual,
            input,
            Some(PerkIntervention {
                perk,
                enabled: !enabled,
                activation_wave,
            }),
            None,
        );
        let counterfactual = finish_summary(counterfactual, input.max_wave);
        return if enabled {
            (counterfactual, baseline)
        } else {
            (baseline, counterfactual)
        };
    }
    let mut off = prefix.fork();
    let mut on = prefix;
    advance_execution(
        &mut off,
        input,
        Some(PerkIntervention {
            perk,
            enabled: false,
            activation_wave,
        }),
        None,
    );
    advance_execution(
        &mut on,
        input,
        Some(PerkIntervention {
            perk,
            enabled: true,
            activation_wave,
        }),
        None,
    );
    (
        finish_summary(off, input.max_wave),
        finish_summary(on, input.max_wave),
    )
}

pub fn run_counterfactual_summary_batch_from_checkpoint(
    input: &Input,
    checkpoint: &CounterfactualCheckpoint,
    perks: &[usize],
    activation_wave: usize,
    reuse_baseline: bool,
) -> Vec<(usize, CounterfactualSummary, CounterfactualSummary)> {
    let prefix = checkpoint.state_for_input(input);
    perks
        .iter()
        .map(|&perk| {
            if reuse_baseline && let Some((enabled, baseline)) = checkpoint.baseline_arm(perk) {
                let mut counterfactual = prefix.fork();
                advance_execution(
                    &mut counterfactual,
                    input,
                    Some(PerkIntervention {
                        perk,
                        enabled: !enabled,
                        activation_wave,
                    }),
                    None,
                );
                let counterfactual = finish_summary(counterfactual, input.max_wave);
                return if enabled {
                    (perk, counterfactual, baseline)
                } else {
                    (perk, baseline, counterfactual)
                };
            }
            let mut off = prefix.fork();
            let mut on = prefix.fork();
            advance_execution(
                &mut off,
                input,
                Some(PerkIntervention {
                    perk,
                    enabled: false,
                    activation_wave,
                }),
                None,
            );
            advance_execution(
                &mut on,
                input,
                Some(PerkIntervention {
                    perk,
                    enabled: true,
                    activation_wave,
                }),
                None,
            );
            (
                perk,
                finish_summary(off, input.max_wave),
                finish_summary(on, input.max_wave),
            )
        })
        .collect()
}

pub fn debug_wave(input: &Input, target_wave: usize, params: Params) -> Vec<DebugTick> {
    let mut sim = Sim::new(params, input.map);
    sim.endless = input.endless;
    sim.apply_initial(input.initial.as_ref());
    let mut pending = input.actions.clone();
    pending.sort_by_key(|a| (a.wave, a.tick.unwrap_or(0)));
    let mut rejected = vec![];
    while !sim.over && sim.wave < target_wave.saturating_sub(1) {
        let current_wave = sim.wave;
        apply_matching_actions(&mut sim, &mut pending, &mut rejected, |action| {
            action.wave <= current_wave && action.tick.is_none() && action.op == "relic"
        });
        if !sim.pick.is_empty() {
            sim.choose_relic(input.seed);
        }
        apply_matching_actions(&mut sim, &mut pending, &mut rejected, |action| {
            action.wave <= current_wave && action.tick.is_none() && action.op != "relic"
        });
        sim.start_wave();
        let mut wave_tick = 0u32;
        while sim.phase_wave && !sim.over {
            let current_wave = sim.wave;
            apply_matching_actions(&mut sim, &mut pending, &mut rejected, |action| {
                action.wave <= current_wave && action.tick.is_some_and(|tick| tick <= wave_tick)
            });
            sim.update(DT);
            wave_tick += 1;
        }
    }
    if sim.over {
        return vec![];
    }
    let current_wave = sim.wave;
    apply_matching_actions(&mut sim, &mut pending, &mut rejected, |action| {
        action.wave <= current_wave && action.tick.is_none() && action.op == "relic"
    });
    if !sim.pick.is_empty() {
        sim.choose_relic(input.seed);
    }
    apply_matching_actions(&mut sim, &mut pending, &mut rejected, |action| {
        action.wave <= current_wave && action.tick.is_none() && action.op != "relic"
    });
    sim.start_wave();
    let mut out = vec![];
    let mut tick = 0;
    while sim.phase_wave && !sim.over {
        let current_wave = sim.wave;
        apply_matching_actions(&mut sim, &mut pending, &mut rejected, |action| {
            action.wave <= current_wave && action.tick.is_some_and(|at| at as usize <= tick)
        });
        sim.update(DT);
        tick += 1;
        out.push(DebugTick {
            tick,
            gold: sim.gold,
            lives: sim.lives,
            kills: sim.kills,
            enemies: (0..sim.enemies.len())
                .map(|i| {
                    (
                        sim.enemies.d[i],
                        sim.enemies.hp[i],
                        sim.enemies.x[i],
                        sim.enemies.y[i],
                    )
                })
                .collect(),
            projectiles: (0..sim.projectiles.len())
                .map(|i| {
                    (
                        sim.projectiles.x[i],
                        sim.projectiles.y[i],
                        sim.enemy_idx(sim.projectiles.target[i])
                            .unwrap_or(usize::MAX),
                        sim.projectiles.tower_slot[i],
                    )
                })
                .collect(),
            tower_damage: sim.towers.iter().map(|t| t.damage).collect(),
        });
    }
    out
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Genome {
    pub tower_weights: [f64; 10],
    pub upgrade_weight: f64,
    pub branch_bias: f64,
    pub merge_weight: f64,
    pub fusion_weights: [f64; 6],
    pub placement: [f64; 5],
    pub doctrine_weights: [f64; 12],
    pub power_weights: [f64; 3],
    pub power_ticks: [u32; 3],
    pub early_call_weight: f64,
    pub early_call_tick: u32,
    pub reaction_ticks: u32,
    pub apm: f64,
}
impl Genome {
    pub fn random(rng: &mut Xoshiro) -> Self {
        let mut w = [0.; 10];
        for x in &mut w {
            *x = rng.next_f64() * 2. - 1.
        }
        let mut fusion_weights = [0.; 6];
        for x in &mut fusion_weights {
            *x = rng.next_f64() * 2. - 1.
        }
        let mut doctrine_weights = [0.; 12];
        for weight in &mut doctrine_weights {
            *weight = rng.next_f64() * 2. - 1.;
        }
        let mut power_weights = [0.; 3];
        for weight in &mut power_weights {
            *weight = rng.next_f64() * 2. - 1.;
        }
        Self {
            tower_weights: w,
            upgrade_weight: rng.next_f64() * 1.5 - 0.25,
            branch_bias: rng.next_f64() * 2. - 1.,
            merge_weight: rng.next_f64() - 0.5,
            fusion_weights,
            placement: [
                rng.next_f64(),
                rng.next_f64(),
                rng.next_f64(),
                rng.next_f64(),
                rng.next_f64(),
            ],
            doctrine_weights,
            power_weights,
            power_ticks: [90, 45, 15].map(|base| base + (rng.next_u64() % 91) as u32),
            early_call_weight: rng.next_f64() * 2. - 1.,
            early_call_tick: 240 + (rng.next_u64() % 481) as u32,
            reaction_ticks: (rng.next_u64() % 31) as u32,
            apm: 35. + rng.next_f64() * 85.,
        }
    }

    pub fn to_vector(&self) -> Vec<f64> {
        let mut vector = Vec::with_capacity(46);
        vector.extend(self.tower_weights);
        vector.extend([self.upgrade_weight, self.branch_bias, self.merge_weight]);
        vector.extend(self.fusion_weights);
        vector.extend(self.placement);
        vector.extend(self.doctrine_weights);
        vector.extend(self.power_weights);
        vector.extend(self.power_ticks.map(|tick| tick as f64));
        vector.extend([
            self.early_call_weight,
            self.early_call_tick as f64,
            self.reaction_ticks as f64,
            self.apm,
        ]);
        vector
    }

    pub fn from_vector(vector: &[f64]) -> Self {
        assert_eq!(vector.len(), 46);
        let mut values = vector.iter().copied();
        let tower_weights = std::array::from_fn(|_| values.next().unwrap());
        let upgrade_weight = values.next().unwrap();
        let branch_bias = values.next().unwrap();
        let merge_weight = values.next().unwrap();
        let fusion_weights = std::array::from_fn(|_| values.next().unwrap());
        let placement = std::array::from_fn(|_| values.next().unwrap());
        let doctrine_weights = std::array::from_fn(|_| values.next().unwrap());
        let power_weights = std::array::from_fn(|_| values.next().unwrap());
        let power_ticks =
            std::array::from_fn(|_| values.next().unwrap().round().clamp(0., 1800.) as u32);
        let early_call_weight = values.next().unwrap();
        let early_call_tick = values.next().unwrap().round().clamp(30., 1800.) as u32;
        let reaction_ticks = values.next().unwrap().round().clamp(0., 90.) as u32;
        let apm = values.next().unwrap().clamp(10., 180.);
        Self {
            tower_weights,
            upgrade_weight,
            branch_bias,
            merge_weight,
            fusion_weights,
            placement,
            doctrine_weights,
            power_weights,
            power_ticks,
            early_call_weight,
            early_call_tick,
            reaction_ticks,
            apm,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct PolicyRun {
    pub run: RunResult,
    pub actions: Vec<Action>,
    pub fusion_counts: [usize; 6],
    pub tower_counts: [usize; 10],
    pub branch_counts: [[usize; 2]; 10],
    pub doctrine_counts: [usize; 12],
    pub power_counts: [usize; 3],
    pub early_calls: usize,
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
    run_policy_internal(genome, seed, map, max_wave, params, None).0
}

pub fn run_policy_with_checkpoint(
    genome: &Genome,
    seed: u64,
    map: usize,
    max_wave: usize,
    params: Params,
    activation_wave: usize,
) -> (PolicyRun, CounterfactualCheckpoint) {
    let (policy, checkpoint) =
        run_policy_internal(genome, seed, map, max_wave, params, Some(activation_wave));
    (policy, checkpoint.unwrap())
}

fn run_policy_internal(
    genome: &Genome,
    seed: u64,
    map: usize,
    max_wave: usize,
    params: Params,
    checkpoint_wave: Option<usize>,
) -> (PolicyRun, Option<CounterfactualCheckpoint>) {
    let mut sim = Sim::new(params, map);
    sim.endless = max_wave > 50;
    let cells = placement_cells(map, genome.placement);
    let mut actions = vec![];
    let mut trace = vec![];
    let mut fusion_counts = [0usize; 6];
    let mut tower_counts = [0usize; 10];
    let mut branch_counts = [[0usize; 2]; 10];
    let mut doctrine_counts = [0usize; 12];
    let mut power_counts = [0usize; 3];
    let mut early_calls = 0usize;
    let mut merges = 0;
    let mut ticks = 0usize;
    let mut total_seconds = 0.;
    let mut checkpoint = None;
    let recipes = sim.params.fusions.clone();

    while !sim.over && sim.wave < max_wave && ticks < EXECUTION_TICK_CAP {
        if checkpoint.is_none() && checkpoint_wave.is_some_and(|wave| sim.wave >= wave) {
            checkpoint = Some(CounterfactualCheckpoint {
                state: ExecutionState {
                    sim: sim.fork(),
                    pending: vec![],
                    rejected: vec![],
                    trace: vec![],
                    total_seconds,
                    collect_trace: false,
                    ticks,
                },
                applied_actions: actions.len(),
                checkpoint_relics: sim.relics,
                final_relics: [false; 12],
                baseline: None,
            });
        }
        if !sim.pick.is_empty() {
            let relic = *sim
                .pick
                .iter()
                .max_by(|&&a, &&b| {
                    genome.doctrine_weights[a].total_cmp(&genome.doctrine_weights[b])
                })
                .unwrap();
            let action = Action {
                wave: sim.wave,
                tick: None,
                op: "relic".into(),
                tower: relic,
                x: 0,
                y: 0,
                branch: 0,
                with: None,
            };
            if sim.apply(&action) {
                doctrine_counts[relic] += 1;
                actions.push(action);
            }
        }
        let decision_window = (3. - genome.reaction_ticks as f64 * DT).max(0.);
        let action_cap = (genome.apm * decision_window / 60.).floor() as usize;
        for _ in 0..action_cap {
            let unlocked = sim.age();
            let preferred_fusion = (0..6)
                .filter(|&r| r == 5 || (recipes[r].a <= unlocked && recipes[r].b <= unlocked))
                .max_by(|&a, &b| genome.fusion_weights[a].total_cmp(&genome.fusion_weights[b]));
            let mut next: Option<Action> = None;

            if let Some(r) = preferred_fusion.filter(|&r| genome.fusion_weights[r] > 0.) {
                let FusionSpec { a, b, level, .. } = recipes[r].clone();
                'pairs: for i in 0..sim.towers.len() {
                    for j in i + 1..sim.towers.len() {
                        let t = &sim.towers[i];
                        let p = &sim.towers[j];
                        let matches = if r == 5 {
                            t.i == p.i && t.branch == p.branch
                        } else {
                            (t.i == a && p.i == b) || (t.i == b && p.i == a)
                        };
                        if t.level >= level
                            && p.level >= level
                            && t.fusion.is_none()
                            && p.fusion.is_none()
                            && matches
                            && (t.x - p.x).abs().max((t.y - p.y).abs()) == 1
                        {
                            next = Some(Action {
                                wave: sim.wave,
                                tick: None,
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
                if next.is_none() && r != 5 {
                    for &kind in &[a, b] {
                        if let Some(t) = sim.towers.iter().find(|t| t.i == kind && t.level < level)
                        {
                            next = Some(Action {
                                wave: sim.wave,
                                tick: None,
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
                        .or_else(|| policy_cell(&sim, &cells, genome.placement[4]));
                    if let Some((x, y)) = cell {
                        next = Some(Action {
                            wave: sim.wave,
                            tick: None,
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
                                tick: None,
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
                        tick: None,
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
                        tick: None,
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
                if let Some((x, y)) = policy_cell(&sim, &cells, genome.placement[4]) {
                    next = Some(Action {
                        wave: sim.wave,
                        tick: None,
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
                if a.op == "branch"
                    && let Some(tower) = sim.tower_at(a.x, a.y).map(|index| sim.towers[index].i)
                {
                    branch_counts[tower][a.branch.saturating_sub(1).min(1) as usize] += 1
                }
                if a.op == "fuse"
                    && let Some(r) = preferred_fusion
                {
                    fusion_counts[r] += 1
                }
                actions.push(a);
            } else {
                break;
            }
        }
        sim.start_wave();
        let scheduled_wave = sim.wave;
        let (meteor_x, meteor_y) = sim.path[sim.path.len() / 2];
        let mut timed_actions: Vec<Action> = (0..3)
            .filter(|&power| genome.power_weights[power] > 0.)
            .map(|power| Action {
                wave: scheduled_wave,
                tick: Some(genome.power_ticks[power]),
                op: "power".into(),
                tower: power,
                x: meteor_x,
                y: meteor_y,
                branch: 0,
                with: None,
            })
            .collect();
        if genome.early_call_weight > 0. && scheduled_wave < max_wave {
            timed_actions.push(Action {
                wave: scheduled_wave,
                tick: Some(genome.early_call_tick),
                op: "early".into(),
                tower: 0,
                x: 0,
                y: 0,
                branch: 0,
                with: None,
            });
        }
        timed_actions.sort_by_key(|action| action.tick.unwrap_or(0));
        let started = sim.t;
        let mut wave_tick = 0u32;
        while sim.phase_wave && !sim.over && ticks < EXECUTION_TICK_CAP {
            while timed_actions
                .first()
                .is_some_and(|action| action.tick.is_some_and(|at| at <= wave_tick))
            {
                let action = timed_actions.remove(0);
                if sim.apply(&action) {
                    if action.op == "power" {
                        power_counts[action.tower] += 1;
                    } else if action.op == "early" {
                        early_calls += 1;
                    }
                    actions.push(action);
                }
            }
            sim.update(DT);
            ticks += 1;
            wave_tick += 1;
        }
        let wave_trace = sim.trace(sim.t - started);
        total_seconds += wave_trace.seconds;
        trace.push(wave_trace);
    }
    if checkpoint.is_none() && checkpoint_wave.is_some() {
        checkpoint = Some(CounterfactualCheckpoint {
            state: ExecutionState {
                sim: sim.fork(),
                pending: vec![],
                rejected: vec![],
                trace: vec![],
                total_seconds,
                collect_trace: false,
                ticks,
            },
            applied_actions: actions.len(),
            checkpoint_relics: sim.relics,
            final_relics: [false; 12],
            baseline: None,
        });
    }
    if let Some(checkpoint) = &mut checkpoint {
        checkpoint.final_relics = sim.relics;
        checkpoint.baseline = Some(summarize(&sim, total_seconds, ticks, max_wave));
    }
    let result = RunResult {
        protocol: 1,
        seed: seed.to_string(),
        map,
        trace,
        rejected: vec![],
        pending: 0,
        result: sim.trace(total_seconds),
    };
    (
        PolicyRun {
            run: result,
            actions,
            fusion_counts,
            tower_counts,
            branch_counts,
            doctrine_counts,
            power_counts,
            early_calls,
            merges,
        },
        checkpoint,
    )
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

pub fn placement_cells(map: usize, weights: [f64; 5]) -> Vec<(i32, i32)> {
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

fn policy_cell(sim: &Sim, cells: &[(i32, i32)], adjacency_weight: f64) -> Option<(i32, i32)> {
    cells
        .iter()
        .enumerate()
        .filter(|item| {
            let (x, y) = *item.1;
            sim.tower_at(x, y).is_none()
        })
        .map(|(rank, &(x, y))| {
            let support_neighbors = sim
                .towers
                .iter()
                .filter(|tower| {
                    [3, 7, 8].contains(&tower.i)
                        && (tower.x - x).abs().max((tower.y - y).abs()) == 1
                })
                .count() as f64;
            (
                -(rank as f64) + adjacency_weight * support_neighbors * 20.,
                (x, y),
            )
        })
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, cell)| cell)
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
    pub clear_times: Vec<f64>,
    pub tower_counts: [usize; 10],
    pub branch_counts: [[usize; 2]; 10],
    pub fusion_counts: [usize; 6],
    pub doctrine_counts: [usize; 12],
    pub power_counts: [usize; 3],
    pub early_calls: usize,
    pub tower_damage: [f64; 10],
}
pub type Archive = BTreeMap<(usize, bool, Option<usize>), Elite>;

#[cfg(test)]
mod tests {
    use super::*;

    fn run_independent_counterfactual_summary(
        input: &Input,
        params: Params,
        intervention: PerkIntervention,
    ) -> CounterfactualSummary {
        let mut state = ExecutionState::compact(input, params);
        advance_execution(&mut state, input, Some(intervention), None);
        finish_summary(state, input.max_wave)
    }

    #[test]
    fn core_is_seed_deterministic() {
        let input = Input {
            seed: 42,
            map: 0,
            max_wave: 5,
            actions: vec![],
            params: None,
            initial: None,
            endless: false,
        };
        let a = run(&input, Params::default());
        let b = run(&input, Params::default());
        assert_eq!(a.trace, b.trace);
    }

    #[test]
    fn policy_is_seed_deterministic() {
        let mut rng = Xoshiro::new(9);
        let genome = Genome::random(&mut rng);
        let a = run_policy(&genome, 123, 1, 12, Params::default());
        let b = run_policy(&genome, 123, 1, 12, Params::default());
        assert_eq!(a.run.trace, b.run.trace);
        assert_eq!(a.actions.len(), b.actions.len());
    }

    #[test]
    fn counterfactual_pairs_are_deterministic_and_collect_telemetry() {
        let input = Input {
            seed: 42,
            map: 0,
            max_wave: 2,
            actions: vec![],
            params: None,
            initial: None,
            endless: false,
        };
        let intervention = Some(PerkIntervention {
            perk: 8,
            enabled: true,
            activation_wave: 0,
        });
        let a = run_counterfactual(&input, Params::default(), intervention);
        let b = run_counterfactual(&input, Params::default(), intervention);
        assert_eq!(a.run.trace, b.run.trace);
        assert_eq!(a.telemetry.leaks, b.telemetry.leaks);
        assert!(a.telemetry.leaks > 0);
        assert_eq!(a.telemetry.projectiles_fired, 0);
    }

    #[test]
    fn iron_curtain_intervention_applies_once() {
        let input = Input {
            seed: 17,
            map: 1,
            max_wave: 1,
            actions: vec![],
            params: None,
            initial: None,
            endless: false,
        };
        let params = Params::default();
        let off = run_counterfactual(
            &input,
            params.clone(),
            Some(PerkIntervention {
                perk: 7,
                enabled: false,
                activation_wave: 0,
            }),
        );
        let on = run_counterfactual(
            &input,
            params.clone(),
            Some(PerkIntervention {
                perk: 7,
                enabled: true,
                activation_wave: 0,
            }),
        );
        assert_eq!(
            on.run.result.lives - off.run.result.lives,
            params.rules.doctrine_lives
        );
    }

    #[test]
    fn shared_prefix_pairs_match_independent_runs() {
        for map in 0..3 {
            let mut rng = Xoshiro::new(900 + map as u64);
            let genome = Genome::random(&mut rng);
            let seed = 77_000 + map as u64;
            let policy = run_policy(&genome, seed, map, 15, Params::default());
            let input = Input {
                seed,
                map,
                max_wave: 15,
                actions: policy.actions,
                params: None,
                initial: None,
                endless: false,
            };
            for activation_wave in [0, 5, 10, 14, 20] {
                for perk in 0..12 {
                    let expected_off = run_counterfactual(
                        &input,
                        Params::default(),
                        Some(PerkIntervention {
                            perk,
                            enabled: false,
                            activation_wave,
                        }),
                    );
                    let expected_on = run_counterfactual(
                        &input,
                        Params::default(),
                        Some(PerkIntervention {
                            perk,
                            enabled: true,
                            activation_wave,
                        }),
                    );
                    let (actual_off, actual_on) =
                        run_counterfactual_pair(&input, Params::default(), perk, activation_wave);
                    assert_eq!(
                        serde_json::to_vec(&actual_off).unwrap(),
                        serde_json::to_vec(&expected_off).unwrap(),
                        "off mismatch map={map} perk={perk} activation={activation_wave}"
                    );
                    assert_eq!(
                        serde_json::to_vec(&actual_on).unwrap(),
                        serde_json::to_vec(&expected_on).unwrap(),
                        "on mismatch map={map} perk={perk} activation={activation_wave}"
                    );
                }
            }
        }
    }

    #[test]
    fn shared_prefix_summaries_match_independent_runs_including_termination_evidence() {
        let mut rng = Xoshiro::new(1_109);
        let genome = Genome::random(&mut rng);
        let seed = 81_109;
        let policy = run_policy(&genome, seed, 1, 15, Params::default());
        let input = Input {
            seed,
            map: 1,
            max_wave: 15,
            actions: policy.actions,
            params: None,
            initial: None,
            endless: false,
        };

        for activation_wave in [0, 7, 20] {
            for perk in [0, 8, 11] {
                let expected_off = run_independent_counterfactual_summary(
                    &input,
                    Params::default(),
                    PerkIntervention {
                        perk,
                        enabled: false,
                        activation_wave,
                    },
                );
                let expected_on = run_independent_counterfactual_summary(
                    &input,
                    Params::default(),
                    PerkIntervention {
                        perk,
                        enabled: true,
                        activation_wave,
                    },
                );
                let (actual_off, actual_on) = run_counterfactual_summary_pair(
                    &input,
                    Params::default(),
                    perk,
                    activation_wave,
                );

                assert_eq!(actual_off.ticks, expected_off.ticks);
                assert_eq!(actual_off.capped, expected_off.capped);
                assert_eq!(actual_on.ticks, expected_on.ticks);
                assert_eq!(actual_on.capped, expected_on.capped);
                assert_eq!(
                    serde_json::to_vec(&actual_off).unwrap(),
                    serde_json::to_vec(&expected_off).unwrap(),
                    "off summary mismatch perk={perk} activation={activation_wave}"
                );
                assert_eq!(
                    serde_json::to_vec(&actual_on).unwrap(),
                    serde_json::to_vec(&expected_on).unwrap(),
                    "on summary mismatch perk={perk} activation={activation_wave}"
                );
            }
        }
    }

    #[test]
    fn shared_prefix_batch_matches_individual_pairs() {
        let mut rng = Xoshiro::new(1_337);
        let genome = Genome::random(&mut rng);
        let seed = 91_337;
        let policy = run_policy(&genome, seed, 2, 15, Params::default());
        let input = Input {
            seed,
            map: 2,
            max_wave: 15,
            actions: policy.actions,
            params: None,
            initial: None,
            endless: false,
        };
        let perks: Vec<_> = (0..12).collect();
        let actual = run_counterfactual_batch(&input, Params::default(), &perks, 7);
        for (perk, actual_off, actual_on) in actual {
            let (expected_off, expected_on) =
                run_counterfactual_pair(&input, Params::default(), perk, 7);
            assert_eq!(
                serde_json::to_vec(&actual_off).unwrap(),
                serde_json::to_vec(&expected_off).unwrap(),
                "off mismatch perk={perk}"
            );
            assert_eq!(
                serde_json::to_vec(&actual_on).unwrap(),
                serde_json::to_vec(&expected_on).unwrap(),
                "on mismatch perk={perk}"
            );
        }

        let actual = run_counterfactual_summary_batch(&input, Params::default(), &perks, 7);
        for (perk, actual_off, actual_on) in actual {
            let (expected_off, expected_on) =
                run_counterfactual_summary_pair(&input, Params::default(), perk, 7);
            assert_eq!(actual_off.ticks, expected_off.ticks);
            assert_eq!(actual_off.capped, expected_off.capped);
            assert_eq!(actual_on.ticks, expected_on.ticks);
            assert_eq!(actual_on.capped, expected_on.capped);
            assert_eq!(
                serde_json::to_vec(&actual_off).unwrap(),
                serde_json::to_vec(&expected_off).unwrap(),
                "off summary mismatch perk={perk}"
            );
            assert_eq!(
                serde_json::to_vec(&actual_on).unwrap(),
                serde_json::to_vec(&expected_on).unwrap(),
                "on summary mismatch perk={perk}"
            );
        }
    }

    #[test]
    fn compact_counterfactuals_match_full_results() {
        let mut rng = Xoshiro::new(4_242);
        let genome = Genome::random(&mut rng);
        let seed = 104_242;
        let policy = run_policy(&genome, seed, 1, 20, Params::default());
        let input = Input {
            seed,
            map: 1,
            max_wave: 20,
            actions: policy.actions,
            params: None,
            initial: None,
            endless: false,
        };
        for activation_wave in [0, 5, 14, 20, 24] {
            for perk in 0..12 {
                let (full_off, full_on) =
                    run_counterfactual_pair(&input, Params::default(), perk, activation_wave);
                let (compact_off, compact_on) = run_counterfactual_summary_pair(
                    &input,
                    Params::default(),
                    perk,
                    activation_wave,
                );
                for (full, compact) in [(full_off, compact_off), (full_on, compact_on)] {
                    assert_eq!(compact.wave, full.run.result.wave);
                    assert_eq!(compact.lives, full.run.result.lives);
                    assert_eq!(compact.gold, full.run.result.gold);
                    assert_eq!(compact.seconds, full.run.result.seconds);
                    assert!(compact.ticks > 0);
                    assert!(!compact.capped);
                    assert_eq!(
                        serde_json::to_vec(&compact.telemetry).unwrap(),
                        serde_json::to_vec(&full.telemetry).unwrap()
                    );
                }
            }
        }
    }

    #[test]
    fn policy_checkpoints_match_replayed_prefixes() {
        for map in 0..3 {
            let mut rng = Xoshiro::new(8_000 + map as u64);
            let genome = Genome::random(&mut rng);
            for activation_wave in [0, 10, 14, 20, 24] {
                let seed = 208_000 + map as u64 * 100 + activation_wave as u64;
                let (policy, checkpoint) = run_policy_with_checkpoint(
                    &genome,
                    seed,
                    map,
                    25,
                    Params::default(),
                    activation_wave,
                );
                let mut actions = policy.actions;
                let mut baseline_reusable = true;
                if activation_wave == 14
                    && let Some(target) = actions
                        .iter()
                        .find(|action| action.op == "place" && action.wave <= activation_wave)
                        .cloned()
                {
                    actions.push(Action {
                        wave: activation_wave,
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
                let input = Input {
                    seed,
                    map,
                    max_wave: 25,
                    actions,
                    params: None,
                    initial: None,
                    endless: false,
                };
                for perk in 0..12 {
                    let expected = run_counterfactual_summary_pair(
                        &input,
                        Params::default(),
                        perk,
                        activation_wave,
                    );
                    let actual = run_counterfactual_summary_pair_from_checkpoint(
                        &input,
                        &checkpoint,
                        perk,
                        activation_wave,
                        baseline_reusable,
                    );
                    assert_eq!(actual.0.ticks, expected.0.ticks);
                    assert_eq!(actual.0.capped, expected.0.capped);
                    assert_eq!(actual.1.ticks, expected.1.ticks);
                    assert_eq!(actual.1.capped, expected.1.capped);
                    assert_eq!(
                        serde_json::to_vec(&actual).unwrap(),
                        serde_json::to_vec(&expected).unwrap(),
                        "checkpoint mismatch map={map} perk={perk} activation={activation_wave}"
                    );
                }
            }
        }
    }

    #[test]
    fn checkpoint_batches_match_replayed_batches_including_termination_evidence() {
        let mut rng = Xoshiro::new(12_345);
        let genome = Genome::random(&mut rng);
        let seed = 212_345;
        let activation_wave = 7;
        let (policy, checkpoint) =
            run_policy_with_checkpoint(&genome, seed, 2, 15, Params::default(), activation_wave);
        let input = Input {
            seed,
            map: 2,
            max_wave: 15,
            actions: policy.actions,
            params: None,
            initial: None,
            endless: false,
        };
        let perks: Vec<_> = (0..12).collect();
        let expected =
            run_counterfactual_summary_batch(&input, Params::default(), &perks, activation_wave);

        let baseline = checkpoint.baseline.as_ref().unwrap();
        assert!(baseline.ticks > 0);
        assert!(!baseline.capped);

        for reuse_baseline in [false, true] {
            let actual = run_counterfactual_summary_batch_from_checkpoint(
                &input,
                &checkpoint,
                &perks,
                activation_wave,
                reuse_baseline,
            );
            for ((perk, actual_off, actual_on), (expected_perk, expected_off, expected_on)) in
                actual.iter().zip(&expected)
            {
                assert_eq!(perk, expected_perk);
                assert_eq!(actual_off.ticks, expected_off.ticks);
                assert_eq!(actual_off.capped, expected_off.capped);
                assert_eq!(actual_on.ticks, expected_on.ticks);
                assert_eq!(actual_on.capped, expected_on.capped);
            }
            assert_eq!(
                serde_json::to_vec(&actual).unwrap(),
                serde_json::to_vec(&expected).unwrap(),
                "checkpoint batch mismatch reuse_baseline={reuse_baseline}"
            );
        }
    }

    #[test]
    fn summaries_mark_only_unfinished_tick_limited_runs_as_capped() {
        let input = Input {
            seed: 501,
            map: 0,
            max_wave: 2,
            actions: vec![],
            params: None,
            initial: None,
            endless: false,
        };
        let state = ExecutionState::compact(&input, Params::default());

        let mut below_cap = state.fork();
        below_cap.ticks = EXECUTION_TICK_CAP - 1;
        let below_cap = finish_summary(below_cap, input.max_wave);
        assert_eq!(below_cap.ticks, EXECUTION_TICK_CAP - 1);
        assert!(!below_cap.capped);

        let mut at_cap = state.fork();
        at_cap.ticks = EXECUTION_TICK_CAP;
        let at_cap = finish_summary(at_cap, input.max_wave);
        assert_eq!(at_cap.ticks, EXECUTION_TICK_CAP);
        assert!(at_cap.capped);

        let mut over_at_cap = state.fork();
        over_at_cap.ticks = EXECUTION_TICK_CAP;
        over_at_cap.sim.over = true;
        assert!(!finish_summary(over_at_cap, input.max_wave).capped);

        let mut max_wave_at_cap = state;
        max_wave_at_cap.ticks = EXECUTION_TICK_CAP;
        max_wave_at_cap.sim.wave = input.max_wave;
        assert!(!finish_summary(max_wave_at_cap, input.max_wave).capped);

        let mut final_wave_in_progress = ExecutionState::compact(&input, Params::default());
        final_wave_in_progress.ticks = EXECUTION_TICK_CAP;
        final_wave_in_progress.sim.wave = input.max_wave;
        final_wave_in_progress.sim.phase_wave = true;
        assert!(finish_summary(final_wave_in_progress, input.max_wave).capped);

        let mut early_called_wave_in_progress = ExecutionState::compact(&input, Params::default());
        early_called_wave_in_progress.ticks = EXECUTION_TICK_CAP;
        early_called_wave_in_progress.sim.wave = input.max_wave + 1;
        early_called_wave_in_progress.sim.phase_wave = true;
        assert!(finish_summary(early_called_wave_in_progress, input.max_wave).capped);
    }

    #[test]
    fn parameters_round_trip() {
        let params = Params::default();
        let json = serde_json::to_string(&params).unwrap();
        let restored: Params = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.towers.len(), 10);
        assert_eq!(restored.constants.growth, 1.12);
    }
}
