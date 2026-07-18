const fs=require('fs');
const src=fs.readFileSync(__dirname+'/index.html','utf8').split('<script>')[1].split('// ---- browser ----')[0];
const boot=new Function(src+';return {mkState,update,startWave,place,upgrade,sell,towerAt,branch,merge,pickRelic,power,setMap,onPath,PATH:()=>PATH,TOWERS,RELICS,C}');
const INTENDED=eval(fs.readFileSync(__dirname+'/sim.js','utf8').match(/const INTENDED=(\[[\s\S]*?\]);/)[1]);
const BR=+(process.argv[2]||2); // 1=Warheads 2=MIRV
const G=boot(),S=G.mkState();S.endless=1;
const P=G.PATH();
const cov=[];
for(let y=0;y<9;y++)for(let x=0;x<14;x++){
  if(G.onPath(x,y))continue;
  let c=0;for(const p of P)if((p[0]-x)**2+(p[1]-y)**2<=9)c++;
  if(c)cov.push([c,x,y]);
}
cov.sort((a,b)=>b[0]-a[0]);
const PREF=['cd07','bounty1','rng8','slow5','pv13','sp12','up50','sell85','irr1','life3','kb15','burn7'];
const q=INTENDED.map(a=>({w:a[0],op:a[1],x:a[2],y:a[3]}));
const tryQ=()=>{while(q.length&&q[0].w<=S.wave-(S.phase==='wave'?1:0)){const a=q[0];let ok;
  if(a.op==='u'){const t=G.towerAt(S,a.x,a.y);ok=t&&G.upgrade(S,t)}
  else if(a.op==='b1'||a.op==='b2'){const t=G.towerAt(S,a.x,a.y);ok=t&&G.branch(S,t,a.op==='b2'?2:1)}
  else if(a.op==='m'){const t=G.towerAt(S,a.x,a.y);ok=t&&G.merge(S,t)}
  else if(a.op==='s'){const t=G.towerAt(S,a.x,a.y);if(t)G.sell(S,t);ok=true}
  else ok=G.place(S,a.op,a.x,a.y);
  if(!ok)break;q.shift()}};
let nMis=0;
function spend(){
  let act=true,g=0;
  while(act&&g++<300){
    act=false;
    for(const t of S.towers){
      if(t.i===6){
        if(t.lvl<3&&G.upgrade(S,t))act=true;
        else if(t.lvl>=3&&!t.br&&G.branch(S,t,BR))act=true;
      }
      if(t.i===7&&t.lvl<2&&G.upgrade(S,t))act=true;
    }
    const sp=cov.find(([c,x,y])=>!G.towerAt(S,x,y));
    if(sp){
      const ti=(nMis%8===7)?8:(nMis%5===4)?7:6;
      if(G.place(S,ti,sp[1],sp[2])){nMis++;act=true}
    }else{
      if(S.gold>=700){
        const junk=S.towers.find(t=>[0,1,2,3].includes(t.i));
        if(junk){G.sell(S,junk);act=true}
      }
      if(!act)outer:for(const t of S.towers)if(t.i===6&&t.lvl>=3&&t.lvl<5)for(const p of S.towers)
        if(p!==t&&p.i===6&&p.lvl===t.lvl&&p.br===t.br&&Math.max(Math.abs(p.x-t.x),Math.abs(p.y-t.y))===1){
          if(G.merge(S,t,p)){act=true;break outer}}
    }
  }
}
function powers(){
  if(S.pw[0]<=0&&S.enemies.length>=6){
    let best=null,bc=0;
    for(const e of S.enemies){let c=0;for(const o of S.enemies)if((o.x-e.x)**2+(o.y-e.y)**2<=2.25)c++;if(c>bc){bc=c;best=e}}
    if(bc>=6)G.power(S,0,best.x,best.y);
  }
  if(S.enemies.length>=20&&S.pw[1]<=0)G.power(S,1);
  if(S.enemies.length>=10&&S.pw[2]<=0)G.power(S,2);
}
function pickBest(){
  const ks=S.pick.map(i=>G.RELICS[i].k);
  let bi=0,br=99;
  ks.forEach((k,i)=>{const r=PREF.indexOf(k);if(r>=0&&r<br){br=r;bi=i}});
  G.pickRelic(S,bi);
}
let stall=0;
while(!S.over&&S.wave<100&&!stall){
  if(S.pick)pickBest();
  if(S.wave>=50){q.length=0;
    if(!S.junkSold){S.junkSold=1;for(const t of S.towers.slice())if([0,1,2,3].includes(t.i))G.sell(S,t)}
  }
  tryQ();
  if(!q.length)spend();
  const w=S.wave,t0=S.t,l0=S.lives;
  G.startWave(S);
  let n=0;
  while(S.phase==='wave'&&!S.over){
    G.update(S,1/30);S.ev.length=0;
    if(S.pw[0]<=0||S.pw[1]<=0||S.pw[2]<=0)powers();
    if(++n%90===0){if(S.wave>=50)q.length=0;tryQ();if(!q.length)spend()}
    if(S.t-t0>240){stall=w+1;break}
  }
  if(w>=49)console.log('w'+(w+1)+' t='+(S.t-t0).toFixed(0)+'s lost='+(l0-S.lives)+' lives='+S.lives+' gold='+Math.round(S.gold)+' towers='+S.towers.length+' L5:'+S.towers.filter(t=>t.lvl===5).length+' L4:'+S.towers.filter(t=>t.lvl===4).length);
}
console.log((BR===1?'WARHEADS':'MIRV')+' endless run: '+(stall?'STALLED at '+stall:'died, reached wave '+S.wave)+' | doctrines: '+Object.keys(S.rl).join(','));
