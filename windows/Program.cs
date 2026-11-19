// DeskPet for Windows — a tiny pixel pet (Mochi, Ember, Pip, Puddle or Sprout) that lives on your taskbar.
//
// It wanders back and forth inside a limited "pen" above the bottom-right of the taskbar.
// When you type, it gets busy too. It reminds you to drink water, eat your main meals and
// stretch, and falls asleep when you're away.
//
// Drag the pet to move its home. Click it to pet it (or dismiss a reminder). Right-click it,
// or click its tray icon, for the menu.

using System;
using System.Threading;
using System.Windows.Forms;

namespace DeskPet
{
    static class Program
    {
        [STAThread]
        static void Main(string[] args)
        {
            using (var single = new Mutex(true, @"Local\DeskPet.SingleInstance", out bool first))
            {
                if (!first) return;   // already running
                Native.AllowDarkMenus();
                Application.EnableVisualStyles();
                Application.SetCompatibleTextRenderingDefault(false);
                Application.Run(new App(args));
                GC.KeepAlive(single);
            }
        }
    }
}
