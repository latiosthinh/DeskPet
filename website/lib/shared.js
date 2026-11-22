// Shared by the Pages Functions. pets.json is bundled in at deploy time,
// so redeploy after regenerating it.
import pets from '../pets.json';

export const PETS = new Map(pets.map((p) => [p.id, p]));
export const KINDS = ['visit', 'download', 'update'];

const BUMP = `INSERT INTO counts (pet_id, kind, day, n) VALUES (?1, ?2, ?3, 1)
  ON CONFLICT (pet_id, kind, day) DO UPDATE SET n = n + 1`;

export function today() {
  return new Date().toISOString().slice(0, 10); // UTC day
}

/** Adds one to a counter. Never throws: counting must not break a download. */
export async function bump(env, petId, kind) {
  if (!env.DB) {
    console.error('DB binding missing; not counting', petId, kind);
    return;
  }
  try {
    await env.DB.prepare(BUMP).bind(petId, kind, today()).run();
  } catch (err) {
    console.error('count failed', petId, kind, err);
  }
}

export function notFound(message = 'Not found') {
  return new Response(message, { status: 404, headers: { 'content-type': 'text/plain; charset=utf-8' } });
}

export function json(body, init = {}) {
  return new Response(JSON.stringify(body), {
    ...init,
    headers: { 'content-type': 'application/json; charset=utf-8', 'cache-control': 'no-store', ...(init.headers || {}) },
  });
}
