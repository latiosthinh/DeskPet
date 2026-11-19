// A native Win32 popup menu, so the tray menu looks like the rest of Windows 11
// (rounded corners, system font, light or dark to match the system).

using System;
using System.Collections.Generic;
using static DeskPet.Native;

namespace DeskPet
{
    sealed class NativeMenu : IDisposable
    {
        readonly IntPtr handle;
        readonly NativeMenu root;
        readonly Dictionary<int, Action> actions;
        readonly List<IntPtr> bitmaps;
        readonly List<NativeMenu> children = new List<NativeMenu>();
        int nextId = 1;
        uint count;

        public NativeMenu() : this(null) { }

        NativeMenu(NativeMenu parent)
        {
            handle = CreatePopupMenu();
            root = parent?.root ?? this;
            actions = root == this ? new Dictionary<int, Action>() : root.actions;
            bitmaps = root == this ? new List<IntPtr>() : root.bitmaps;
        }

        void Insert(ref MENUITEMINFO info)
        {
            info.cbSize = System.Runtime.InteropServices.Marshal.SizeOf(typeof(MENUITEMINFO));
            InsertMenuItem(handle, count++, true, ref info);
        }

        /// Adds an item. `radio` shows a dot instead of a tick (used for "on, but switched off elsewhere").
        public void Add(string text, Action action, bool check = false, bool enabled = true, bool bold = false,
                        IntPtr bitmap = default(IntPtr), bool radio = false)
        {
            int id = root.nextId++;
            if (action != null) actions[id] = action;
            uint state = (check ? MFS_CHECKED : 0) | (enabled ? 0 : MFS_DISABLED) | (bold ? MFS_DEFAULT : 0);
            var info = new MENUITEMINFO
            {
                fMask = MIIM_ID | MIIM_STRING | MIIM_STATE | MIIM_FTYPE | (bitmap != IntPtr.Zero ? MIIM_BITMAP : 0),
                fType = radio ? MFT_RADIOCHECK : 0,
                fState = state,
                wID = (uint)id,
                dwTypeData = text.Replace("&", "&&"),
                hbmpItem = bitmap,
            };
            if (bitmap != IntPtr.Zero) bitmaps.Add(bitmap);
            Insert(ref info);
        }

        public NativeMenu Submenu(string text)
        {
            var sub = new NativeMenu(this);
            children.Add(sub);
            var info = new MENUITEMINFO
            {
                fMask = MIIM_STRING | MIIM_SUBMENU,
                hSubMenu = sub.handle,
                dwTypeData = text.Replace("&", "&&"),
            };
            Insert(ref info);
            return sub;
        }

        public void Separator()
        {
            var info = new MENUITEMINFO { fMask = MIIM_FTYPE, fType = MFT_SEPARATOR };
            Insert(ref info);
        }

        /// Shows the menu at a screen point and returns the chosen item's action, if any.
        public Action Show(IntPtr owner, int x, int y)
        {
            SetForegroundWindow(owner);
            int id = TrackPopupMenuEx(handle, TPM_RIGHTBUTTON | TPM_BOTTOMALIGN | TPM_RETURNCMD | TPM_NONOTIFY, x, y, owner, IntPtr.Zero);
            PostMessage(owner, WM_NULL, IntPtr.Zero, IntPtr.Zero);            return id != 0 && actions.TryGetValue(id, out var a) ? a : null;
        }

        public void Dispose()
        {
            if (root != this) return;
            DestroyMenu(handle);   // also destroys the submenus
            foreach (var b in bitmaps) DeleteObject(b);
        }
    }
}
