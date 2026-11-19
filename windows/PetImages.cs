// Small pictures of a pet's face for the tray icon and the menu.

using System;
using System.Drawing;
using System.Drawing.Imaging;
using System.Linq;
using System.Runtime.InteropServices;
using static DeskPet.Native;

namespace DeskPet
{
    static class PetImages
    {
        /// The pet in its own colours, cropped to its drawn pixels and centred in a `size`×`size` square.
        /// Uses a whole number of device pixels per art pixel when that fills the square well, so it stays crisp.
        public static int[] Pixels(Pet pet, int size)
        {
            var all = pet.Rows(Eyes.Open);
            var used = Enumerable.Range(0, all.Length).Where(r => all[r].Any(ch => ch != '.')).ToArray();
            var rows = all.Skip(used.First()).Take(used.Last() - used.First() + 1).ToArray();
            var colsUsed = Enumerable.Range(0, 16).Where(c => rows.Any(r => r[c] != '.')).ToArray();
            int c0 = colsUsed.First(), cols = colsUsed.Last() - c0 + 1;

            double fit = (double)size / Math.Max(cols, rows.Length);
            double k = Math.Floor(fit) >= 1 && Math.Floor(fit) / fit >= 0.8 ? Math.Floor(fit) : fit;
            double ox = Math.Round((size - cols * k) / 2), oy = Math.Round((size - rows.Length * k) / 2);

            var px = new int[size * size];
            for (int r = 0; r < rows.Length; r++)
                for (int c = 0; c < cols; c++)
                {
                    if (!pet.Palette.TryGetValue(rows[r][c0 + c], out var color)) continue;
                    int x0 = (int)Math.Round(ox + c * k), x1 = (int)Math.Round(ox + (c + 1) * k);
                    int y0 = (int)Math.Round(oy + r * k), y1 = (int)Math.Round(oy + (r + 1) * k);
                    for (int y = Math.Max(0, y0); y < Math.Min(size, y1); y++)
                        for (int x = Math.Max(0, x0); x < Math.Min(size, x1); x++)
                            px[y * size + x] = color.ToArgb();
                }
            return px;
        }

        /// A 32 bpp DIB for a menu item (all pixels are fully opaque or fully clear, so no premultiplying is needed).
        public static IntPtr MenuBitmap(Pet pet, int size)
        {
            var px = Pixels(pet, size);
            var dib = CreateDib(size, size, out IntPtr bits);
            Marshal.Copy(px, 0, bits, px.Length);
            return dib;
        }

        public static Icon TrayIcon(Pet pet, int size)
        {
            var px = Pixels(pet, size);
            var color = CreateDib(size, size, out IntPtr bits);
            Marshal.Copy(px, 0, bits, px.Length);
            var mask = CreateBitmap(size, size, 1, 1, IntPtr.Zero);
            var info = new ICONINFO { fIcon = true, hbmColor = color, hbmMask = mask };
            var hicon = CreateIconIndirect(ref info);
            DeleteObject(color);
            DeleteObject(mask);
            // Icon.FromHandle doesn't own the handle; clone so the caller can dispose normally.
            using (var tmp = Icon.FromHandle(hicon))
            {
                var owned = (Icon)tmp.Clone();
                DestroyIcon(hicon);
                return owned;
            }
        }

        public static Bitmap Bitmap(Pet pet, int size)
        {
            var px = Pixels(pet, size);
            var bmp = new Bitmap(size, size, PixelFormat.Format32bppArgb);
            var data = bmp.LockBits(new Rectangle(0, 0, size, size), ImageLockMode.WriteOnly, PixelFormat.Format32bppArgb);
            Marshal.Copy(px, 0, data.Scan0, px.Length);
            bmp.UnlockBits(data);
            return bmp;
        }
    }
}
