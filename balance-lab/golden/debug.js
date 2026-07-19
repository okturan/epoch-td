#!/usr/bin/env node
'use strict';
const fs=require('fs'),path=require('path');
const root=path.join(__dirname,'..','..');
const html=fs.readFileSync(path.join(root,'index.html'),'utf8');
const core=html.split('<script>')[1].split('// ---- browser ----')[0];
const G=new Function(core+';return {mkState,update,startWave,place,upgrade,sell,towerAt,branch,merge,secretPair,secretMerge,pickRelic,setMap,TOWERS,WAVES,C}')();
const input=JSON.parse(fs.readFileSync(process.argv[2],'utf8')),target=Number(process.argv[3]||7);
function patchObject(target,patch){if(!patch)return;for(const[k,v]of Object.entries(patch)){if(v&&typeof v==='object'&&!Array.isArray(v)){if(!target[k]||typeof target[k]!=='object')target[k]={};patchObject(target[k],v)}else target[k]=v}}
G.setMap(input.map||0);patchObject(G.C,input.params&&input.params.constants);if(input.params&&input.params.towers)for(const[i,p]of Object.entries(input.params.towers))patchObject(G.TOWERS[Number(i)],p);if(input.params&&input.params.constants)for(let i=0;i<G.WAVES.length;i++)G.WAVES[i]=undefined;
const S=G.mkState(),pending=input.actions.slice();
function action(a){const t=G.towerAt(S,a.x,a.y);if(a.op==='place')return G.place(S,a.tower,a.x,a.y);if(a.op==='upgrade')return t&&G.upgrade(S,t);if(a.op==='branch')return t&&G.branch(S,t,a.branch);if(a.op==='sell'){if(t)G.sell(S,t);return!!t}if(a.op==='merge'){const p=a.with&&G.towerAt(S,a.with.x,a.with.y);return t&&G.merge(S,t,p)}if(a.op==='fuse'){const p=a.with&&G.towerAt(S,a.with.x,a.with.y),ri=t&&p&&G.secretPair(t,p);return ri!=null&&G.secretMerge(S,t,p,ri)}return false}
function prep(){if(S.pick)G.pickRelic(S,(Number(input.seed||0)+S.wave)%S.pick.length);while(pending.length&&pending[0].wave<=S.wave)action(pending.shift());G.startWave(S)}
while(!S.over&&S.wave<target-1){prep();while(S.phase==='wave'&&!S.over){G.update(S,1/30);S.ev.length=0}}
if(S.over)process.stdout.write('[]');else{prep();const out=[];let tick=0;while(S.phase==='wave'&&!S.over){G.update(S,1/30);S.ev.length=0;tick++;out.push({tick,gold:S.gold,lives:S.lives,kills:S.kills,enemies:S.enemies.map(e=>[e.d,e.hp,e.x,e.y]),projectiles:S.projs.map(p=>[p.x,p.y,S.enemies.indexOf(p.tg),S.towers.indexOf(p.tw)]),towerDamage:S.towers.map(t=>t.dd||0)})}process.stdout.write(JSON.stringify(out))}
