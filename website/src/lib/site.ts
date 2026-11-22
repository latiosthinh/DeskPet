import generated from "../data/pets.generated.json";

/** BNB Smart Chain (BEP-20) support address. Stored once; never log it with card data. */
export const BNB_ADDRESS = "0x4d809c797169fa32448f075c15c4ad0c500bb109";

export const SITE = {
  name: "DeskPet",
  url: "https://deskpet.pages.dev",
  repo: "https://github.com/latiosthinh/DeskPet",
  release: "https://github.com/latiosthinh/DeskPet/releases/latest",
  contributing: "https://github.com/latiosthinh/DeskPet/blob/main/CONTRIBUTING-PETS.md",
} as const;

export interface Category {
  id: string;
  title: string;
  blurb: string;
}

export interface Media {
  src: string;
  poster: string;
  width: number;
  height: number;
  durationMs: number;
  fallback: boolean;
}

export interface Trick extends Media {
  id: string;
  title: string;
}

/** Static, cropped, integer-scaled poster used only by gallery cards. */
export interface GalleryImage {
  src: string;
  width: number;
  height: number;
}

export interface Pet {
  id: string;
  name: string;
  shortName: string;
  category: string;
  author: string;
  version: string;
  description: string;
  builtin: boolean;
  fileSize: number | null;
  trickCount: number;
  hero: Media;
  gallery: GalleryImage;
  og: string;
  tricks: Trick[];
}

export const categories = generated.categories as Category[];
export const pets = generated.pets as Pet[];
export const builtinShortNames = generated.builtinShortNames as string[];
export const qr = generated.qr as {
  src: string;
  webp: string;
  width: number;
  height: number;
};

/** Compact MoMo card built from the pixel-locked poster, QR quiet zone included. */
export const momoCard = generated.momoCard as {
  src: string;
  webp: string;
  width: number;
  height: number;
};

export const petsById = new Map(pets.map((pet) => [pet.id, pet]));

export const INCLUDED_LABEL = "Included with DeskPet";
export const INCLUDED_SHORT_LABEL = "Included";

export interface AuthorGroup {
  name: string;
  slug: string;
  pets: Pet[];
}

export function authorSlug(name: string): string {
  if (name === "DeskPet") return "deskpet";
  const readable = name
    .normalize("NFKD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
  return `${readable || "author"}-${Buffer.from(name, "utf8").toString("base64url")}`;
}

/** Groups pets by author: DeskPet first, then every community author alphabetically. */
export function authorGroups(list: readonly Pet[] = pets): AuthorGroup[] {
  const groups = new Map<string, Pet[]>();
  for (const pet of list) {
    const group = groups.get(pet.author);
    if (group) group.push(pet);
    else groups.set(pet.author, [pet]);
  }
  return [...groups]
    .map(([name, groupPets]) => ({ name, slug: authorSlug(name), pets: groupPets }))
    .sort((a, b) => {
      if (a.name === "DeskPet") return b.name === "DeskPet" ? 0 : -1;
      if (b.name === "DeskPet") return 1;
      return a.name.localeCompare(b.name);
    });
}

export function petsInCategory(category: string): Pet[] {
  return pets.filter((pet) => pet.category === category);
}

export function neighbors(id: string): { previous: Pet | null; next: Pet | null } {
  const index = pets.findIndex((pet) => pet.id === id);
  return {
    previous: index > 0 ? pets[index - 1] : null,
    next: index >= 0 && index < pets.length - 1 ? pets[index + 1] : null,
  };
}

export function absoluteUrl(path: string): string {
  return new URL(path, SITE.url).href;
}

export function formatBytes(bytes: number | null): string | null {
  if (!bytes) return null;
  if (bytes < 1024 * 1024) return `${Math.max(1, Math.round(bytes / 1024))} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

export function formatDuration(ms: number): string {
  return `${Math.max(1, Math.round(ms / 1000))} s`;
}

export function categoryTitle(id: string): string {
  return categories.find((category) => category.id === id)?.title ?? id;
}

export function listText(items: readonly string[]): string {
  if (items.length < 2) return items.join("");
  return `${items.slice(0, -1).join(", ")} and ${items.at(-1)}`;
}

export function petImageAlt(pet: Pet): string {
  return `${pet.name}, a pixel pet for Desktop`;
}
