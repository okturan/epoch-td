const fs=require('fs');
const src=fs.readFileSync(__dirname+'/index.html','utf8').split('<script>')[1].split('// ---- browser ----')[0];
const boot=new Function(src+';return {mkState,update,startWave,place,upgrade,sell,towerAt,branch,merge,setMap,onPath,pickRelic,PATH:()=>PATH,WAVES,TOWERS,C}');

const INTENDED=[
[0,0,4,2],[0,0,8,2],[1,0,6,2],[2,0,10,2],[3,0,6,4],[4,'u',4,2],[4,'u',8,2],
[5,1,9,2],[6,0,2,6],[7,0,10,6],[7,1,3,2],[8,'u',6,2],[8,'u',10,2],[9,'u',2,6],[9,'u',10,6],[9,'u',6,4],
[10,2,5,4],[11,2,7,4],[12,'u',5,4],[12,'u',7,4],[13,'u',5,4],[13,'u',7,4],
[15,3,3,4],[15,3,9,4],[16,'u',3,4],[16,'u',9,4],[17,'u',9,2],[18,'u',3,2],
[19,'u',5,4],[19,'u',7,4],
[20,4,11,4],[21,4,2,4],[22,'u',11,4],[22,'u',2,4],[23,'u',9,2],[23,'u',3,2],
[24,'u',11,4],[24,'u',2,4],
[25,5,5,6],[25,5,7,6],[25,'u',5,6],[25,'u',7,6],[26,5,4,6],[26,5,8,6],[26,'u',3,4],[26,'u',9,4],
[27,'u',5,6],[27,'u',7,6],[28,'u',5,6],[28,'u',7,6],[28,'u',4,6],[28,'u',8,6],
[29,'s',4,2],[29,'s',8,2],[29,'s',6,2],[29,'s',10,2],[29,'s',6,4],[29,'s',2,6],[29,'s',10,6],
[29,'u',4,6],[29,'u',8,6],[29,'u',3,4],[29,'u',9,4],
[30,6,6,4],[30,6,7,2],[31,'u',6,4],[31,'u',7,2],[32,6,2,2],[32,'u',6,4],
[33,'u',7,2],[33,'u',2,2],[34,'u',6,4],
[35,6,11,2],[35,'u',11,2],[36,'u',11,2],[36,'u',7,2],[37,'u',2,2],[37,'u',2,2],
[38,7,3,6],[38,'u',3,6],[39,7,9,6],[39,'u',9,6],
[40,8,6,6],[40,'u',11,2],
[41,'u',4,6],[41,'u',8,6],[41,'b2',5,6],[41,'b2',7,6],[41,'u',11,4],[41,'u',2,4],
[42,6,12,0],[42,'u',12,0],
[43,3,10,4],[43,'u',10,4],[43,'u',12,0],
[44,6,7,0],[44,'u',7,0],[44,'u',10,4],
[45,9,0,0],[45,'u',0,0],[45,'u',0,0],[45,'u',10,4],
[46,6,11,0],[46,'u',11,0],[46,9,13,8],[46,'b2',6,4],
[47,'u',13,8],[47,'u',7,0],[47,'u',11,0],[47,'b2',7,2],
[48,'u',13,8],[48,'u',12,0],[48,'u',11,0],[48,'b1',3,4],[48,'b1',9,4],
[49,'u',7,0],[49,'b1',0,0],[49,'u',6,6],[49,'u',3,6]];

const BUDGET=[
[0,0,4,2],[0,0,8,2],[1,0,6,2],[3,0,6,4],[4,1,9,2],[6,1,3,2],[8,'u',4,2],[8,'u',8,2],
[10,2,5,4],[12,2,7,4],[13,'u',5,4],[14,'u',7,4],[15,3,3,4],[17,3,9,4],
[18,0,2,6],[19,0,10,6],[20,1,10,4],[20,'u',2,6],[20,'u',10,6],[21,'u',9,2],
[22,'u',3,4],[22,'u',9,4],[23,4,11,4],[24,1,5,6],[25,'u',10,4],[25,2,2,4],
[26,'u',5,4],[27,'u',7,4],[28,'u',3,4],[28,'u',9,4],[29,0,5,2],[29,0,7,2],
[30,'u',11,4],[31,'u',2,4],[31,'u',10,4],[33,1,11,2],[34,'u',6,2],[35,'u',5,4],
[36,'u',7,4],[37,0,2,2],[38,'u',4,2],[39,'u',8,2]];

const GREEDY=[
[0,0,6,4],[1,'u',6,4],[1,'u',6,4],[2,'u',6,4],
[5,1,6,2],[6,'u',6,2],[7,'u',6,2],[8,'u',6,2],
[10,2,5,4],[10,'u',5,4],[11,'u',5,4],[12,'u',5,4],
[15,2,4,4],[16,'u',4,4],[16,'u',4,4],[17,'u',4,4],
[18,'m',5,4],
[20,2,4,4],[20,'u',4,4],[20,'u',4,4],[20,'u',4,4],
[21,2,3,4],[21,'u',3,4],[21,'u',3,4],[21,'u',3,4],
[22,'m',4,4],
[30,6,2,2]];

