#!/usr/bin/env node
// Builds the DeskPet website from pets.json into public/. No dependencies.
//
//   node site/build.mjs
//
// Writes:   public/index.html, public/pets/<id>/index.html, public/contribute/index.html,
//           public/404.html, public/site.css
// Creates placeholders only when the real file is missing (never overwrites):
//           public/pets/<id>/hero.gif, public/pets/<id>/<trick-id>.gif, public/donate-qr.png
// Never touches public/files/ (the installer and the <id>.deskpet modules live there).

import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { deflateSync } from 'node:zlib';

const ROOT = dirname(fileURLToPath(import.meta.url));
const OUT = join(ROOT, 'public');
const REPO = 'https://github.com/latiosthinh/DeskPet';
const CONTRIBUTING = `${REPO}/blob/main/CONTRIBUTING-PETS.md`;
// The app is a zip on the GitHub release page (see the README's Install section).
const INSTALLER = `${REPO}/releases/latest`;

const CATEGORIES = [
  { id: 'critter', title: 'Critters', blurb: 'Small, soft and always up to something.' },
  { id: 'hero', title: 'Heroes', blurb: 'Tiny adventurers with big plans.' },
  { id: 'legend', title: 'Legends', blurb: 'Detailed, dramatic and a little bit extra.' },
];

// ---------------------------------------------------------------- pets.json

const pets = JSON.parse(readFileSync(join(ROOT, 'pets.json'), 'utf8'));
const ID = /^[a-z0-9][a-z0-9-]{0,40}$/;
const seen = new Set();
for (const p of pets) {
  const where = `pets.json: pet "${p.id}"`;
  if (!ID.test(p.id ?? '')) throw new Error(`${where}: id must match ${ID}`);
  if (seen.has(p.id)) throw new Error(`${where}: duplicate id`);
  seen.add(p.id);
  for (const key of ['name', 'author', 'version', 'description']) {
    if (typeof p[key] !== 'string' || !p[key]) throw new Error(`${where}: missing ${key}`);
  }
  if (!CATEGORIES.some((c) => c.id === p.category)) throw new Error(`${where}: unknown category "${p.category}"`);
  if (!Array.isArray(p.tricks)) throw new Error(`${where}: tricks must be a list`);
  for (const t of p.tricks) {
    if (!ID.test(t.id ?? '') || t.id === 'hero') throw new Error(`${where}: bad trick id "${t.id}"`);
    if (typeof t.title !== 'string' || !t.title) throw new Error(`${where}: trick "${t.id}" needs a title`);
  }
}

// ---------------------------------------------------------------- html helpers

const esc = (s) =>
  String(s).replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]);

function fileSize(p) {
  let bytes = p.file_size;
  if (!bytes) {
    const f = join(OUT, 'files', `${p.id}.deskpet`);
    if (existsSync(f)) bytes = statSync(f).size;
  }
  if (!bytes) return '';
  return bytes < 1024 * 1024 ? `${Math.max(1, Math.round(bytes / 1024))} KB` : `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

const builtinNames = pets.filter((p) => p.builtin).map((p) => p.name.split(' the ')[0]);
const listText = (a) => (a.length < 2 ? a.join('') : `${a.slice(0, -1).join(', ')} and ${a.at(-1)}`);

const FAVICON =
  "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 8 8' shape-rendering='crispEdges'%3E%3Cpath fill='%2333293d' d='M1 1h1v1h1v1h2V2h1V1h1v6H1z'/%3E%3Cpath fill='%23f3c98b' d='M2 3h4v3H2z'/%3E%3Cpath fill='%2333293d' d='M3 4h1v1H3zm2 0h1v1H5z'/%3E%3C/svg%3E";

function layout({ title, description, path, body, scripts = '' }) {
  const nav = [
    ['/', 'Pets'],
    ['/contribute/', 'Make a pet'],
  ]
    .map(([href, label]) => `<a href="${href}"${href === path ? ' aria-current="page"' : ''}>${label}</a>`)
    .join('');
  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${esc(title)}</title>
<meta name="description" content="${esc(description)}">
<meta property="og:title" content="${esc(title)}">
<meta property="og:description" content="${esc(description)}">
<link rel="icon" href="${FAVICON}">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Pixelify+Sans:wght@500;700&display=swap">
<link rel="stylesheet" href="/site.css">
</head>
<body>
<header class="top">
  <a class="brand" href="/"><img src="/pets/mochi/hero.gif" alt="" width="32" height="32"><span>DeskPet</span></a>
  <nav>${nav}<a class="btn btn-small" href="${INSTALLER}">Download</a></nav>
</header>
<main>
${body}
</main>
<footer class="foot">
  <p>DeskPet is free. Pets are made by the DeskPet community. <a href="${REPO}">Source on GitHub</a> · <a href="/contribute/">Make a pet</a></p>
</footer>
${scripts}
</body>
</html>
`;
}

