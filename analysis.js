// EPOCH balance analysis — measures every tower / branch / fusion on a common
// yardstick using the real engine. Pins indestructible dummy enemies in range,
// runs the actual update() loop, and reads each tower's career damage (.dd).
//
//   node analysis.js          full report
//   node analysis.js fusions  fusion dominance + irradiate synergy only
const fs=require('fs');
const src=fs.readFileSync(__dirname+'/index.html','utf8').split('<script>')[1].split('// ---- browser ----')[0];
const boot=new Function(src+';return {mkState,update,place,upgrade,branch,merge,secretMerge,towerAt,mkEnemy,setMap,TOWERS,SECRETS,C}');

const T=20, DT=1/30;
// A dummy that never moves and never dies, so a tower fires at it for the whole run.
const dummy=(ar,cap,rg)=>({n:'Dummy',hp:1e13,spd:0,ar:ar||0,rg:rg||0,cap:cap||0,dash:0,split:0,bty:0,leak:0,boss:0,name:'Dummy'});
const SCEN={
  single:{ar:0,n:1},      // one bare target — pure single-target output
  armored:{ar:12,n:1},    // one armored target — rewards pierce / big per-hit
  swarm:{ar:0,n:6},       // six targets stacked on one cell — rewards splash
};

// Build the measured tower per spec, return {tw, gold}. Cell (6,0) sits beside the
// top path row (y=1); a fusion's second component goes on the adjacent cell (7,0).
function build(G,S,spec){
  const lvl=spec.lvl||3;
  const up=(t,to)=>{while(t.lvl<to)G.upgrade(S,t)};
  if(spec.fuse!=null){
    const r=G.SECRETS[spec.fuse];
    G.place(S,r.a,6,0);G.place(S,r.b,7,0);
    const a=G.towerAt(S,6,0),b=G.towerAt(S,7,0);
    up(a,Math.max(lvl,r.lv));up(b,Math.max(lvl,r.lv));
    G.secretMerge(S,a,b,spec.fuse);
    return {tw:G.towerAt(S,6,0),gold:a.inv};
  }
  G.place(S,spec.i,6,0);
  const t=G.towerAt(S,6,0);
  up(t,lvl);
  if(spec.br)G.branch(S,t,spec.br);
  return {tw:t,gold:t.inv};
}

function measure(spec,scen){
  const G=boot();G.setMap(0);
  const S=G.mkState();S.lives=1e9;S.gold=1e12;S.wave=50;S.phase='clear';S.autoT=1e9;
  const {tw,gold}=build(G,S,spec);
  for(let k=0;k<scen.n;k++)S.enemies.push(G.mkEnemy(dummy(scen.ar),6,0));
  tw.dd=0;
  for(let t=0;t<T;t+=DT){G.update(S,DT);S.ev.length=0;}
  return {dps:tw.dd/T,gold};
}

function row(name,spec){
  const s=measure(spec,SCEN.single),a=measure(spec,SCEN.armored),w=measure(spec,SCEN.swarm);
  const best=Math.max(s.dps,a.dps,w.dps);
  return {name,gold:s.gold,single:s.dps,armored:a.dps,swarm:w.dps,best,perGold:best/s.gold*1000,perCell:best};
}

const fmt=v=>v>=1e4?(v/1e3).toFixed(1)+'k':v.toFixed(0);
function table(rows,title){
  console.log('\n=== '+title+' ===');
  console.log(['tower'.padEnd(22),'gold'.padStart(6),'single'.padStart(8),'armor'.padStart(8),'swarm'.padStart(8),'DPS/1kg'.padStart(8),'DPS/cell'.padStart(9)].join(' '));
  for(const r of rows)console.log([r.name.padEnd(22),(''+Math.round(r.gold)).padStart(6),fmt(r.single).padStart(8),fmt(r.armored).padStart(8),fmt(r.swarm).padStart(8),r.perGold.toFixed(1).padStart(8),fmt(r.perCell).padStart(9)].join(' '));
}

const NAMES=(i)=>['Rock','Catapult','Ballista','Brazier','Cannon','Gatling','Missile','Reactor','DroneHub','Laser'][i];
const BR={0:['Boulders','SkipShot'],1:['SiegeLoad','WideNet'],2:['Overdraw','TwinBolts'],3:['Inferno','Pyre'],4:['DoomShell','Concussive'],5:['HotBarrels','APRounds'],6:['Warheads','MIRV'],7:['Meltdown','WideField'],8:['Overclock','Stasis'],9:['Overcharge','RapidFocus']};

if(process.argv[2]!=='fusions'){
  // Base towers L3
  const base=[];
  for(let i=0;i<10;i++)base.push(row(NAMES(i)+' L3',{i,lvl:3}));
  table(base.sort((a,b)=>b.perGold-a.perGold),'BASE TOWERS (L3), sorted by gold efficiency');

  // Branches L3
  const br=[];
  for(let i=0;i<10;i++)for(const b of [1,2])br.push(row(NAMES(i)+':'+BR[i][b-1],{i,br:b,lvl:3}));
  table(br.sort((a,b)=>b.perGold-a.perGold),'BRANCHES (L3), sorted by gold efficiency');
}