function run(build){
  const G=boot(),S=G.mkState();
  const q=build.map(a=>({w:a[0],op:a[1],x:a[2],y:a[3]})),rows=[];
  const tryQ=()=>{
    while(q.length&&q[0].w<=S.wave-(S.phase==='wave'?1:0)){
      const a=q[0];let ok;
      if(a.op==='u'){const t=G.towerAt(S,a.x,a.y);ok=t&&G.upgrade(S,t)}
      else if(a.op==='b1'||a.op==='b2'){const t=G.towerAt(S,a.x,a.y);ok=t&&G.branch(S,t,a.op==='b2'?2:1)}
      else if(a.op==='m'){const t=G.towerAt(S,a.x,a.y);ok=t&&G.merge(S,t)}
      else if(a.op==='s'){const t=G.towerAt(S,a.x,a.y);if(t)G.sell(S,t);ok=true}
      else ok=G.place(S,a.op,a.x,a.y);
      if(!ok)break;
      q.shift();
    }
  };
  let stall=0;
  while(!S.over&&S.wave<50&&!stall){
    if(S.pick)G.pickRelic(S,0);
    tryQ();
    const w=S.wave,l0=S.lives,t0=S.t,g0=Math.round(S.gold);
    G.startWave(S);
    let n=0;
    while(S.phase==='wave'&&!S.over){
      G.update(S,1/30);S.ev.length=0;
      if(++n%90===0)tryQ();
      if(S.t-t0>240){stall=w+1;break}
    }
    rows.push({w:w+1,t:+(S.t-t0).toFixed(1),lost:l0-S.lives,g:g0,lives:S.lives});
  }
  return{S,rows,stall,spent:q.length};
}

const isLaser=a=>(a[2]===0&&a[3]===0||a[2]===13&&a[3]===8)&&(a[1]===9||a[1]==='u'||a[1]==='b1');
const NOLASER=INTENDED.filter(a=>!isLaser(a)).concat([
[45,6,1,0],[45,'u',1,0],[46,'u',1,0],[46,6,2,0],[47,'u',2,0],[47,'u',2,0],
[48,6,3,0],[48,'u',3,0],[49,'u',3,0],[49,'b2',1,0]]);
const V=process.argv.includes('-v');
const res={};
for(const[name,b]of[['INTENDED',INTENDED],['BUDGET',BUDGET],['GREEDY',GREEDY],['NOLASER',NOLASER]]){
  const r=run(b);res[name]=r;
  const last=r.rows[r.rows.length-1];
  console.log('== '+name+' == '+(r.S.won?'WON, lives '+r.S.lives:(r.stall?'STALLED wave '+r.stall:'DIED wave '+(last?last.w:'?')))+' | unspent actions: '+r.spent);
  if(V||name==='INTENDED')for(const x of r.rows)console.log('  w'+String(x.w).padStart(2)+' t='+String(x.t).padStart(6)+'s lost='+x.lost+' lives='+String(x.lives).padStart(2)+' gold@start='+x.g);
}
const I=res.INTENDED,B=res.BUDGET,Gr=res.GREEDY;
const bw=B.rows.length?B.rows[B.rows.length-1].w:0;
const gw=Gr.rows.length?Gr.rows[Gr.rows.length-1].w:0;
const slow=I.rows.filter(x=>x.t>90&&![10,20,30,40,50].includes(x.w));
console.log('--- CRITERIA ---');
console.log('1 Intended wins w/ >=5 lives : '+(I.S.won&&I.S.lives>=5?'PASS':'FAIL')+' (won='+I.S.won+' lives='+I.S.lives+')');
console.log('2 Budget dies wave 28-34     : '+(!B.S.won&&bw>=28&&bw<=34?'PASS':'FAIL')+' (death wave '+bw+')');
console.log('3 Non-boss waves <=90s       : '+(slow.length===0&&!I.stall?'PASS':'FAIL')+(slow.length?' slow: '+slow.map(x=>x.w+'('+x.t+'s)').join(' '):''));
console.log('  boss wave times: '+I.rows.filter(x=>[10,20,30,40,50].includes(x.w)).map(x=>x.w+':'+x.t+'s').join(' '));
console.log('4 Greedy dies on a rush wave : '+(!Gr.S.won&&[5,15,25,35,45].includes(gw)?'PASS':'FAIL')+' (death wave '+gw+')');
const NL=res.NOLASER,nw=NL.rows.length?NL.rows[NL.rows.length-1].w:0;
function bot(mi){
  const G=boot();G.setMap(mi);const S=G.mkState(),P=G.PATH();
  const cov=[];
  for(let y=0;y<9;y++)for(let x=0;x<14;x++){
    if(G.onPath(x,y))continue;
    let c=0;for(const p of P)if((p[0]-x)**2+(p[1]-y)**2<=6.25)c++;
    if(c)cov.push([c,x,y]);
  }
  cov.sort((a,b)=>b[0]-a[0]);
  const pref=[6,5,4,2,1,0];
  while(!S.over&&S.wave<50){
    if(S.pick)G.pickRelic(S,0);
    let g=0;
    while(g++<20){
      const ag=Math.min(9,Math.floor(S.wave/5)),ti=pref.find(i=>i<=ag&&S.gold>=G.TOWERS[i].c);
      if(ti===undefined)break;
      const sp=cov.find(([c,x,y])=>!G.towerAt(S,x,y));
      if(!sp||!G.place(S,ti,sp[1],sp[2]))break;
    }
    for(const t of S.towers)if(S.gold>500)G.upgrade(S,t);
    const t0=S.t;G.startWave(S);
    while(S.phase==='wave'&&!S.over){G.update(S,1/30);S.ev.length=0;if(S.t-t0>240)return S.wave}
  }
  return S.wave;
}
const botW=[0,1,2].map(bot);
console.log('6 All maps bot-playable >=18 : '+(botW.every(w=>w>=18)?'PASS':'FAIL')+' (bot reached '+botW.join(' / ')+')');
console.log('5 Laser required for Space   : '+(!NL.S.won?'PASS':'FAIL')+' (no-laser build '+(NL.S.won?'still wins':'dies wave '+nw)+')');