function petCard(p) {
  const n = p.tricks.length;
  return `<li><a class="card" href="/pets/${p.id}/">
    <span class="frame"><img src="/pets/${p.id}/hero.gif" alt="${esc(p.name)}" loading="lazy"></span>
    <span class="card-name">${esc(p.name)}</span>
    <span class="card-meta">${n} trick${n === 1 ? '' : 's'}${p.builtin ? ' <span class="badge">Included</span>' : ''}</span>
  </a></li>`;
}

function donateSection(name) {
  return `<section class="panel donate" aria-labelledby="donate">
  <img class="qr" src="/donate-qr.png" alt="Donation QR code" width="180" height="180">
  <div>
    <h2 id="donate">Support DeskPet</h2>
    <p>DeskPet and every pet on this site are free, with no ads and no tracking. If ${esc(name)} made your desk a little
    happier, you can chip in by scanning the code. It keeps new pets coming.</p>
  </div>
</section>`;
}

// ---------------------------------------------------------------- pages

function homePage() {
  const sections = CATEGORIES.map((c) => {
    const list = pets.filter((p) => p.category === c.id);
    if (!list.length) return '';
    return `<section class="group" aria-labelledby="g-${c.id}">
  <div class="group-head"><h2 id="g-${c.id}">${c.title}</h2><p>${c.blurb}</p></div>
  <ul class="grid">${list.map(petCard).join('')}</ul>
</section>`;
  }).join('\n');

  const trio = pets
    .filter((p) => p.builtin)
    .map((p) => `<img src="/pets/${p.id}/hero.gif" alt="${esc(p.name)}">`)
    .join('');

  const body = `<section class="hero">
  <div class="hero-text">
    <h1>Tiny pixel pets that live on your desktop.</h1>
    <p class="lead">They walk along your taskbar, nap when you're busy, cheer when you type and do little tricks when you're not looking.</p>
    <p><a class="btn" href="${INSTALLER}">Download DeskPet</a></p>
    <p class="small">Free for Windows. Comes with ${esc(listText(builtinNames))}. Every other pet is a free one-click add-on below.</p>
  </div>
  <div class="hero-pets" aria-hidden="true">${trio}</div>
</section>
${sections}
<section class="panel cta">
  <div><h2>Make your own pet</h2><p>Pets are small files anyone can build. Draw one, give it tricks and share it here.</p></div>
  <a class="btn btn-ghost" href="/contribute/">How to make a pet</a>
</section>`;
  return layout({
    title: 'DeskPet: tiny pixel pets for your desktop',
    description: 'Free pixel pets that live on your desktop. Pick a pet, watch its tricks and add it to DeskPet in one click.',
    path: '/',
    body,
  });
}

