import generated from "../data/pets.generated.json";

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
  og: string;
  tricks: Trick[];
}

export const categories = generated.categories as Category[];
export const pets = generated.pets as Pet[];
export const builtinShortNames = generated.builtinShortNames as string[];
export const qr = generated.qr as { src: string; width: number; height: number };

export const petsById = new Map(pets.map((pet) => [pet.id, pet]));

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
  const seconds = Math.max(0.1, ms / 1000);
  return `${seconds.toFixed(seconds < 10 ? 1 : 0)}s`;
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
