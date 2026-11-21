// Sprite normalisation helpers shared by the build-time asset step and its tests.
//
// The GIFs written by `petshot` are faithful to the desktop app, but each one is
// cropped to its own moving content: a speech bubble, a firework blast or a
// walking path can make the same pet look tiny in one clip and large in another.
// These helpers find the real pet body, remove the baked-in greeting bubble and
// express every frame on one transparent canvas at one body size.

import sharp from "sharp";

/** The paper colour petshot paints behind every GIF (site --frame). */
export const SPRITE_BG = [241, 227, 200];
/** Integer enlargement target for the pet body, in output pixels. */
export const TARGET_BODY_HEIGHT = 96;
/** Transparent breathing room around the scaled artwork. */
export const CANVAS_MARGIN = 4;
/** Shared gallery poster canvas (4:3, displayed at its natural 1x size). */
export const GALLERY_WIDTH = 224;
export const GALLERY_HEIGHT = 168;

const BG_TOLERANCE = 6;
const WHITE = [245, 240, 230];
const BUBBLE_PADDING = 4;

/** Decode every frame of a GIF as one stacked RGBA buffer. */
export async function readFrames(file) {
  const meta = await sharp(file).metadata();
  const { data, info } = await sharp(file, { animated: true })
    .ensureAlpha()
    .raw()
    .toBuffer({ resolveWithObject: true });
  const pageHeight = info.height / meta.pages;
  return {
    data,
    width: info.width,
    height: pageHeight,
    pages: meta.pages,
    channels: info.channels,
    stride: info.channels,
    pageBytes: info.width * pageHeight * info.channels,
    delays: Array.isArray(meta.delay) ? meta.delay : [meta.delay ?? 100],
  };
}

export function frameView(frames, index) {
  const start = index * frames.pageBytes;
  return frames.data.subarray(start, start + frames.pageBytes);
}

export function isBackground(data, offset) {
  return (
    Math.abs(data[offset] - SPRITE_BG[0]) <= BG_TOLERANCE &&
    Math.abs(data[offset + 1] - SPRITE_BG[1]) <= BG_TOLERANCE &&
    Math.abs(data[offset + 2] - SPRITE_BG[2]) <= BG_TOLERANCE
  );
}

export function isContent(data, offset) {
  return data[offset + 3] > 16 && !isBackground(data, offset);
}

export function isWhite(data, offset) {
  return (
    data[offset + 3] > 16 &&
    data[offset] >= WHITE[0] &&
    data[offset + 1] >= WHITE[1] &&
    data[offset + 2] >= WHITE[2]
  );
}

/**
 * The greeting bubble is the largest near-white rounded box in the top half of
 * a hero frame. Returns its padded box, or null when the frame has no bubble.
 */
export function findBubble(data, width, height) {
  const mask = new Uint8Array(width * height);
  for (let p = 0, i = 0; p < mask.length; p++, i += 4) {
    if (isWhite(data, i)) mask[p] = 1;
  }
  const components = connectedComponents(mask, width, height);
  for (const box of components) {
    if (box.area < 150) break;
    if (box.minY >= height * 0.5 || box.maxY >= height * 0.65) continue;
    if (box.width < 24 || box.width < box.height * 1.6) continue;
    return {
      minX: Math.max(0, box.minX - BUBBLE_PADDING),
      minY: Math.max(0, box.minY - BUBBLE_PADDING),
      maxX: Math.min(width - 1, box.maxX + BUBBLE_PADDING),
      maxY: Math.min(height - 1, box.maxY + BUBBLE_PADDING),
    };
  }
  return null;
}

export function insideBox(x, y, box) {
  return x >= box.minX && x <= box.maxX && y >= box.minY && y <= box.maxY;
}

/** Bounding boxes of 8-connected runs in a binary mask, largest first. */
export function connectedComponents(mask, width, height) {
  const seen = new Uint8Array(mask.length);
  const out = [];
  for (let start = 0; start < mask.length; start++) {
    if (!mask[start] || seen[start]) continue;
    let minX = width;
    let minY = height;
    let maxX = -1;
    let maxY = -1;
    let area = 0;
    const stack = [start];
    seen[start] = 1;
    while (stack.length) {
      const index = stack.pop();
      const x = index % width;
      const y = (index / width) | 0;
      area++;
      if (x < minX) minX = x;
      if (y < minY) minY = y;
      if (x > maxX) maxX = x;
      if (y > maxY) maxY = y;
      for (let dy = -1; dy <= 1; dy++) {
        for (let dx = -1; dx <= 1; dx++) {
          const nx = x + dx;
          const ny = y + dy;
          if (nx < 0 || ny < 0 || nx >= width || ny >= height) continue;
          const next = ny * width + nx;
          if (mask[next] && !seen[next]) {
            seen[next] = 1;
            stack.push(next);
          }
        }
      }
    }
    out.push({ minX, minY, maxX, maxY, width: maxX - minX + 1, height: maxY - minY + 1, area });
  }
  out.sort((a, b) => b.area - a.area);
  return out;
}