function petPage(p) {
  const cat = CATEGORIES.find((c) => c.id === p.category);
  const size = fileSize(p);
  const action = p.builtin
    ? `<p class="included"><span class="badge">Included</span> Comes with the app</p>`
    : `<p><a class="btn" href="/download/${p.id}" download="${p.id}.deskpet">Download ${esc(p.name.split(' the ')[0])}</a></p>
    <p class="small">${esc(p.id)}.deskpet${size ? ` · ${size}` : ''} · version ${esc(p.version)}</p>`;

  const steps = p.builtin
    ? `<ol class="steps">
    <li><a href="${INSTALLER}">Download DeskPet</a>, unzip it and run <b>DeskPet.exe</b>.</li>
    <li>${esc(p.name)} is already inside: pick ${esc(p.name.split(' the ')[0])} from the DeskPet menu.</li>
  </ol>`
    : `<ol class="steps">
    <li>Have <a href="${INSTALLER}">DeskPet</a> running.</li>
    <li>Download <b>${esc(p.id)}.deskpet</b> with the button above.</li>
    <li>Double-click the file, or drag it and drop it on your pet.</li>
    <li>${esc(p.name)} joins your desktop and stays in the pet list for next time.</li>
  </ol>`;

  const tricks = p.tricks.length
    ? `<section aria-labelledby="tricks">
  <h2 id="tricks">Tricks</h2>
  <ul class="grid tricks">${p.tricks
    .map(
      (t) => `<li class="card trick"><span class="frame"><img src="/pets/${p.id}/${t.id}.gif" alt="${esc(p.name)}: ${esc(t.title)}" loading="lazy"></span>
      <span class="card-name">${esc(t.title)}</span></li>`,
    )
    .join('')}</ul>
</section>`
    : '';

  const body = `<p class="crumbs"><a href="/">← All pets</a> · ${cat.title}</p>
<section class="pet">
  <div class="frame frame-big"><img src="/pets/${p.id}/hero.gif" alt="${esc(p.name)}"></div>
  <div class="pet-text">
    <h1>${esc(p.name)}</h1>
    <p class="lead">${esc(p.description)}</p>
    <p class="small">Made by ${esc(p.author)}</p>
    ${action}
  </div>
</section>
${tricks}
<section class="panel" aria-labelledby="install">
  <h2 id="install">${p.builtin ? 'Get ' + esc(p.name.split(' the ')[0]) : 'How to install'}</h2>
  ${steps}
</section>
${donateSection(p.name.split(' the ')[0])}`;

  const scripts = `<script>
fetch('/api/visit/${p.id}', { method: 'POST', keepalive: true }).catch(function () {});
</script>`;
  return layout({
    title: `${p.name} · DeskPet`,
    description: p.description,
    path: `/pets/${p.id}/`,
    body,
    scripts,
  });
}

function contributePage() {
  const body = `<section class="prose">
  <h1>Make a pet</h1>
  <p class="lead">Every pet on this site is a single small file. Anyone can make one, and good ones get a page here next to the rest.</p>
  <h2>What a pet is</h2>
  <p>A pet is a <b>.deskpet</b> file: a tiny WebAssembly module built with the DeskPet pet SDK. It holds the pixel art, the walk
  and sleep poses, and the tricks. DeskPet runs it safely in a sandbox, so a pet can only draw itself and react to your desktop.</p>
  <h2>How it works</h2>
  <ol class="steps">
    <li>Read the <a href="${CONTRIBUTING}">pet guide (CONTRIBUTING-PETS.md)</a> in the DeskPet repository.</li>
    <li>Draw your pet and its tricks, and build the .deskpet file.</li>
    <li>Try it: double-click the file, or drop it on your pet in DeskPet.</li>
    <li>Open a pull request. Once it's merged, your pet gets its own page here with GIFs of every trick, credited to you.</li>
  </ol>
  <p><a class="btn" href="${CONTRIBUTING}">Read the pet guide</a></p>
  <h2>A few house rules</h2>
  <ul>
    <li>Original art only, or art you have the rights to share.</li>
    <li>Keep it friendly: pets live on people's work screens.</li>
    <li>Small is good. Most pets are a 16×16 grid with a handful of colours.</li>
  </ul>
</section>`;
  return layout({
    title: 'Make a pet · DeskPet',
    description: 'Make your own DeskPet pet and share it with everyone.',
    path: '/contribute/',
    body,
  });
}

