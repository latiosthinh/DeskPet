// GET /stats?token=...  visits, downloads and update checks per pet and per day.
// The token is the STATS_TOKEN secret (`npx wrangler pages secret put STATS_TOKEN`).
import { PETS, KINDS } from '../lib/shared.js';

const esc = (s) => String(s).replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);

async function sameText(a, b) {
  // Compare digests so the comparison takes the same time whatever the input.
  const enc = new TextEncoder();
  const [x, y] = await Promise.all([a, b].map((s) => crypto.subtle.digest('SHA-256', enc.encode(s))));
  const ua = new Uint8Array(x), ub = new Uint8Array(y);
  let diff = 0;
  for (let i = 0; i < ua.length; i++) diff |= ua[i] ^ ub[i];
  return diff === 0;
}

function page(body) {
  return new Response(
    `<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="robots" content="noindex">
<title>DeskPet stats</title>
<style>
:root{color-scheme:light dark;--bg:#fbf3e4;--ink:#33293d;--line:#d9cbb4;--soft:#f3e7d2}
@media (prefers-color-scheme:dark){:root{--bg:#1e1a24;--ink:#efe4d2;--line:#4a4055;--soft:#2a2432}}
body{margin:0;padding:24px 16px;background:var(--bg);color:var(--ink);font:15px/1.4 system-ui,sans-serif}
main{max-width:960px;margin:0 auto}
h1{font-size:24px;margin:0 0 4px}h2{font-size:18px;margin:28px 0 8px}
p{margin:0 0 12px;opacity:.75}
.wrap{overflow-x:auto}
table{border-collapse:collapse;width:100%;font-variant-numeric:tabular-nums}
th,td{padding:6px 10px;border-bottom:1px solid var(--line);text-align:right;white-space:nowrap}
th:first-child,td:first-child,th.l,td.l{text-align:left}
thead th{background:var(--soft);position:sticky;top:0}
tfoot td{font-weight:700}
</style></head><body><main>${body}</main></body></html>`,
    { headers: { 'content-type': 'text/html; charset=utf-8', 'cache-control': 'no-store', 'x-robots-tag': 'noindex' } },
  );
}

function table(head, rows, foot) {
  const tr = (cells, tag = 'td') => `<tr>${cells.map((c) => `<${tag}>${esc(c)}</${tag}>`).join('')}</tr>`;
  return `<div class="wrap"><table><thead>${tr(head, 'th')}</thead><tbody>${rows.map((r) => tr(r)).join('')}</tbody>${
    foot ? `<tfoot>${tr(foot)}</tfoot>` : ''
  }</table></div>`;
}

export async function onRequestGet({ request, env }) {
  const url = new URL(request.url);
  const token = url.searchParams.get('token') || '';
  if (!env.STATS_TOKEN || !token || !(await sameText(token, env.STATS_TOKEN))) {
    return new Response('Not found', { status: 404 });
  }
  if (!env.DB) return page('<h1>DeskPet stats</h1><p>The DB binding is missing.</p>');

  const days = Math.min(Math.max(parseInt(url.searchParams.get('days') || '30', 10) || 30, 1), 3650);
  const since = new Date(Date.now() - (days - 1) * 86400000).toISOString().slice(0, 10);

  const [totals, daily, detail] = await Promise.all([
    env.DB.prepare('SELECT pet_id, kind, SUM(n) AS n FROM counts GROUP BY pet_id, kind').all(),
    env.DB.prepare('SELECT day, kind, SUM(n) AS n FROM counts WHERE day >= ?1 GROUP BY day, kind ORDER BY day DESC').bind(since).all(),
    env.DB.prepare('SELECT day, pet_id, kind, n FROM counts WHERE day >= ?1 ORDER BY day DESC, pet_id').bind(since).all(),
  ]);

  // Per pet, all time. Known pets first (in pets.json order), then any retired ids still in the table.
  const perPet = new Map([...PETS.keys()].map((id) => [id, { visit: 0, download: 0, update: 0 }]));
  for (const r of totals.results) {
    if (!perPet.has(r.pet_id)) perPet.set(r.pet_id, { visit: 0, download: 0, update: 0 });
    perPet.get(r.pet_id)[r.kind] = r.n;
  }
  const sum = { visit: 0, download: 0, update: 0 };
  const petRows = [...perPet].map(([id, c]) => {
    KINDS.forEach((k) => (sum[k] += c[k] || 0));
    return [PETS.get(id)?.name || id, c.visit, c.download, c.update];
  });

  const perDay = new Map();
  for (const r of daily.results) {
    if (!perDay.has(r.day)) perDay.set(r.day, { visit: 0, download: 0, update: 0 });
    perDay.get(r.day)[r.kind] = r.n;
  }
  const dayRows = [...perDay].map(([day, c]) => [day, c.visit, c.download, c.update]);

  const cell = new Map();
  for (const r of detail.results) {
    const key = `${r.day}|${r.pet_id}`;
    if (!cell.has(key)) cell.set(key, { day: r.day, pet: r.pet_id, visit: 0, download: 0, update: 0 });
    cell.get(key)[r.kind] = r.n;
  }
  const detailRows = [...cell.values()].map((c) => [c.day, PETS.get(c.pet)?.name || c.pet, c.visit, c.download, c.update]);

  const head = ['Visits', 'Downloads', 'Updates'];
  return page(`<h1>DeskPet stats</h1>
<p>Days are UTC. Daily tables cover the last ${days} days (<a href="?token=${esc(encodeURIComponent(token))}&days=90">90</a>, <a href="?token=${esc(encodeURIComponent(token))}&days=365">365</a>).</p>
<h2>Per pet, all time</h2>
${table(['Pet', ...head], petRows, ['Total', sum.visit, sum.download, sum.update])}
<h2>Per day, all pets</h2>
${dayRows.length ? table(['Day', ...head], dayRows) : '<p>Nothing counted yet.</p>'}
<h2>Per day and pet</h2>
${detailRows.length ? table(['Day', 'Pet', ...head], detailRows) : '<p>Nothing counted yet.</p>'}`);
}