// Fusions L3 (non-Ascendant)
const G0=boot();
const fus=[];
for(let fi=0;fi<5;fi++)fus.push(row('✦'+G0.SECRETS[fi].n,{fuse:fi,lvl:3}));
table(fus.slice().sort((a,b)=>b.perCell-a.perCell),'FUSIONS (L3), sorted by absolute DPS/cell');
table(fus.slice().sort((a,b)=>b.perGold-a.perGold),'FUSIONS (L3), sorted by gold efficiency');

// Fusion dominance: compare each fusion to the best branch of its damage-base type.
console.log('\n=== FUSION vs BEST BASE-TYPE BRANCH (is the fusion a runaway?) ===');
for(let fi=0;fi<5;fi++){
  const r=G0.SECRETS[fi],baseI=r.base;
  const b1=row('x',{i:baseI,br:1,lvl:3}),b2=row('x',{i:baseI,br:2,lvl:3});
  const bestBranch=Math.max(b1.perCell,b2.perCell),bestGold=Math.max(b1.perGold,b2.perGold);
  const f=row('x',{fuse:fi,lvl:3});
  const cellX=(f.perCell/bestBranch).toFixed(2),goldX=(f.perGold/bestGold).toFixed(2);
  console.log('✦'+r.n.padEnd(16)+' vs '+NAMES(baseI)+' branches: DPS/cell '+cellX+'×  DPS/gold '+goldX+'×'+(cellX>1.6?'   <-- HIGH cell power':'')+(goldX>1.4?'  <-- HIGH gold value':''));
}

// Irradiate synergy: the real reason Warhead Silo / Sun Lance multiply a board.
// Reference board of 6 Ballistas hammering one fat target. Measure total board
// damage WITHOUT any irradiator, then with an irradiate debuff kept active.
console.log('\n=== IRRADIATE SYNERGY (board-wide multiplier from one irradiator) ===');
function boardSynergy(irr){
  const G=boot();G.setMap(0);
  const S=G.mkState();S.lives=1e9;S.gold=1e12;S.wave=50;S.phase='clear';S.autoT=1e9;
  const cells=[[3,0],[5,0],[7,0],[9,0],[4,2],[8,2]];
  for(const [x,y] of cells){G.place(S,2,x,y);const t=G.towerAt(S,x,y);while(t.lvl<3)G.upgrade(S,t);}
  const e=G.mkEnemy(dummy(0),6,0);S.enemies.push(e);
  let dealt=0;
  for(let t=0;t<T;t+=DT){if(irr)e.irr=1;G.update(S,DT);S.ev.length=0;}
  for(const tw of S.towers)dealt+=tw.dd||0;
  return dealt/T;
}
const off=boardSynergy(false),on=boardSynergy(true);
console.log('6-Ballista board DPS on a target:  plain '+fmt(off)+'   irradiated '+fmt(on)+'   = '+((on/off-1)*100).toFixed(0)+'% more, board-wide');
console.log('(A Warhead Silo applies this to every enemy it hits, on top of its own damage.)');

// ---- BALANCE GUARD ----------------------------------------------------------
// A fusion should be a worthwhile, distinct pick: at least as good as its base
// tower, never a runaway over that base's best branch, and (for the two premium
// irradiators) a debuff sidegrade rather than a strict damage upgrade.
console.log('\n=== BALANCE GUARD ===');
let fail=0;
const cellOf=spec=>row('x',spec).perCell;
for(let fi=0;fi<5;fi++){
  const r=G0.SECRETS[fi],bi=r.base;
  const fusCell=cellOf({fuse:fi,lvl:3});
  const baseCell=cellOf({i:bi,lvl:3});
  const branchCell=Math.max(cellOf({i:bi,br:1,lvl:3}),cellOf({i:bi,br:2,lvl:3}));
  const overBase=fusCell/baseCell, overBranch=fusCell/branchCell;
  const ok=overBase>=0.95 && overBranch<=1.85;
  if(!ok)fail++;
  console.log((ok?'  ok  ':'  XX  ')+('✦'+r.n).padEnd(16)+' '+overBase.toFixed(2)+'× its base, '+overBranch.toFixed(2)+'× best branch  (want >=0.95× base, <=1.85× branch)');
}
// The friend's report, encoded: Warhead Silo must NOT out-damage a branched missile single-target.
const wsSingle=measure({fuse:2,lvl:3},SCEN.single).dps;
const brSingle=Math.max(measure({i:6,br:1,lvl:3},SCEN.single).dps,measure({i:6,br:2,lvl:3},SCEN.single).dps);
const wsOk=wsSingle<brSingle;
if(!wsOk)fail++;
console.log((wsOk?'  ok  ':'  XX  ')+'Warhead Silo single DPS '+wsSingle.toFixed(0)+' < branched missile '+brSingle.toFixed(0)+' (its draw is the debuff, not raw damage)');
console.log(fail?'\nBALANCE GUARD: '+fail+' FAIL':'\nBALANCE GUARD: PASS');
process.exit(fail?1:0);
