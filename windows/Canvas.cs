// Drawing in the Mac version's coordinates: points (DIPs), y pointing up from the bottom of the pen.
// Everything is converted to device pixels here, and rectangle edges are rounded to whole pixels
// so neighbouring art pixels never leave seams.

using System;
using System.Collections.Generic;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Text;

namespace DeskPet
{
    sealed class Canvas
    {
        readonly Graphics g;
        public readonly double S;    // device pixels per point
        readonly int h;              // surface height in device pixels

        static readonly Dictionary<int, SolidBrush> brushes = new Dictionary<int, SolidBrush>();
        static readonly Dictionary<string, Font> fonts = new Dictionary<string, Font>();
        static readonly StringFormat typo = MakeFormat();

        static StringFormat MakeFormat()
        {
            var f = (StringFormat)StringFormat.GenericTypographic.Clone();
            f.FormatFlags |= StringFormatFlags.NoWrap | StringFormatFlags.MeasureTrailingSpaces;
            return f;
        }

        public Canvas(Graphics g, double scale, int heightPx)
        {
            this.g = g;
            S = scale;
            h = heightPx;
            g.SmoothingMode = SmoothingMode.AntiAlias;
            g.PixelOffsetMode = PixelOffsetMode.Half;   // whole-pixel rectangles stay sharp
            g.TextRenderingHint = TextRenderingHint.AntiAliasGridFit;
            g.CompositingQuality = CompositingQuality.HighSpeed;
        }

        public double Snap(double v) => Math.Round(v * S) / S;
        int DX(double x) => (int)Math.Round(x * S);
        int DY(double y) => h - (int)Math.Round(y * S);
        public PointF P(double x, double y) => new PointF((float)(x * S), (float)(h - y * S));

        static SolidBrush Brush(Color c)
        {
            int key = c.ToArgb();
            if (!brushes.TryGetValue(key, out var b)) { b = new SolidBrush(c); brushes[key] = b; }
            return b;
        }

        /// Fills the rectangle between two corners, each edge rounded to the nearest device pixel.
        public void FillEdges(double x0, double y0, double x1, double y1, Color c)
        {
            int l = DX(x0), r = DX(x1), t = DY(y1), b = DY(y0);
            if (r > l && b > t) g.FillRectangle(Brush(c), l, t, r - l, b - t);
        }

        public void FillRect(double x, double y, double w, double hgt, Color c) => FillEdges(x, y, x + w, y + hgt, c);

        public void FillPolygon(Color c, params PointF[] pts) => g.FillPolygon(Brush(c), pts);

        public static GraphicsPath RoundRect(RectangleF r, float radius)
        {
            float d = Math.Min(radius * 2, Math.Min(r.Width, r.Height));
            var path = new GraphicsPath();
            path.AddArc(r.X, r.Y, d, d, 180, 90);
            path.AddArc(r.Right - d, r.Y, d, d, 270, 90);
            path.AddArc(r.Right - d, r.Bottom - d, d, d, 0, 90);
            path.AddArc(r.X, r.Bottom - d, d, d, 90, 90);
            path.CloseFigure();
            return path;
        }

        /// A rectangle given in points (y up from the bottom), as device pixels.
        public RectangleF DeviceRect(double x, double y, double w, double hgt) =>
            new RectangleF((float)(x * S), (float)(h - (y + hgt) * S), (float)(w * S), (float)(hgt * S));

        public void FillRoundRect(double x, double y, double w, double hgt, double radius, Color c)
        {
            using (var path = RoundRect(DeviceRect(x, y, w, hgt), (float)(radius * S))) g.FillPath(Brush(c), path);
        }

        public void StrokeRoundRect(double x, double y, double w, double hgt, double radius, Color c, double width, float[] dash = null)
        {
            using (var path = RoundRect(DeviceRect(x, y, w, hgt), (float)(radius * S)))
            using (var pen = new Pen(c, (float)(width * S)))
            {
                if (dash != null) pen.DashPattern = dash;
                g.DrawPath(pen, path);
            }
        }

        /// Light falling off from `from` along the triangle, like the Mac's NSGradient fill.
        public void FillGradientTriangle(PointF a, PointF b, PointF c, Color start, Color end, bool reversed)
        {
            float minX = Math.Min(a.X, Math.Min(b.X, c.X)), maxX = Math.Max(a.X, Math.Max(b.X, c.X));
            float minY = Math.Min(a.Y, Math.Min(b.Y, c.Y)), maxY = Math.Max(a.Y, Math.Max(b.Y, c.Y));
            var rect = new RectangleF(minX - 1, minY - 1, maxX - minX + 2, maxY - minY + 2);
            using (var brush = new LinearGradientBrush(rect, reversed ? end : start, reversed ? start : end, LinearGradientMode.Horizontal))
                g.FillPolygon(brush, new[] { a, b, c });
        }

        Font FontFor(string family, double sizePt, FontStyle style)
        {
            string key = family + "|" + Math.Round(sizePt * S, 2) + "|" + style;
            if (!fonts.TryGetValue(key, out var f))
            {
                f = new Font(family, (float)(sizePt * S), style, GraphicsUnit.Pixel);
                fonts[key] = f;
            }
            return f;
        }

        /// Size of a line of text, in points.
        public SizeF Measure(string text, string family, double sizePt, FontStyle style)
        {
            var f = FontFor(family, sizePt, style);
            var sz = g.MeasureString(text, f, PointF.Empty, typo);
            return new SizeF((float)(sz.Width / S), (float)(f.GetHeight(g) / S));
        }

        /// Draws text whose line box has its bottom-left corner at (x, y), like NSAttributedString.draw(at:).
        public void Text(string text, string family, double sizePt, FontStyle style, Color c, double x, double y)
        {
            var f = FontFor(family, sizePt, style);
            float lineH = f.GetHeight(g);
            g.DrawString(text, f, Brush(c), (float)(x * S), DY(y) - lineH, typo);
        }
    }
}