/**
 * The pet body is the largest connected run of artwork once the greeting is
 * gone. Props are usually separate components or thinner than the body.
 */
export function bodyOf(data, width, height, bubble) {
  return componentPixels(data, width, height, bubble).box;
}

/**
 * The largest body component as a pixel mask. Detached hearts, sparks and other
 * props are separate components, so using only this mask keeps cards clean.
 */
export function largestBody(data, width, height, bubble) {
  return componentPixels(data, width, height, bubble);
}

function componentPixels(data, width, height, bubble) {
  const mask = new Uint8Array(width * height);
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const p = y * width + x;
      if (bubble && insideBox(x, y, bubble)) continue;
      if (isContent(data, p * 4)) mask[p] = 1;
    }
  }
  const seen = new Uint8Array(mask.length);
  let best = null;
  for (let start = 0; start < mask.length; start++) {
    if (!mask[start] || seen[start]) continue;
    let minX = width;
    let minY = height;
    let maxX = -1;
    let maxY = -1;
    let area = 0;
    const pixels = [];
    const stack = [start];
    seen[start] = 1;
    while (stack.length) {
      const index = stack.pop();
      const x = index % width;
      const y = (index / width) | 0;
      area++;
      pixels.push(index);
      if (x < minX) minX = x;
      if (y < minY) minY = y;
      if (x > maxX) maxX = x;
      if (y > maxY) maxY = y;
      for (let dy = -1; dy <= 1; dy++) {
        for (let dx = -1; dx <= 1; dx++) {
          const nx = x + dx;
          const ny = y + dy;
          if (nx < 0 || ny < 0 || nx >= width || ny >= height) continue;
          const next = ny * width + nx;
          if (mask[next] && !seen[next]) {
            seen[next] = 1;
            stack.push(next);
          }
        }
      }
    }
    if (!best || area > best.area) {
      best = {
        minX,
        minY,
        maxX,
        maxY,
        width: maxX - minX + 1,
        height: maxY - minY + 1,
        area,
        pixels,
      };
    }
  }
  if (!best) return { box: null, mask: null };
  const keep = new Uint8Array(width * height);
  for (const index of best.pixels) keep[index] = 1;
  const { pixels: _, ...box } = best;
  return { box, mask: keep };
}

/** Median body height over every frame of a hero clip. */
export function heroBodyHeight(frames) {
  const heights = [];
  for (let frame = 0; frame < frames.pages; frame++) {
    const data = frameView(frames, frame);
    const bubble = findBubble(data, frames.width, frames.height);
    const body = bodyOf(data, frames.width, frames.height, bubble);
    if (body) heights.push(body.height);
  }
  if (!heights.length) return TARGET_BODY_HEIGHT;
  heights.sort((a, b) => a - b);
  return heights[heights.length >> 1];
}

/**
 * Integer nearest-neighbour enlargement that puts a body as close as possible to
 * the shared target without ever resampling on a fractional grid.
 */
export function targetScale(bodyHeight) {
  return Math.max(1, Math.round(TARGET_BODY_HEIGHT / Math.max(1, bodyHeight)));
}

/**
 * The source art is a mix of 1x and 2x pixel grids. Detect which one a pet uses
 * by checking whether its body is composed of 2x2 identical pixel blocks; 2x art
 * must be enlarged by even factors (2, 4, ...) to stay on its own grid.
 */
