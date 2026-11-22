import type { APIRoute } from "astro";
import { authorSlug, pets } from "../../lib/site";
import type { IndexPet } from "../../lib/catalogue-core";

export const GET: APIRoute = () => {
  const index: IndexPet[] = pets.map((pet) => ({
    id: pet.id,
    name: pet.name,
    shortName: pet.shortName,
    description: pet.description,
    author: pet.author,
    authorSlug: authorSlug(pet.author),
    category: pet.category,
    builtin: pet.builtin,
    trickCount: pet.trickCount,
    gallery: {
      src: pet.gallery.src,
      width: pet.gallery.width,
      height: pet.gallery.height,
    },
  }));

  return new Response(JSON.stringify({ pets: index }), {
    headers: { "Content-Type": "application/json; charset=utf-8" },
  });
};