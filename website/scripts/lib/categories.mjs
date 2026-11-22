// Shared shelf metadata. Kept dependency-free so prepare-assets and the unit
// tests can import it without touching the build side effects.

export const CATEGORY_COPY = [
  { id: "critter", title: "Critters", blurb: "Small, soft and always up to something." },
  { id: "hero", title: "Heroes", blurb: "Tiny adventurers with big plans." },
  { id: "legend", title: "Legends", blurb: "Detailed, dramatic and a little bit extra." },
];

export const CATEGORY_ID = /^[a-z0-9][a-z0-9-]{0,40}$/;

export function titleCase(id) {
  return id
    .split("-")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}

export function deriveCategories(list) {
  const present = new Set(list.map((pet) => pet.category));
  const known = CATEGORY_COPY.filter((category) => present.has(category.id));
  const unknown = [...present].filter(
    (id) => !CATEGORY_COPY.some((category) => category.id === id),
  );
  return [
    ...known,
    ...unknown.sort().map((id) => {
      const title = titleCase(id);
      return { id, title, blurb: `Community pets on the ${title} shelf.` };
    }),
  ];
}