function notFoundPage() {
  return layout({
    title: 'Not found · DeskPet',
    description: 'This page wandered off.',
    path: '',
    body: `<section class="prose center"><h1>This page wandered off.</h1><p><a class="btn" href="/">See all pets</a></p></section>`,
  });
}

// ---------------------------------------------------------------- css

const CSS = `:root {
  --bg: #fbf3e4; --paper: #fffaf1; --ink: #33293d; --muted: #6d6177; --line: #33293d;
  --accent: #e47c63; --accent-ink: #fffaf1; --frame: #f1e3c8; --badge: #729d98; --badge-ink: #fffaf1;
  --shadow: 4px 4px 0 var(--line);
  color-scheme: light;
}
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    --bg: #1e1a24; --paper: #2a2432; --ink: #efe4d2; --muted: #b3a7bd; --line: #0e0b12;
    --accent: #f08b70; --accent-ink: #1e1a24; --frame: #f1e3c8; --badge: #8bbbb5; --badge-ink: #1e1a24;
    color-scheme: dark;
  }
}
:root[data-theme="dark"] {
  --bg: #1e1a24; --paper: #2a2432; --ink: #efe4d2; --muted: #b3a7bd; --line: #0e0b12;
  --accent: #f08b70; --accent-ink: #1e1a24; --frame: #f1e3c8; --badge: #8bbbb5; --badge-ink: #1e1a24;
  color-scheme: dark;
}
* { box-sizing: border-box; }
html { -webkit-text-size-adjust: 100%; }
body { margin: 0; background: var(--bg); color: var(--ink); font: 17px/1.55 system-ui, -apple-system, "Segoe UI", sans-serif; }
img { image-rendering: pixelated; image-rendering: crisp-edges; }
a { color: inherit; text-decoration-thickness: 2px; text-underline-offset: 3px; }
h1, h2, .brand, .btn, .card-name { font-family: "Pixelify Sans", system-ui, sans-serif; font-weight: 700; letter-spacing: .01em; }
h1 { font-size: clamp(30px, 6vw, 48px); line-height: 1.1; margin: 0 0 12px; }
h2 { font-size: 26px; line-height: 1.2; margin: 0 0 10px; }
p { margin: 0 0 12px; }
.lead { font-size: 19px; }
.small { font-size: 14px; color: var(--muted); }
main, .top, .foot { max-width: 1080px; margin: 0 auto; padding: 0 16px; }

.top { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding-top: 14px; padding-bottom: 14px; flex-wrap: wrap; }
.brand { display: flex; align-items: center; gap: 8px; font-size: 24px; text-decoration: none; }
.brand img { width: 32px; height: 32px; }
nav { display: flex; align-items: center; gap: 16px; flex-wrap: wrap; }
nav a { font-weight: 600; text-decoration: none; }
nav a[aria-current="page"], nav a:not(.btn):hover { text-decoration: underline; }

.btn { display: inline-block; background: var(--accent); color: var(--accent-ink); border: 3px solid var(--line); box-shadow: var(--shadow);
  padding: 10px 20px; font-size: 20px; text-decoration: none; border-radius: 0; transition: transform .08s, box-shadow .08s; }
.btn:hover { transform: translate(-1px, -1px); box-shadow: 5px 5px 0 var(--line); }
.btn:active { transform: translate(3px, 3px); box-shadow: 1px 1px 0 var(--line); }
.btn:focus-visible, .card:focus-visible, a:focus-visible { outline: 3px solid var(--badge); outline-offset: 3px; }
.btn-small { font-size: 16px; padding: 4px 12px; box-shadow: 3px 3px 0 var(--line); }
.btn-ghost { background: var(--paper); color: var(--ink); }

.badge { display: inline-block; background: var(--badge); color: var(--badge-ink); font-size: 12px; font-weight: 700; letter-spacing: .04em;
  text-transform: uppercase; padding: 1px 7px; border: 2px solid var(--line); vertical-align: 1px; }

.hero { display: grid; grid-template-columns: 1.2fr 1fr; gap: 32px; align-items: center; padding: 40px 0 24px; }
.hero-pets { display: flex; justify-content: center; align-items: flex-end; gap: 4px; background: var(--frame); border: 3px solid var(--line);
  box-shadow: var(--shadow); padding: 24px 12px 0; min-height: 220px; }
.hero-pets img { width: 30%; max-width: 140px; height: auto; }

.group { padding: 28px 0 8px; }
.group-head { display: flex; align-items: baseline; gap: 14px; flex-wrap: wrap; margin-bottom: 14px; }
.group-head h2 { margin: 0; }
.group-head p { margin: 0; color: var(--muted); }

.grid { list-style: none; margin: 0; padding: 0; display: grid; gap: 18px; grid-template-columns: repeat(auto-fill, minmax(170px, 1fr)); }
.card { display: flex; flex-direction: column; gap: 6px; height: 100%; background: var(--paper); border: 3px solid var(--line); box-shadow: var(--shadow);
  padding: 10px; text-decoration: none; transition: transform .08s, box-shadow .08s; }
a.card:hover { transform: translate(-2px, -2px); box-shadow: 6px 6px 0 var(--line); }
.card-name { font-size: 19px; line-height: 1.2; }
.card-meta { font-size: 14px; color: var(--muted); display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
/* Picture frames stay light in dark mode too: the GIFs are drawn on this colour (petshot BACKGROUND). */
.frame { display: flex; align-items: center; justify-content: center; aspect-ratio: 1; background: var(--frame); overflow: hidden; }
.frame img { width: 100%; height: 100%; object-fit: contain; }

.crumbs { margin: 16px 0 0; font-size: 15px; color: var(--muted); }
.pet { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1.1fr); gap: 32px; align-items: center; padding: 16px 0 32px; }
.frame-big { border: 3px solid var(--line); box-shadow: var(--shadow); }
.included { font-weight: 600; display: flex; gap: 10px; align-items: center; }
.tricks { grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); margin-bottom: 36px; }
.trick .card-name { font-size: 17px; }

.panel { background: var(--paper); border: 3px solid var(--line); box-shadow: var(--shadow); padding: 20px; margin: 0 0 28px; }
.steps { margin: 0; padding-left: 22px; }
.steps li { margin: 0 0 6px; padding-left: 4px; }
.donate { display: flex; gap: 22px; align-items: center; }
.qr { width: 180px; height: 180px; flex: none; background: #fff; border: 3px solid var(--line); padding: 6px; }
.cta { display: flex; gap: 20px; align-items: center; justify-content: space-between; margin-top: 28px; }
.cta p { margin: 0; }

.prose { max-width: 720px; padding: 32px 0 16px; }
.prose h2 { margin-top: 28px; }
.prose ul { padding-left: 22px; }
.center { margin: 0 auto; text-align: center; padding: 80px 0; }

.foot { padding-top: 24px; padding-bottom: 40px; font-size: 14px; color: var(--muted); }

@media (max-width: 760px) {
  .hero, .pet { grid-template-columns: 1fr; gap: 20px; }
  .hero { padding-top: 20px; }
  .hero-pets { min-height: 160px; }
  .grid { grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
  .tricks { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .card-name { font-size: 17px; }
  .donate, .cta { flex-direction: column; align-items: flex-start; }
  .btn { font-size: 18px; }
}
@media (prefers-reduced-motion: reduce) { .btn, .card { transition: none; } }
`;

