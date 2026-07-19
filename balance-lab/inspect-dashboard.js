let chromium;
try { chromium = require('playwright').chromium; }
catch { chromium = require(process.env.PLAYWRIGHT_PATH || '/Users/okan/.npm/_npx/e41f203b7505f1fb/node_modules/playwright').chromium; }
const path = require('path');

const out = path.join(__dirname, 'out');
(async () => {
  const browser = await chromium.launch({ channel: 'chrome' }).catch(() => chromium.launch());
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
  await page.goto(`file://${path.join(out, 'index.html')}`);
  const summary = await page.evaluate(() => ({
    kind: document.querySelector('#attacker').hidden ? 'defender' : 'attack',
    cells: document.querySelectorAll('#heat .cell').length,
    rows: document.querySelectorAll('#table tr').length - 1,
    comparisonRows: document.querySelectorAll('#defender table tr').length - 1,
    games: document.querySelector('#cards .card')?.textContent.trim(),
    gate: document.querySelector('#cards .card:last-child')?.textContent.trim(),
    replay: document.querySelector('#table a')?.href,
  }));
  if (summary.kind === 'defender') {
    if (summary.comparisonRows !== 4 || !summary.gate.startsWith('PASS')) throw new Error(`invalid defender dashboard: ${JSON.stringify(summary)}`);
    await page.screenshot({ path: path.join(out, 'defender-dashboard.png'), fullPage: true });
    await browser.close();
    if (errors.length) throw new Error(errors.join('; '));
    console.log(JSON.stringify({ dashboard: { ...summary, replay: false } }, null, 2));
    return;
  }
  if (!summary.cells || summary.cells !== summary.rows || !summary.replay) throw new Error(`invalid attacker dashboard: ${JSON.stringify(summary)}`);
  await page.screenshot({ path: path.join(out, 'attacker-dashboard.png'), fullPage: true });
  await page.goto(summary.replay);
  await page.evaluate(() => { speed = 100; });
  await page.waitForFunction(() => S.over || S.wave >= Math.min(10, labReplay.maxWave), null, { timeout: 10000 });
  await page.evaluate(() => { speed = 0; floats.length = 0; });
  await page.waitForTimeout(100);
  const replay = await page.evaluate(() => ({ wave: S.wave, towers: S.towers.length, queued: labQueue.length, maxWave: labReplay.maxWave }));
  if (!replay.towers || replay.wave < Math.min(10, replay.maxWave)) throw new Error(`replay did not advance: ${JSON.stringify(replay)}`);
  await page.screenshot({ path: path.join(out, 'replay.png') });
  await browser.close();
  if (errors.length) throw new Error(errors.join('; '));
  console.log(JSON.stringify({ dashboard: { ...summary, replay: true }, replay }, null, 2));
})().catch(error => { console.error(error); process.exitCode = 1; });
