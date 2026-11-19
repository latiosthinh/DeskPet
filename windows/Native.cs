// Win32 calls DeskPet needs: layered windows, idle time, key state, native menus, DPI and fullscreen detection.

using System;
using System.Runtime.InteropServices;

namespace DeskPet
{
    static class Native
    {
        public const int WS_POPUP = unchecked((int)0x80000000);
        public const int WS_EX_TOPMOST = 0x00000008;
        public const int WS_EX_TRANSPARENT = 0x00000020;
        public const int WS_EX_TOOLWINDOW = 0x00000080;
        public const int WS_EX_LAYERED = 0x00080000;
        public const int WS_EX_NOACTIVATE = 0x08000000;
        public const int GWL_EXSTYLE = -20;

        public const int WM_NULL = 0x0000;
        public const int WM_MOUSEACTIVATE = 0x0021;
        public const int WM_MOUSEMOVE = 0x0200;
        public const int WM_LBUTTONDOWN = 0x0201;
        public const int WM_LBUTTONUP = 0x0202;
        public const int WM_RBUTTONUP = 0x0205;
        public const int WM_CAPTURECHANGED = 0x0215;
        public const int WM_DPICHANGED = 0x02E0;
        public const int MA_NOACTIVATE = 3;

        public const int SW_HIDE = 0;
        public const int SW_SHOWNOACTIVATE = 4;
        public static readonly IntPtr HWND_TOPMOST = new IntPtr(-1);
        public const uint SWP_NOSIZE = 0x0001, SWP_NOMOVE = 0x0002, SWP_NOACTIVATE = 0x0010,
                          SWP_FRAMECHANGED = 0x0020, SWP_NOZORDER = 0x0004;