// ---------------------------------------------------------------- placeholder images

const CRC_TABLE = Array.from({ length: 256 }, (_, n) => {
  let c = n;
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  return c >>> 0;
});
function crc32(buf) {
  let c = 0xffffffff;
  for (const b of buf) c = CRC_TABLE[(c ^ b) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

/** Minimal GIF89a encoder: up to 4 colours (index 0 transparent), looping frames. */
function gif(w, h, palette, frames) {
  const bytes = [];
  const u16 = (n) => bytes.push(n & 0xff, (n >> 8) & 0xff);
  bytes.push(...Buffer.from('GIF89a'));
  u16(w); u16(h);
  bytes.push(0xf1, 0, 0); // global colour table, 4 entries
  for (let i = 0; i < 4; i++) bytes.push(...(palette[i] || [0, 0, 0]));
  bytes.push(0x21, 0xff, 11, ...Buffer.from('NETSCAPE2.0'), 3, 1, 0, 0, 0); // loop forever
  for (const f of frames) {
    bytes.push(0x21, 0xf9, 4, 0x09); // restore to background, transparent index used
    u16(Math.round(f.delay / 10));
    bytes.push(0, 0);
    bytes.push(0x2c); u16(0); u16(0); u16(w); u16(h); bytes.push(0);
    // LZW with 3-bit codes: a clear code every two pixels keeps the table from growing.
    bytes.push(2);
    const codes = [];
    f.pixels.forEach((px, i) => {
      if (i % 2 === 0) codes.push(4);
      codes.push(px);
    });
    codes.push(5);
    const data = [];
    let acc = 0, bits = 0;
    for (const c of codes) {
      acc |= c << bits; bits += 3;
      while (bits >= 8) { data.push(acc & 0xff); acc >>= 8; bits -= 8; }
    }
    if (bits) data.push(acc & 0xff);
    for (let i = 0; i < data.length; i += 255) {
      const chunk = data.slice(i, i + 255);
      bytes.push(chunk.length, ...chunk);
    }
    bytes.push(0);
  }
  bytes.push(0x3b);
  return Buffer.from(bytes);
}

/** Minimal greyscale PNG. */
function png(w, h, pixel) {
  const raw = Buffer.alloc((w + 1) * h);
  for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) raw[y * (w + 1) + 1 + x] = pixel(x, y);
  const chunk = (type, data) => {
    const len = Buffer.alloc(4); len.writeUInt32BE(data.length);
    const td = Buffer.concat([Buffer.from(type), data]);
    const crc = Buffer.alloc(4); crc.writeUInt32BE(crc32(td));
    return Buffer.concat([len, td, crc]);
  };
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(w, 0); ihdr.writeUInt32BE(h, 4); ihdr[8] = 8; ihdr[9] = 0;
  return Buffer.concat([Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]), chunk('IHDR', ihdr), chunk('IDAT', deflateSync(raw)), chunk('IEND', Buffer.alloc(0))]);
}

