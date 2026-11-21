// POST /api/visit/:id  (the pet's detail page calls this once per load)
import { PETS, bump, notFound } from '../../../lib/shared.js';

export async function onRequestPost({ params, env }) {
  const id = String(params.id);
  if (!PETS.has(id)) return notFound('Unknown pet');
  await bump(env, id, 'visit');
  return new Response(null, { status: 204, headers: { 'cache-control': 'no-store' } });
}
