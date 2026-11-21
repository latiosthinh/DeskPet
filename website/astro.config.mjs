import sitemap from "@astrojs/sitemap";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "astro/config";

// Public production origin used by the Mac/Windows apps (windows/src/net.rs, site()).
export const SITE_URL = "https://deskpet.pages.dev";

export default defineConfig({
  site: SITE_URL,
  output: "static",
  trailingSlash: "always",
  build: { format: "directory" },
  // Vite+ (`vp`) does not yet read Astro's config/commands (voidzero-dev/vite-plus#1506);
  // we use Tailwind's Vite plugin directly and oxlint/oxfmt/Vitest for the rest.
  vite: { plugins: [tailwindcss()] },
  integrations: [
    sitemap({
      filter: (page) => !page.endsWith("/404/") && !page.endsWith("/404"),
    }),
  ],
});
