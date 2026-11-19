// The pet's pen: a borderless, always-on-top, per-pixel-alpha window that never takes focus.
// Each frame is drawn with GDI+ into a premultiplied DIB and pushed with UpdateLayeredWindow.

using System;
using System.Drawing;
using System.Drawing.Imaging;
using System.Windows.Forms;
using static DeskPet.Native;

namespace DeskPet
{
    sealed class PetWindow : NativeWindow, IDisposable
    {
        public int X, Y, W = 1, H = 1;          // physical pixels
        public double Scale = 1;                // device pixels per point
        public int MinX, MaxX;                  // how far a drag may move the window (screen pixels)
        public readonly PetView View;
        public event Action RightClicked;
        public event Action DpiChanged;

        IntPtr memDC, dib, oldBitmap, bits;
        Bitmap bitmap;
        int surfaceW, surfaceH;
        bool clickThrough = true, shown;

        bool mouseDown, dragMoved;
        int dragStartMouseX, dragStartX;

        public PetWindow(PetView view)
        {
            View = view;
            CreateHandle(new CreateParams
            {
                Caption = "DeskPet",
                Style = WS_POPUP,
                ExStyle = WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT,
                X = 0, Y = 0, Width = 1, Height = 1,
            });
        }

        public bool Shown
        {
            get => shown;
            set
            {
                if (value == shown) return;
                shown = value;
                if (value) Render();   // put the first frame in place before showing
                ShowWindow(Handle, value ? SW_SHOWNOACTIVATE : SW_HIDE);
            }
        }

        /// Only the pet itself catches clicks; the rest of the pen lets them through.
        public void SetClickThrough(bool through)
        {
            if (through == clickThrough) return;
            clickThrough = through;
            long ex = GetExStyle(Handle);
            SetExStyle(Handle, through ? ex | WS_EX_TRANSPARENT : ex & ~WS_EX_TRANSPARENT);
        }

        /// Keeps the pen above other always-on-top windows that came up since.
        public void KeepOnTop() =>
            SetWindowPos(Handle, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);

        void EnsureSurface()
        {
            if (surfaceW == W && surfaceH == H && bitmap != null) return;
            FreeSurface();
            memDC = CreateCompatibleDC(IntPtr.Zero);
            dib = CreateDib(W, H, out bits);
            oldBitmap = SelectObject(memDC, dib);
            bitmap = new Bitmap(W, H, W * 4, PixelFormat.Format32bppPArgb, bits);
            surfaceW = W;
            surfaceH = H;
        }

        void FreeSurface()
        {
            bitmap?.Dispose();
            bitmap = null;
            if (memDC != IntPtr.Zero)
            {
                SelectObject(memDC, oldBitmap);
                DeleteObject(dib);
                DeleteDC(memDC);
                memDC = IntPtr.Zero;
            }
        }

        public void Render()
        {
            EnsureSurface();
            using (var g = Graphics.FromImage(bitmap))
            {
                g.Clear(Color.Transparent);
                var canvas = new Canvas(g, Scale, H);
                // A nearly invisible fill so the whole sprite area takes clicks, not just its painted pixels.
                var r = View.PetRect;
                canvas.FillRect(View.petX + Art.Pixel, 0, r.Width, r.Height, Color.FromArgb(1, 0, 0, 0));
                View.Draw(canvas);
                g.Flush(System.Drawing.Drawing2D.FlushIntention.Sync);
            }
            var dst = new POINT { X = X, Y = Y };
            var size = new SIZE { CX = W, CY = H };
            var src = new POINT();
            var blend = new BLENDFUNCTION { BlendOp = 0, SourceConstantAlpha = 255, AlphaFormat = 1 /* AC_SRC_ALPHA */ };
            UpdateLayeredWindow(Handle, IntPtr.Zero, ref dst, ref size, memDC, ref src, 0, ref blend, ULW_ALPHA);
        }

        /// The pet's clickable rectangle on screen, in physical pixels.
        public Rectangle PetRectOnScreen()
        {
            var r = View.PetRect;
            int left = X + (int)Math.Round(r.X * Scale), right = X + (int)Math.Round((r.X + r.Width) * Scale);
            int bottom = Y + H, top = Y + H - (int)Math.Round(r.Height * Scale);
            return Rectangle.FromLTRB(left, top, right, bottom);
        }

        static int CursorX() { GetCursorPos(out var p); return p.X; }

        protected override void WndProc(ref Message m)
        {
            switch (m.Msg)
            {
                case WM_MOUSEACTIVATE:
                    m.Result = (IntPtr)MA_NOACTIVATE;
                    return;

                case WM_LBUTTONDOWN:
                    SetCapture(Handle);
                    mouseDown = true;
                    dragMoved = false;
                    dragStartMouseX = CursorX();
                    dragStartX = X;
                    View.BeginDrag();
                    return;

                case WM_MOUSEMOVE:
                    if (mouseDown)
                    {
                        int dx = CursorX() - dragStartMouseX;
                        if (Math.Abs(dx) > 3 * Scale) dragMoved = true;
                        if (dragMoved) X = Math.Min(Math.Max(dragStartX + dx, MinX), Math.Max(MinX, MaxX - W));
                    }
                    break;

                case WM_LBUTTONUP:
                    if (mouseDown)
                    {
                        mouseDown = false;
                        ReleaseCapture();
                        View.EndDrag(dragMoved);
                    }
                    return;

                case WM_CAPTURECHANGED:
                    if (mouseDown)
                    {
                        mouseDown = false;
                        View.EndDrag(dragMoved);
                    }
                    break;

                case WM_RBUTTONUP:
                    RightClicked?.Invoke();
                    return;

                case WM_DPICHANGED:
                    DpiChanged?.Invoke();
                    m.Result = IntPtr.Zero;
                    return;
            }
            base.WndProc(ref m);
        }

        public void Dispose()
        {
            FreeSurface();
            DestroyHandle();
        }
    }
}
