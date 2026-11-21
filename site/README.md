# DeskPet website

The pet website: a gallery of every pet, a page per pet with GIFs of its tricks, one-click
downloads of `<id>.deskpet` modules, a donation QR code, and per-pet counters.
Static HTML generated from `pets.json`, hosted on Cloudflare Pages, counters in D1.

```
site/
  pets.json            source of truth, regenerated from the pets by `petshot site`
  build.mjs            node build.mjs  ->  public/*.html (no dependencies)
  schema.sql           D1 table: counts(pet_id, kind, day, n)
  wrangler.toml        Pages project + D1 binding DB
  lib/shared.js        pet lookup and counter upsert used by the functions
  functions/
    api/visit/[id].js    POST  counts a visit (the pet page calls it once per load)
    api/update/[id].js   GET   {"version": "..."} for the app, counts an update check
    download/[id].js     GET   counts a download, serves public/files/<id>.deskpet
    files/[[path]].js    404s any direct /files/*.deskpet URL (so downloads can't skip counting)
    stats.js             GET   /stats?token=STATS_TOKEN  visits/downloads/updates per pet and per day
  public/              what gets deployed
    index.html, pets/<id>/index.html, contribute/index.html, 404.html, site.css   (generated)
    pets/<id>/hero.gif, pets/<id>/<trick-id>.gif    pet GIFs (placeholders until real ones exist)
    donate-qr.png                                   donation QR (placeholder until replaced)
    files/<id>.deskpet                              pet modules (only reachable via /download/<id>)
```

## pets.json

```json
{ "id": "ember", "name": "Ember the fox", "category": "critter" | "hero" | "legend",
  "author": "DeskPet", "version": "1.0.0", "description": "...", "builtin": false,
  "tricks": [{ "id": "firework", "title": "Light Fireworks" }], "file_size": 12345 }
```

- `id` is the folder name, the module name (`<id>.deskpet`) and the counter key. Lowercase letters, digits, `-`.
- `builtin: true` pets ship inside the app: no download button, "Comes with the app" instead.
- Trick GIFs live at `public/pets/<id>/<trick-id>.gif`, the big one at `public/pets/<id>/hero.gif`.
- `file_size` (bytes) is optional; if missing, the build reads the size of `public/files/<id>.deskpet`.
- The build checks ids, categories and required fields and stops with an error on bad data.

`build.mjs` rewrites the HTML and CSS every run. It creates a placeholder GIF or QR code only where the
real file is missing, and never overwrites or deletes images or anything in `public/files/`. To get a
fresh placeholder, delete the file and build again.

The functions bundle `pets.json` at deploy time, so redeploy after changing it (new pet, new version).

## Local development

From the repository root:

```powershell
node site/build.mjs
```

Then, from `site/` (wrangler reads `wrangler.toml` and `functions/` from the current folder):

```powershell
cd site
Copy-Item .dev.vars.example .dev.vars
npx wrangler d1 execute deskpet --local --file=schema.sql
npx wrangler pages dev
```

Open http://localhost:8788. Local D1 data lives in `site/.wrangler/` (git-ignored).

- Stats: http://localhost:8788/stats?token=local-dev-token
- To try a download, put any file at `public/files/ember.deskpet`, then open
  http://localhost:8788/download/ember (counted) and http://localhost:8788/files/ember.deskpet (404).

## Deploying (owner)

One-time setup, from `site/`:

```powershell
npx wrangler login
npx wrangler d1 create deskpet
```

Copy the printed `database_id` into `wrangler.toml`, then:

```powershell
npx wrangler d1 execute deskpet --remote --file=schema.sql
npx wrangler pages project create deskpet --production-branch main
npx wrangler pages secret put STATS_TOKEN --project-name deskpet
```

Pick a long random value for `STATS_TOKEN`; the stats page is at `https://<site>/stats?token=<that value>`.
Without the right token it answers 404.

Before the first deploy, replace `public/donate-qr.png` with the real QR code.

Before each deploy, from the repository root:

```powershell
powershell -ExecutionPolicy Bypass -File windows\check-pet.ps1     # builds and checks every pet
target\release\petshot.exe site build\pets site                     # GIFs, modules and pets.json
node site/build.mjs
cd site; npx wrangler pages deploy
```

`petshot site` writes each pet's GIFs into `public/pets/<id>/` and each add-on pet's module into `public/files/`,
and rewrites `pets.json` from the pets themselves: names, descriptions, versions, trick ids and file sizes. The
"Download DeskPet" buttons link to the latest GitHub release, where the app's zip is.

The app's **Get More Pets…** and its update check use `https://deskpet.pages.dev`. If the project gets another
address (or a domain), change `site()` in `windows/src/net.rs` to match before releasing the app.

(`wrangler pages deploy` uploads `public/` and the `functions/` folder; the D1 binding comes from `wrangler.toml`.
If the dashboard's Pages project settings disagree, set the D1 binding `DB` there too under
Settings → Bindings.)

## Counters

`counts` has one row per pet, kind and UTC day; each event is an upsert `n = n + 1`.

| kind       | counted when                                                |
| ---------- | ----------------------------------------------------------- |
| `visit`    | a pet's page loads (POST `/api/visit/<id>` from the page)   |
| `download` | `/download/<id>` serves the module                          |
| `update`   | the app calls GET `/api/update/<id>` to check the version   |

Unknown ids get a 404 and are not counted. Counts are raw: reloads and bots count too.
