import type { APIRoute } from 'astro';
import { listingRecords } from '../lib/catalogue';
import { absoluteUrl, pets } from '../lib/site';

export const GET: APIRoute = () => {
  const paths = [
    '/',
    '/contribute/',
    '/support/',
    ...pets.map((pet) => `/pets/${pet.id}/`),
    '/authors/',
    ...listingRecords().map((record) => record.path),
  ];
  const urls = paths
    .map((path) => `  <url>\n    <loc>${absoluteUrl(path)}</loc>\n  </url>`)
    .join('\n');
  const body = `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${urls}\n</urlset>\n`;
  return new Response(body, {
    headers: { 'content-type': 'application/xml; charset=utf-8' },
  });
};