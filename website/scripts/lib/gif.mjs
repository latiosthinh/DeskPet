// Tiny GIF reader/encoder helpers used by the build-time asset step.
// The reader only needs the canvas size and the total animation time.

import { SPRITE_BG } from "./sprite.mjs";

export function readGifInfo(buf) {
  if (buf.length < 13) throw new Error("GIF is too short");
  const signature = buf.toString("ascii", 0, 6);
  if (signature !== "GIF87a" && signature !== "GIF89a") throw new Error(`Not a GIF (${signature})`);

  const width = buf.readUInt16LE(6);
  const height = buf.readUInt16LE(8);
  if (!width || !height) throw new Error("GIF has no size");

  const packed = buf[10];
  let pos = 13;
  if (packed & 0x80) pos += 3 * 2 ** ((packed & 0x07) + 1);

  let frames = 0;
  let durationMs = 0;
  while (pos < buf.length) {
    const block = buf[pos++];
    if (block === 0x3b) break;
    if (block === 0x21) {
      const label = buf[pos++];
      if (label === 0xf9) {
        const size = buf[pos++];
        if (size < 4) throw new Error("Bad graphic control extension");
        const delay = buf.readUInt16LE(pos + 1);
        // Browsers treat a 0 delay as roughly 100ms.
        durationMs += (delay || 10) * 10;
        pos += size;
      }
      pos = skipSubBlocks(buf, pos);
    } else if (block === 0x2c) {
      if (pos + 9 > buf.length) throw new Error("Bad image descriptor");
      const imagePacked = buf[pos + 8];
      pos += 9;
      if (imagePacked & 0x80) pos += 3 * 2 ** ((imagePacked & 0x07) + 1);
      pos += 1; // LZW minimum code size
      pos = skipSubBlocks(buf, pos);
      frames++;
    } else {
      throw new Error(`Unknown GIF block 0x${block.toString(16)}`);
    }
  }

  if (!frames) throw new Error("GIF has no frames");
  return { width, height, frames, durationMs: durationMs || frames * 100 };
}

function skipSubBlocks(buf, pos) {
  while (pos < buf.length) {
    const length = buf[pos++];
    if (length === 0) break;
    pos += length;
    if (pos > buf.length) throw new Error("Truncated GIF block");
  }
  return pos;
}

/** Deterministic, tiny "art missing" GIF. Only used when a real GIF is absent (fresh checkout). */
export function makeFallbackGif(width = 64, height = 40) {
  const palette = [
    SPRITE_BG, // 0 frame (must match the paper background so it reads as background)
    [49, 46, 129], // 1 indigo outline
    [124, 58, 237], // 2 violet body
    [255, 255, 255], // 3 highlight
  ];
  const frames = [];
  for (const shift of [0, 1, 0]) {
    const px = Array.from({ length: width * height }, () => 0);
    const bodyLeft = Math.floor(width * 0.22);
    const bodyRight = width - bodyLeft;
    const bodyTop = 6;
    const bodyBottom = height - 4;
    for (let y = bodyTop; y < bodyBottom; y++) {
      for (let x = bodyLeft; x < bodyRight; x++) {
        const edge = x === bodyLeft || x === bodyRight - 1 || y === bodyTop || y === bodyBottom - 1;
        const corner =
          (x - bodyLeft < 3 || bodyRight - 1 - x < 3) &&
          (y - bodyTop < 3 || bodyBottom - 1 - y < 3);
        if (corner) continue;
        px[(y + shift) * width + x] = edge ? 1 : (x - bodyLeft) % 6 < 3 ? 2 : 1;
      }
    }
    for (const [x, y] of [
      [bodyLeft + 8, bodyTop + 8],
      [bodyRight - 11, bodyTop + 8],
    ]) {
      px[(y + shift) * width + x] = 3;
      px[(y + shift) * width + x + 1] = 3;
      px[(y + shift + 1) * width + x] = 3;
      px[(y + shift + 1) * width + x + 1] = 3;
    }
    frames.push({ pixels: px, delay: shift === 1 ? 350 : 500 });
  }
  return encodeGif(width, height, palette, frames, "BUILD-ONLY FALLBACK ART");
}

function encodeGif(width, height, palette, frames, comment) {
  const bytes = [];
  const u16 = (n) => bytes.push(n & 0xff, (n >> 8) & 0xff);
  bytes.push(...Buffer.from("GIF89a"));
  u16(width);
  u16(height);
  bytes.push(0xf1, 0, 0); // global colour table, 4 entries
  for (let i = 0; i < 4; i++) bytes.push(...(palette[i] || [0, 0, 0]));
  bytes.push(0x21, 0xff, 11, ...Buffer.from("NETSCAPE2.0"), 3, 1, 0, 0, 0);
  if (comment) {
    const text = Buffer.from(comment);
    for (let i = 0; i < text.length; i += 255) {
      const chunk = text.subarray(i, i + 255);
      bytes.push(0x21, 0xfe, chunk.length, ...chunk);
    }
    bytes.push(0);
  }
  for (const frame of frames) {
    bytes.push(0x21, 0xf9, 4, 0x04);
    u16(Math.max(1, Math.round(frame.delay / 10)));
    bytes.push(0, 0);
    bytes.push(0x2c);
    u16(0);
    u16(0);
    u16(width);
    u16(height);
    bytes.push(0);
    // 3-bit codes; a clear code every two pixels keeps the table from growing.
    bytes.push(2);
    const codes = [];
    frame.pixels.forEach((pixel, index) => {
      if (index % 2 === 0) codes.push(4);
      codes.push(pixel);
    });
    codes.push(5);
    const data = [];
    let acc = 0;
    let bits = 0;
    for (const code of codes) {
      acc |= code << bits;
      bits += 3;
      while (bits >= 8) {
        data.push(acc & 0xff);
        acc >>= 8;
        bits -= 8;
      }
    }
    if (bits) data.push(acc & 0xff);
    for (let i = 0; i < data.length; i += 255) {
      const chunk = data.slice(i, i + 255);
      bytes.push(chunk.length, ...chunk);
    }
    bytes.push(0);
  }
  bytes.push(0x3b);
  return Buffer.from(bytes);
}