        [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
        [StructLayout(LayoutKind.Sequential)] public struct SIZE { public int CX, CY; }

        [StructLayout(LayoutKind.Sequential, Pack = 1)]
        public struct BLENDFUNCTION { public byte BlendOp, BlendFlags, SourceConstantAlpha, AlphaFormat; }

        [StructLayout(LayoutKind.Sequential)]
        public struct BITMAPINFOHEADER
        {
            public int biSize, biWidth, biHeight;
            public short biPlanes, biBitCount;
            public int biCompression, biSizeImage, biXPelsPerMeter, biYPelsPerMeter, biClrUsed, biClrImportant;
        }

        [StructLayout(LayoutKind.Sequential)] struct LASTINPUTINFO { public int cbSize; public uint dwTime; }

        [DllImport("user32.dll", SetLastError = true)]
        public static extern bool UpdateLayeredWindow(IntPtr hwnd, IntPtr hdcDst, ref POINT pptDst, ref SIZE psize,
            IntPtr hdcSrc, ref POINT pptSrc, int crKey, ref BLENDFUNCTION pblend, int dwFlags);
        public const int ULW_ALPHA = 2;

        [DllImport("user32.dll", EntryPoint = "GetWindowLong")] static extern int GetWindowLong32(IntPtr h, int i);
        [DllImport("user32.dll", EntryPoint = "GetWindowLongPtr")] static extern IntPtr GetWindowLongPtr64(IntPtr h, int i);
        [DllImport("user32.dll", EntryPoint = "SetWindowLong")] static extern int SetWindowLong32(IntPtr h, int i, int v);
        [DllImport("user32.dll", EntryPoint = "SetWindowLongPtr")] static extern IntPtr SetWindowLongPtr64(IntPtr h, int i, IntPtr v);

        public static long GetExStyle(IntPtr h) =>
            IntPtr.Size == 8 ? GetWindowLongPtr64(h, GWL_EXSTYLE).ToInt64() : GetWindowLong32(h, GWL_EXSTYLE);
        public static void SetExStyle(IntPtr h, long v)
        {
            if (IntPtr.Size == 8) SetWindowLongPtr64(h, GWL_EXSTYLE, new IntPtr(v));
            else SetWindowLong32(h, GWL_EXSTYLE, (int)v);
        }

        [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
        [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
        [DllImport("user32.dll")] public static extern IntPtr SetCapture(IntPtr h);
        [DllImport("user32.dll")] public static extern bool ReleaseCapture();
        [DllImport("user32.dll")] public static extern bool GetCursorPos(out POINT p);
        [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
        [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, int msg, IntPtr w, IntPtr l);
        [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr SendMessage(IntPtr h, int msg, IntPtr w, string l);
        [DllImport("user32.dll")] public static extern short GetAsyncKeyState(int vk);
        [DllImport("user32.dll")] static extern bool GetLastInputInfo(ref LASTINPUTINFO info);
        [DllImport("user32.dll")] public static extern bool DestroyIcon(IntPtr h);

        /// Seconds since the last keyboard or mouse input anywhere in the session.
        public static double SecondsSinceInput()
        {
            var info = new LASTINPUTINFO { cbSize = Marshal.SizeOf(typeof(LASTINPUTINFO)) };
            if (!GetLastInputInfo(ref info)) return 0;
            return unchecked((uint)Environment.TickCount - info.dwTime) / 1000.0;
        }

        [DllImport("gdi32.dll")] public static extern IntPtr CreateCompatibleDC(IntPtr hdc);
        [DllImport("gdi32.dll")] public static extern bool DeleteDC(IntPtr hdc);
        [DllImport("gdi32.dll")] public static extern IntPtr SelectObject(IntPtr hdc, IntPtr obj);
        [DllImport("gdi32.dll")] public static extern bool DeleteObject(IntPtr obj);
        [DllImport("gdi32.dll")] public static extern IntPtr CreateBitmap(int w, int h, uint planes, uint bpp, IntPtr bits);
        [DllImport("gdi32.dll")]
        public static extern IntPtr CreateDIBSection(IntPtr hdc, ref BITMAPINFOHEADER bmi, uint usage, out IntPtr bits, IntPtr section, uint offset);

        /// A top-down 32 bpp DIB section; `bits` points at its pixels.
        public static IntPtr CreateDib(int w, int h, out IntPtr bits)
        {
            var bmi = new BITMAPINFOHEADER
            {
                biSize = Marshal.SizeOf(typeof(BITMAPINFOHEADER)), biWidth = w, biHeight = -h,
                biPlanes = 1, biBitCount = 32,
            };
            return CreateDIBSection(IntPtr.Zero, ref bmi, 0, out bits, IntPtr.Zero, 0);
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct ICONINFO { public bool fIcon; public int xHotspot, yHotspot; public IntPtr hbmMask, hbmColor; }
        [DllImport("user32.dll")] public static extern IntPtr CreateIconIndirect(ref ICONINFO info);

        // DPI
        [DllImport("user32.dll")] static extern IntPtr MonitorFromPoint(POINT pt, uint flags);
        [DllImport("shcore.dll")] static extern int GetDpiForMonitor(IntPtr mon, int type, out uint x, out uint y);

        /// DPI of the monitor containing a point (physical pixels), 96 if it can't be read.
        public static int DpiAt(int x, int y)
        {
            try
            {
                var mon = MonitorFromPoint(new POINT { X = x, Y = y }, 2 /* nearest */);
                if (GetDpiForMonitor(mon, 0, out uint dx, out _) == 0 && dx > 0) return (int)dx;
            }
            catch (DllNotFoundException) { }
            catch (EntryPointNotFoundException) { }
            return 96;
        }

        // Fullscreen apps and presentations
        [DllImport("shell32.dll")] static extern int SHQueryUserNotificationState(out int state);

        /// True while a fullscreen app, a fullscreen Direct3D game or presentation mode is in front.
        public static bool FullscreenAppInFront()
        {
            try
            {
                if (SHQueryUserNotificationState(out int s) != 0) return false;
                return s == 2 || s == 3 || s == 4;   // QUNS_BUSY, QUNS_RUNNING_D3D_FULL_SCREEN, QUNS_PRESENTATION_MODE
            }
            catch { return false; }
        }

        // Native popup menus
        public const uint MIIM_STATE = 0x1, MIIM_ID = 0x2, MIIM_SUBMENU = 0x4, MIIM_STRING = 0x40,
                          MIIM_BITMAP = 0x80, MIIM_FTYPE = 0x100;
        public const uint MFT_SEPARATOR = 0x800, MFT_RADIOCHECK = 0x200;
        public const uint MFS_DISABLED = 0x3, MFS_CHECKED = 0x8, MFS_DEFAULT = 0x1000;
        public const uint TPM_RIGHTBUTTON = 0x2, TPM_BOTTOMALIGN = 0x20, TPM_NONOTIFY = 0x80, TPM_RETURNCMD = 0x100;

        [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
        public struct MENUITEMINFO
        {
            public int cbSize;
            public uint fMask, fType, fState, wID;
            public IntPtr hSubMenu, hbmpChecked, hbmpUnchecked, dwItemData;
            public string dwTypeData;
            public uint cch;
            public IntPtr hbmpItem;
        }

        [DllImport("user32.dll")] public static extern IntPtr CreatePopupMenu();
        [DllImport("user32.dll")] public static extern bool DestroyMenu(IntPtr menu);
        [DllImport("user32.dll", CharSet = CharSet.Unicode)]
        public static extern bool InsertMenuItem(IntPtr menu, uint item, bool byPosition, ref MENUITEMINFO info);
        [DllImport("user32.dll")]
        public static extern int TrackPopupMenuEx(IntPtr menu, uint flags, int x, int y, IntPtr hwnd, IntPtr tpm);

        // Dark context menus: undocumented uxtheme exports that Windows' own apps use.
        [DllImport("uxtheme.dll", EntryPoint = "#135")] static extern int SetPreferredAppMode(int mode);
        [DllImport("uxtheme.dll", EntryPoint = "#136")] static extern void FlushMenuThemes();

        /// Lets popup menus follow the system's dark mode. Does nothing where unsupported.
        public static void AllowDarkMenus()
        {
            try { SetPreferredAppMode(1 /* AllowDark */); FlushMenuThemes(); }
            catch { }
        }
    }
}