function hash(s) {
  let h = 2166136261;
  for (const c of s) h = Math.imul(h ^ c.charCodeAt(0), 16777619);
  return h >>> 0;
}
function hsl(h, s, l) {
  const k = (n) => (n + h / 30) % 12, a = s * Math.min(l, 1 - l);
  const f = (n) => Math.round(255 * (l - a * Math.max(-1, Math.min(k(n) - 3, 9 - k(n), 1))));
  return [f(0), f(8), f(4)];
}

// 16×16 placeholder critter: 1 outline, 2 body, 3 light.
const BLOB = [
  '................',
  '................',
  '...11......11...',
  '..1221....1221..',
  '..1222111122221.',
  '..122222222221..',
  '.12222222222221.',
  '.12212222221221.',
  '.12212222221221.',
  '.12222333322221.',
  '.12223333332221.',
  '.12222333322221.',
  '..122222222221..',
  '...1111111111...',
  '..11........11..',
  '................',
];

function sprite({ dy = 0, blink = false, sparkle = 0 }) {
  const px = new Array(256).fill(0);
  BLOB.forEach((row, y) => {
    [...row].forEach((c, x) => {
      const ty = y - dy;
      if (c !== '.' && ty >= 0 && ty < 16) px[ty * 16 + x] = Number(c);
    });
  });
  if (blink) for (const x of [4, 11]) { px[(8 - dy) * 16 + x] = 2; }
  if (sparkle) {
    const sx = sparkle === 1 ? 13 : 2, sy = sparkle === 1 ? 1 : 3;
    for (const [ox, oy] of [[0, 0], [-1, 0], [1, 0], [0, -1], [0, 1]]) {
      const x = sx + ox, y = sy + oy;
      if (x >= 0 && x < 16 && y >= 0 && y < 16 && px[y * 16 + x] === 0) px[y * 16 + x] = 3;
    }
  }
  return px;
}

