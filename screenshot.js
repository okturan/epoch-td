let chromium;
try{chromium=require('playwright').chromium}
catch{chromium=require(process.env.PLAYWRIGHT_PATH||'/Users/okan/.npm/_npx/e41f203b7505f1fb/node_modules/playwright').chromium}
const fs=require('fs');
const INTENDED=eval(fs.readFileSync(__dirname+'/sim.js','utf8').match(/const INTENDED=(\[[\s\S]*?\]);/)[1]);
const shot=(pg,name)=>pg.screenshot({path:__dirname+'/media/'+name+'.png'});
(async()=>{
  fs.mkdirSync(__dirname+'/media',{recursive:true});
  const br=await chromium.launch({channel:'chrome'}).catch(()=>chromium.launch());
  const pg=await br.newPage({viewport:{width:1320,height:820},deviceScaleFactor:2});
  await pg.goto('file://'+__dirname+'/index.html');
  await pg.waitForTimeout(500);

  // 1. Start / map select
  await shot(pg,'start');

  await pg.evaluate((build)=>{
    window.__reset=()=>{setMap(0);S=mkState();selT=null;selE=null;
      document.getElementById('mapsov').style.display='none';
      document.getElementById('end').style.display='none';};
    const q0=build.map(a=>({w:a[0],op:a[1],x:a[2],y:a[3]}));
    window.__load=()=>{const q=q0.slice();window.__q=q;
      window.__tryQ=()=>{while(q.length&&q[0].w<=S.wave-(S.phase==='wave'?1:0)){const a=q[0];let r;
        if(a.op==='u'){const t=towerAt(S,a.x,a.y);r=t&&upgrade(S,t)}
        else if(a.op==='b1'||a.op==='b2'){const t=towerAt(S,a.x,a.y);r=t&&branch(S,t,a.op==='b2'?2:1)}
        else if(a.op==='m'){const t=towerAt(S,a.x,a.y);r=t&&merge(S,t)}
        else if(a.op==='s'){const t=towerAt(S,a.x,a.y);if(t)sell(S,t);r=true}
        else r=place(S,a.op,a.x,a.y);
        if(!r)break;q.shift()}};};
    window.__ff=w=>{let g=0;while(!S.over&&(S.wave<w||S.phase==='wave'||S.pick)&&g++<4e6){if(S.pick)pickRelic(S,0);__tryQ();if(S.phase==='clear'&&S.wave<w)startWave(S);update(S,1/30);S.ev.length=0}};
    window.__mid=w=>{__ff(w-1);__tryQ();startWave(S);let g=0;
      while(S.phase==='wave'&&!S.over&&g++<4e6){update(S,1/30);S.ev.length=0;
        if((!S.spawns.length||S.spawns[0].sn>=S.spawns[0].R.n*.5)&&S.enemies.length>4)break}};
  },INTENDED);

  // 2. Gameplay mid-run — Modern age
  await pg.evaluate(()=>{__reset();__load();__mid(34);selT=null;selE=null;syncPanel();});
  await pg.waitForTimeout(200);
  await shot(pg,'gameplay');

  // 3. Tower card — a branched tower selected
  await pg.evaluate(()=>{selE=null;selT=S.towers.find(t=>t.br)||S.towers.find(t=>t.i===2)||S.towers[0];syncPanel();});
  await pg.waitForTimeout(150);
  await shot(pg,'card');

  // 4. Space age with laser beams firing
  await pg.evaluate(()=>{__reset();__load();__mid(47);selT=null;selE=null;syncPanel();});
  await pg.waitForTimeout(200);
  await shot(pg,'space');

  // 5. Doctrine draft overlay
  await pg.evaluate(()=>{S.pick=[0,4,7]});
  await pg.waitForTimeout(150);
  await shot(pg,'doctrine');

  // 6. Victory scorecard
  await pg.evaluate(()=>{__reset();__load();__ff(50);});
  await pg.waitForTimeout(250);
  await shot(pg,'scorecard');

  await br.close();
  console.log('screenshots written to media/');
})().catch(e=>{console.error(e);process.exit(1)});
