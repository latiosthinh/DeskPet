// GET /api/update/:id  -> {"version": "1.2.0"}  (the app asks this to see if a pet has an update)
import { PETS, bump, json, notFound } from '../../../lib/shared.js';

export async function onRequestGet({ params, env }) {
  const pet = PETS.get(String(params.id));
  if (!pet) return notFound('Unknown pet');
  await bump(env, pet.id, 'update');
  return json({ version: pet.version });
}