export function detectPixelGrid(frames) {
  const samples = new Set([0, frames.pages >> 1, frames.pages - 1]);
  let best = 0;
  for (const frame of samples) {
    const data = frameView(frames, frame);
    const bubble = findBubble(data, frames.width, frames.height);
    const body = bodyOf(data, frames.width, frames.height, bubble);
    if (!body) continue;
    for (let oy = 0; oy < 2; oy++) {
      for (let ox = 0; ox < 2; ox++) {
        let same = 0;
        let total = 0;
        for (let y = body.minY + oy; y + 1 <= body.maxY; y += 2) {
          for (let x = body.minX + ox; x + 1 <= body.maxX; x += 2) {
            const offset = (y * frames.width + x) * 4;
            const right = offset + 4;
            const below = offset + frames.width * 4;
            const corner = below + 4;
            if (![offset, right, below, corner].every((p) => isContent(data, p))) continue;
            total++;
            if (
              data[offset] === data[right] &&
              data[offset] === data[below] &&
              data[offset] === data[corner] &&
              data[offset + 1] === data[right + 1] &&
              data[offset + 1] === data[below + 1] &&
              data[offset + 1] === data[corner + 1] &&
              data[offset + 2] === data[right + 2] &&
              data[offset + 2] === data[below + 2] &&
              data[offset + 2] === data[corner + 2]
            ) {
              same++;
            }
          }
        }
        if (total) best = Math.max(best, same / total);
      }
    }
  }
  return best >= 0.9 ? 2 : 1;
}

/** Frame whose body is closest to the median body height (the settled pose). */
export function settledFrameIndex(frames, medianBody) {
  let bestIndex = 0;
  let bestScore = Infinity;
  let bestArea = 0;
  for (let frame = 0; frame < frames.pages; frame++) {
    const data = frameView(frames, frame);
    const bubble = findBubble(data, frames.width, frames.height);
    const body = bodyOf(data, frames.width, frames.height, bubble);
    if (!body) continue;
    const score = Math.abs(body.height - medianBody);
    if (score < bestScore || (score === bestScore && body.area > bestArea)) {
      bestIndex = frame;
      bestScore = score;
      bestArea = body.area;
    }
  }
  return bestIndex;
}

/**
 * Union of everything that is not background, ignoring the greeting bubble when
 * asked. Used only to size the normalised canvas, never to scale the pet.
 */
export function contentBounds(frames, { greeting = false } = {}) {
  const { width, height } = frames;
  const box = { minX: width, minY: height, maxX: -1, maxY: -1 };
  for (let frame = 0; frame < frames.pages; frame++) {
    const data = frameView(frames, frame);
    const bubble = greeting ? findBubble(data, width, height) : null;
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const p = y * width + x;
        if (bubble && insideBox(x, y, bubble)) continue;
        if (!isContent(data, p * 4)) continue;
        if (x < box.minX) box.minX = x;
        if (y < box.minY) box.minY = y;
        if (x > box.maxX) box.maxX = x;
        if (y > box.maxY) box.maxY = y;
      }
    }
  }
  if (box.maxX < 0) return null;
  return { ...box, width: box.maxX - box.minX + 1, height: box.maxY - box.minY + 1 };
}

/** Scale a whole animation with nearest-neighbour pixels (no blur, no smoothing). */
export async function scaleFrames(frames, width, height) {
  // Resize the stacked frames as one tall image. Passing pageHeight here makes
  // libvips treat them as pages and shifts frame boundaries; a plain tall image
  // keeps every frame aligned after the uniform nearest-neighbour scale.
  const { data, info } = await sharp(frames.data, {
    raw: {
      width: frames.width,
      height: frames.height * frames.pages,
      channels: frames.channels,
    },
  })
    .resize(width, height * frames.pages, { kernel: "nearest" })
    .raw()
    .toBuffer({ resolveWithObject: true });

  return {
    data,
    width: info.width,
    height: info.height / frames.pages,
    pages: frames.pages,
    channels: info.channels,
    stride: info.channels,
    pageBytes: info.width * (info.height / frames.pages) * info.channels,
    delays: frames.delays,
  };
}

/** Encode transparent, frame-aligned RGBA pages as one looping GIF. */
export async function encodeFrames(frames) {
  return sharp(frames.data, {
    raw: {
      width: frames.width,
      height: frames.height * frames.pages,
      channels: frames.channels,
      pageHeight: frames.height,
    },
  })
    .gif({ loop: 0, delay: frames.delays })
    .toBuffer();
}

/** First frame as a transparent PNG poster. */
export async function posterFrom(frames) {
  return sharp(frames.data.subarray(0, frames.pageBytes), {
    raw: {
      width: frames.width,
      height: frames.height,
      channels: frames.channels,
    },
  })
    .png({ compressionLevel: 9 })
    .toBuffer();
}
