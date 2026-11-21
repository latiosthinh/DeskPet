// GET /download/:id  counts a download, then serves public/files/<id>.deskpet.
// env.ASSETS.fetch reads the static file directly, so it skips functions/files/[[path]].js,
// which blocks the same file for everyone else.
import { PETS, bump, notFound } from '../../lib/shared.js';

export async function onRequestGet({ params, env, request }) {
  const pet = PETS.get(String(params.id));
  if (!pet || pet.builtin) return notFound('No download for this pet');

  const asset = await env.ASSETS.fetch(new URL(`/files/${pet.id}.deskpet`, request.url));
  if (!asset.ok) return notFound('This pet is not available yet');

  await bump(env, pet.id, 'download');
  const headers = new Headers(asset.headers);
  headers.set('content-type', 'application/octet-stream');
  headers.set('content-disposition', `attachment; filename="${pet.id}.deskpet"`);
  headers.set('cache-control', 'no-store');
  return new Response(asset.body, { status: 200, headers });
}
