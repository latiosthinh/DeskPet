// Pet modules are only served through /download/:id so every download is counted.
// Everything else under /files/ (the app installer) is served as usual.
export async function onRequest({ request, next }) {
  let path = new URL(request.url).pathname;
  try {
    path = decodeURIComponent(path);
  } catch {
    // malformed escapes: check the raw path
  }
  if (path.toLowerCase().includes('.deskpet')) {
    return new Response('Download pets from their page on the DeskPet website.', {
      status: 404,
      headers: { 'content-type': 'text/plain; charset=utf-8' },
    });
  }
  return next();
}
