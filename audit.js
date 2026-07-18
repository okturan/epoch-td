// UI text audit — render every dynamic panel/overlay state in a real browser and
// scan the rendered text for placeholder artifacts (undefined / NaN / [object] / null).
let chromium;
try{chromium=require('playwright').chromium}
catch{chromium=require(process.env.PLAYWRIGHT_PATH||'/Users/okan/.npm/_npx/e41f203b7505f1fb/node_modules/playwright').chromium}
const BAD=/\bundefined\b|\bNaN\b|\[object |>\s*null\s*</i;
let checks=0, fails=0;
const scan=(label,text)=>{checks++;const m=text.match(BAD);if(m){fails++;console.log('  BAD  '+label+'  -->  "'+m[0].trim()+'"  in: '+text.replace(/\s+/g,' ').slice(0,120));}};

(async()=>{
  const br=await chromium.launch({channel:'chrome'}).catch(()=>chromium.launch());
  const pg=await br.newPage({viewport:{width:1320,height:900}});
  const perr=[];pg.on('pageerror',e=>perr.push(e.message));
  await pg.goto('file://'+__dirname+'/index.html');
  await pg.waitForTimeout(300);
  await pg.click('#maps button:nth-child(1)');
  await pg.evaluate(()=>{S.wave=50;S.gold=1e9;});

  const panelText=()=>pg.evaluate(()=>document.getElementById('panel').innerText);

  // every base tower at L0 and L3, plus both branches at L3
  const names=['Rock','Catapult','Ballista','Brazier','Cannon','Gatling','Missile','Reactor','DroneHub','Laser'];
  for(let i=0;i<10;i++){
    await pg.evaluate(i=>{S.towers.length=0;place(S,i,4,2);selE=null;selT=towerAt(S,4,2);syncPanel();},i);
    scan('tower '+names[i]+' L0', await panelText());
    await pg.evaluate(i=>{const t=towerAt(S,4,2);upgrade(S,t);upgrade(S,t);upgrade(S,t);syncPanel();},i);
    scan('tower '+names[i]+' L3', await panelText());
    for(const b of [1,2]){
      await pg.evaluate(([i,b])=>{S.towers.length=0;place(S,i,4,2);const t=towerAt(S,4,2);upgrade(S,t);upgrade(S,t);upgrade(S,t);branch(S,t,b);selT=t;syncPanel();},[i,b]);
      scan('tower '+names[i]+' branch'+b, await panelText());
    }
  }

  // every fusion, and an L4 merge, and an L5 merge
  for(let f=0;f<5;f++){
    await pg.evaluate(f=>{S.towers.length=0;const r=SECRETS[f];
      place(S,r.a,4,2);place(S,r.b,5,2);
      const a=towerAt(S,4,2),b=towerAt(S,5,2);
      for(const t of [a,b])while(t.lvl<Math.max(3,r.lv))upgrade(S,t);
      secretMerge(S,a,b,f);selE=null;selT=towerAt(S,4,2);syncPanel();},f);
    scan('fusion '+f, await panelText());
  }
  await pg.evaluate(()=>{S.towers.length=0;place(S,6,4,2);place(S,6,5,2);
    for(const c of [[4,2],[5,2]]){const t=towerAt(S,c[0],c[1]);upgrade(S,t);upgrade(S,t);upgrade(S,t);branch(S,t,1);}
    merge(S,towerAt(S,4,2),towerAt(S,5,2));selT=towerAt(S,4,2);syncPanel();});
  scan('merged L4', await panelText());

  // every enemy archetype inspected (walk representative waves that spawn each type)
  const waves=[1,6,11,16,21,26,31,36,41,46,10];
  for(const w of waves){
    await pg.evaluate(w=>{S.wave=w-1;S.phase='clear';S.autoT=1e9;S.enemies.length=0;startWave(S);
      let g=0;while(!S.enemies.length&&g++<2000){update(S,1/30);S.ev.length=0;}
      selT=null;selE=S.enemies[0];syncPanel();},w);
    scan('enemy wave '+w, await panelText());
  }

  // all 12 doctrines in the draft overlay
  await pg.evaluate(()=>{S.pick=[0,1,2,3,4,5,6,7,8,9,10,11];});
  await pg.waitForTimeout(60);
  scan('doctrine overlay (all 12)', await pg.evaluate(()=>document.getElementById('relics').innerText));
  await pg.evaluate(()=>{S.pick=null;});

  // wave preview + threat hint for all 50 waves
  for(let w=0;w<50;w++){
    const t=await pg.evaluate(w=>{S.wave=w;S.phase='clear';S.over=0;hud();return document.getElementById('nextw').innerText;},w);
    scan('preview wave '+(w+1), t);
  }

  // HUD stats and empty panel
  scan('HUD stats', await pg.evaluate(()=>{hud();return document.getElementById('stats').innerText;}));
  await pg.evaluate(()=>{selT=null;selE=null;syncPanel();});
  scan('empty panel', await panelText());

  // scorecard (victory) and defeat
  await pg.evaluate(()=>{S=mkState();S.won=1;S.over=1;S.wave=50;S.lives=14;S.kills=1200;S.bossK=5;S.goldE=9000;S.t=1500;
    S.ty={6:900000,9:120000,3:80000,2:50000,pw:40000};S.rl={bounty1:1,rng8:1};hud();});
  await pg.waitForTimeout(60);
  scan('victory scorecard', await pg.evaluate(()=>document.getElementById('endd').innerText));
  await pg.evaluate(()=>{S=mkState();S.won=0;S.over=1;S.wave=27;S.ty={};hud();});
  await pg.waitForTimeout(60);
  scan('defeat scorecard', await pg.evaluate(()=>document.getElementById('endd').innerText));

  console.log('\naudited '+checks+' rendered states; '+(fails?fails+' with placeholder text':'0 placeholders')+'; page errors: '+(perr.length||'none'));
  if(perr.length)console.log(perr.join('\n'));
  await br.close();
  process.exit(fails||perr.length?1:0);
})().catch(e=>{console.error(e);process.exit(1)});
