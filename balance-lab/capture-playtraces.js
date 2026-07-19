let chromium;
try { chromium = require('playwright').chromium; }
catch { chromium = require(process.env.PLAYWRIGHT_PATH || '/Users/okan/.npm/_npx/e41f203b7505f1fb/node_modules/playwright').chromium; }
const fs = require('fs');
const path = require('path');

const root = path.resolve(__dirname, '..');
const sim = fs.readFileSync(path.join(root, 'sim.js'), 'utf8');
const readBuild = name => eval(sim.match(new RegExp(`const ${name}=(\\[[\\s\\S]*?\\]);`))[1]);
const sessions = [
  { name: 'intended-deliberate', map: 0, build: readBuild('INTENDED'), reaction: 18, earlyEvery: 5 },
  { name: 'budget-reactive', map: 1, build: readBuild('BUDGET'), reaction: 9, earlyEvery: 3 },
  { name: 'greedy-focused', map: 2, build: readBuild('GREEDY'), reaction: 27, earlyEvery: 4 },
];

(async () => {
  const browser = await chromium.launch({ channel: 'chrome' }).catch(() => chromium.launch());
  const traces = [];
  try {
    for (const session of sessions) {
      const page = await browser.newPage({ viewport: { width: 1060, height: 900 } });
      const errors = [];
      page.on('pageerror', error => errors.push(error.message));
      await page.goto(`file://${path.join(root, 'index.html')}`);
      await page.click(`#maps button:nth-child(${session.map + 1})`);
      const trace = await page.evaluate(({ build, reaction, earlyEvery }) => {
        const queue = build.map(action => ({ wave: action[0], op: action[1], x: action[2], y: action[3] }));
        const apply = action => {
          const tower = towerAt(S, action.x, action.y);
          if (action.op === 'u') return tower && upgrade(S, tower);
          if (action.op === 'b1' || action.op === 'b2') return tower && branch(S, tower, action.op === 'b2' ? 2 : 1);
          if (action.op === 'm') return tower && merge(S, tower);
          if (action.op === 's') { if (tower) sell(S, tower); return true; }
          return place(S, action.op, action.x, action.y);
        };
        const applyReady = () => {
          while (queue.length && queue[0].wave <= S.wave - (S.phase === 'wave' ? 1 : 0)) {
            if (!apply(queue[0])) break;
            queue.shift();
          }
        };
        let guard = 0;
        while (!S.over && (S.wave < 50 || S.phase === 'wave') && guard++ < 5e6) {
          if (S.pick) pickRelic(S, (S.wave / 10) % Math.min(3, S.pick.length) | 0);
          applyReady();
          if (S.phase === 'clear') startWave(S);
          const tick = Math.max(0, Math.round((S.t - S.wst) * 30));
          if (tick === reaction) {
            const powerIndex = age(S) >= 6 ? S.wave % 3 : 0;
            power(S, powerIndex, 7, 4);
          }
          if (S.phase === 'wave' && S.wave % earlyEvery === 0 && tick === reaction + 12) startWave(S);
          if (tick > 0 && tick % 90 === reaction % 90) applyReady();
          update(S, 1 / 30);
          S.ev.length = 0;
        }
        return JSON.parse(exportEpochPlaytrace());
      }, session);
      if (errors.length) throw new Error(`${session.name}: ${errors.join('; ')}`);
      trace.label = session.name;
      traces.push(trace);
      console.log(`${session.name}: wave ${trace.result.wave}, ${trace.actions.length} actions, ${trace.durationSeconds.toFixed(1)}s`);
      await page.close();
    }
  } finally {
    await browser.close();
  }
  const out = path.join(__dirname, 'out');
  fs.mkdirSync(out, { recursive: true });
  fs.writeFileSync(path.join(out, 'playtraces.json'), `${JSON.stringify(traces, null, 2)}\n`);
  console.log('balance-lab/out/playtraces.json');
})().catch(error => { console.error(error); process.exitCode = 1; });