function placeholderGif(seed, trick) {
  const h = hash(seed);
  const palette = [[255, 255, 255], [51, 41, 61], hsl(h % 360, 0.55, 0.68), [255, 247, 232]];
  const frames = trick
    ? [
        { pixels: sprite({}), delay: 300 },
        { pixels: sprite({ dy: 2, sparkle: 1 }), delay: 160 },
        { pixels: sprite({ dy: 1, sparkle: 2, blink: true }), delay: 160 },
        { pixels: sprite({}), delay: 300 },
      ]
    : [
        { pixels: sprite({}), delay: 500 },
        { pixels: sprite({ dy: 1 }), delay: 250 },
        { pixels: sprite({}), delay: 700 },
        { pixels: sprite({ blink: true }), delay: 140 },
      ];
  return gif(16, 16, palette, frames);
}

function placeholderQr() {
  // Not a real QR code: replace public/donate-qr.png with the owner's.
  const n = 29;
  const finder = (x, y) => {
    for (const [fx, fy] of [[0, 0], [n - 7, 0], [0, n - 7]]) {
      const dx = x - fx, dy = y - fy;
      if (dx >= -1 && dx <= 7 && dy >= -1 && dy <= 7) {
        if (dx < 0 || dy < 0 || dx > 6 || dy > 6) return 255;
        const ring = Math.min(dx, dy, 6 - dx, 6 - dy);
        return ring === 1 ? 255 : 0;
      }
    }
    return -1;
  };
  return png(n, n, (x, y) => {
    const f = finder(x, y);
    if (f >= 0) return f;
    return hash(`${x},${y}`) % 3 === 0 ? 0 : 255;
  });
}

// ---------------------------------------------------------------- write

let written = 0, placeholders = 0;
function write(rel, data) {
  const file = join(OUT, rel);
  mkdirSync(dirname(file), { recursive: true });
  writeFileSync(file, data);
  written++;
}
function placeholder(rel, make) {
  const file = join(OUT, rel);
  if (existsSync(file)) return;
  mkdirSync(dirname(file), { recursive: true });
  writeFileSync(file, make());
  placeholders++;
}

write('site.css', CSS);
write('index.html', homePage());
write('contribute/index.html', contributePage());
write('404.html', notFoundPage());
for (const p of pets) {
  write(`pets/${p.id}/index.html`, petPage(p));
  placeholder(`pets/${p.id}/hero.gif`, () => placeholderGif(p.id, false));
  for (const t of p.tricks) placeholder(`pets/${p.id}/${t.id}.gif`, () => placeholderGif(`${p.id}/${t.id}`, true));
}
placeholder('donate-qr.png', placeholderQr);
mkdirSync(join(OUT, 'files'), { recursive: true });

console.log(`Built ${pets.length} pets: ${written} files written, ${placeholders} placeholder images created in ${OUT}`);
