// The app: tray icon and menu, the frame timer, reminders and settings. Ported from AppDelegate in main.swift.

using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.Globalization;
using System.Linq;
using System.Windows.Forms;
using Microsoft.Win32;
using static DeskPet.Art;

namespace DeskPet
{
    sealed class App : ApplicationContext
    {
        readonly (double value, string title)[] rangeOptions =
        {
            (0, "Stay Put"), (80, "Tiny  (±80 px)"), (160, "Small  (±160 px)"),
            (300, "Medium  (±300 px)"), (500, "Large  (±500 px)"),
        };

        readonly PetWindow window;
        readonly PetView view;
        readonly NotifyIcon tray;
        readonly MenuOwner menuOwner = new MenuOwner();
        readonly Timer timer;

        double range = 80;
        int? homeX;                 // centre of the pen, physical screen pixels
        const double CornerMargin = 4;
        bool breaksOn = true, waterOn = true, mealsOn = true, unlocked, visible = true;
        bool fullscreenInFront;
        bool menuOpen;
        Rectangle lastWorkArea;
        int lastDpi;

        static double Uptime => Stopwatch.GetTimestamp() / (double)Stopwatch.Frequency;

        double lastTick = Uptime, streakStart = Uptime, lastKeyDown = -1000;
        double lastSlowCheck;
        /// Each interval is re-picked at random within its range, so reminders don't feel mechanical.
        const double BreakMin = 40 * 60, BreakMax = 60 * 60, WaterMin = 20 * 60, WaterMax = 30 * 60;
        double breakDue = PetView.Rand(BreakMin, BreakMax);
        const double BreakResetIdle = 5 * 60;
        double activeSinceWater;
        double waterDue = PetView.Rand(WaterMin, WaterMax);
        /// Main meals as (reminder, name, hour, minute). A meal reminder can fire once a day, up to 2 hours after its time.
        readonly (Reminder, string, int, int)[] meals = { (Reminder.Lunch, "lunch", 12, 0), (Reminder.Dinner, "dinner", 18, 30) };
        double lastMealCheck;

        readonly List<(double seconds, Action run)> demoQueue = new List<(double, Action)>();
        double nextDemo;

        public App(string[] args)
        {
            range = Settings.GetDouble("range") ?? 80;
            var savedHome = Settings.GetDouble("homeX");
            if (savedHome != null) homeX = (int)savedHome.Value;
            breaksOn = Settings.GetBool("breaks", true);
            waterOn = Settings.GetBool("water", true);
            mealsOn = Settings.GetBool("meals", true);
            visible = Settings.GetBool("visible", true);
            Login.RefreshPath();

            view = new PetView();
            unlocked = License.IsValid(Settings.GetString("license") ?? "");
            var saved = Pets.WithId(Settings.GetString("pet") ?? "mochi");
            view.pet = saved.Premium && !unlocked ? Pets.All[0] : saved;
            view.OnHomeMoved = HomeMoved;
            view.now = Uptime;

            window = new PetWindow(view);
            window.RightClicked += ShowMenu;
            window.DpiChanged += ScreensChanged;
            Layout();
            view.petX = (view.MinPetX + view.MaxPetX) / 2;
            view.Say("hi!", 2);
            window.Shown = visible;

            tray = new NotifyIcon { Visible = true };
            tray.MouseUp += (_, e) => { if (e.Button == MouseButtons.Left || e.Button == MouseButtons.Right) ShowMenu(); };
            UpdateTrayIcon();

            // `--demo` cycles through every activity, reminder and trick; `--tricks` plays only the pet's tricks.
            bool demo = args.Contains("--demo"), tricks = args.Contains("--tricks"), typing = args.Contains("--typing");
            if (demo || tricks || typing)
            {
                var v = view;
                if (demo)
                {
                    demoQueue.Add((7, () => v.BeginTyping(6)));
                    demoQueue.Add((7, () => v.SetMode(PetView.Mode.Reading, 6)));
                    demoQueue.Add((7, () => v.Remind(Reminder.Water)));
                    demoQueue.Add((7, () => v.Remind(Reminder.Lunch)));
                    demoQueue.Add((7, () => v.Remind(Reminder.Stretch)));
                    if (v.pet.Sweeps)
                        demoQueue.Insert(1, (7, () => { v.SetMode(PetView.Mode.Sweeping, 6); v.targetX = v.RandomTarget(v.petX, 50); }));
                }
                if (typing) demoQueue.Add((9, () => v.BeginTyping(8)));
                else foreach (var trick in v.pet.Tricks) { var tr = trick; demoQueue.Add((tr.Duration() + 0.5, () => v.StartTrick(tr))); }
                nextDemo = Uptime + 2;
            }

            timer = new Timer { Interval = 33 };
            timer.Tick += (_, __) => Tick();
            timer.Start();

            SystemEvents.DisplaySettingsChanged += OnSystemChange;
            SystemEvents.UserPreferenceChanged += OnSystemChange;
        }

        void OnSystemChange(object sender, EventArgs e) => ScreensChanged();

        Screen Screen => Screen.PrimaryScreen;

        /// Sizes the pen window to the roam range around home, standing on top of the taskbar.
        /// With no saved home, the pen sits in the bottom-right corner, above the clock.
        void Layout()
        {
            var wa = Screen.WorkingArea;
            lastWorkArea = wa;
            lastDpi = Native.DpiAt(Screen.Bounds.Left + 1, Screen.Bounds.Top + 1);
            double s = lastDpi / 96.0;
            double width = Math.Min(wa.Width / s, 2 * range + PetSize + 2 * PropRoom);
            double height = PetSize + 70;
            int w = (int)Math.Round(width * s), h = (int)Math.Round(height * s);
            int home = homeX ?? (wa.Right - (int)Math.Round(CornerMargin * s) - w / 2);
            int x = Math.Min(Math.Max(home - w / 2, wa.Left), wa.Right - w);

            window.Scale = s;
            window.X = x;
            window.Y = wa.Bottom - h;
            window.W = w;
            window.H = h;
            window.MinX = wa.Left;
            window.MaxX = wa.Right;
            view.Width = w / s;
            view.Height = h / s;
            view.range = range;
        }

        double PetScreenX => window.X + view.petX * window.Scale;

        void KeepPet(double screenX) =>
            view.petX = Math.Min(Math.Max((screenX - window.X) / window.Scale, view.MinPetX), view.MaxPetX);

        void HomeMoved()
        {
            homeX = window.X + window.W / 2;
            Settings.SetDouble("homeX", homeX.Value);
        }

        void ScreensChanged()
        {
            if (view.dragging) return;
            double absX = PetScreenX;
            int oldDpi = lastDpi;
            Layout();
            KeepPet(absX);
            if (lastDpi != oldDpi) UpdateTrayIcon();
        }

        /// Keys that count as typing: everything except mouse buttons and the modifier keys on their own.
        static readonly int[] typingKeys = Enumerable.Range(0x08, 0xFE - 0x08 + 1)
            .Where(vk => !(vk >= 0x10 && vk <= 0x12) && !(vk >= 0xA0 && vk <= 0xA5) && vk != 0x5B && vk != 0x5C)
            .ToArray();

        static bool AnyTypingKeyDown() => typingKeys.Any(vk => (Native.GetAsyncKeyState(vk) & 0x8001) != 0);

        void Tick()
        {
            double now = Uptime;
            double dt = Math.Min(now - lastTick, 0.1);
            lastTick = now;

            if (AnyTypingKeyDown()) lastKeyDown = now;
            double keyIdle = now - lastKeyDown;
            double idle = Math.Min(Native.SecondsSinceInput(), keyIdle);

            if (now - lastSlowCheck > 1)
            {
                lastSlowCheck = now;
                fullscreenInFront = Native.FullscreenAppInFront();
                if (Screen.WorkingArea != lastWorkArea || Native.DpiAt(Screen.Bounds.Left + 1, Screen.Bounds.Top + 1) != lastDpi)
                    ScreensChanged();
                if (window.Shown) window.KeepOnTop();
            }

            CheckReminders(now, dt, idle);

            bool show = visible && !fullscreenInFront;
            window.Shown = show;
            if (!show) return;

            if (demoQueue.Count > 0 && now >= nextDemo)
            {
                var next = demoQueue[0];
                demoQueue.RemoveAt(0);
                next.run();
                nextDemo = now + next.seconds;
            }

            Native.GetCursorPos(out var cursor);
            bool demoRunning = demoQueue.Count > 0 || now < nextDemo;   // demo ignores real typing and idleness
            view.Step(dt, now, demoRunning ? 0 : idle, demoRunning ? 99 : keyIdle,
                      new Pt((cursor.X - window.X) / window.Scale, (window.Y + window.H - cursor.Y) / window.Scale));

            // Only the pet itself catches clicks; the rest of the pen is click-through.
            window.SetClickThrough(!(view.dragging || window.PetRectOnScreen().Contains(cursor.X, cursor.Y)));
            window.Render();
        }

        // MARK: Reminders

        void CheckReminders(double now, double dt, double idle)
        {
            bool atDesk = idle < 60;
            if (atDesk) activeSinceWater += dt;
            if (idle > BreakResetIdle) streakStart = now;   // a 5-minute pause counts as a break
            if (!(visible && !fullscreenInFront && atDesk && view.CanRemind)) return;

            if (breaksOn && now - streakStart > breakDue)
            {
                streakStart = now;
                breakDue = PetView.Rand(BreakMin, BreakMax);
                view.Remind(Reminder.Stretch);
            }
            else if (waterOn && activeSinceWater > waterDue)
            {
                activeSinceWater = 0;
                waterDue = PetView.Rand(WaterMin, WaterMax);
                view.Remind(Reminder.Water);
            }
            else if (mealsOn && now - lastMealCheck > 30)
            {
                lastMealCheck = now;
                CheckMeals();
            }
        }

        void CheckMeals()
        {
            var date = DateTime.Now;
            var today = date.Date;
            foreach (var (reminder, key, hour, minute) in meals)
            {
                var start = today.AddHours(hour).AddMinutes(minute);
                var lastShown = DateTime.TryParseExact(Settings.GetString("lastMeal." + key), "yyyy-MM-dd",
                    CultureInfo.InvariantCulture, DateTimeStyles.None, out var d) ? d : DateTime.MinValue;
                if (date >= start && date < start.AddHours(2) && lastShown < today)
                {
                    Settings.SetString("lastMeal." + key, today.ToString("yyyy-MM-dd", CultureInfo.InvariantCulture));
                    view.Remind(reminder);
                    return;
                }
            }
        }

        // MARK: Menu

        void UpdateTrayIcon()
        {
            int size = (int)Math.Round(16 * lastDpi / 96.0);
            var old = tray.Icon;
            tray.Icon = PetImages.TrayIcon(view.pet, size);
            tray.Text = "DeskPet · " + view.pet.FirstName;
            old?.Dispose();
        }

        void ShowMenu()
        {
            if (menuOpen) return;
            menuOpen = true;
            Action chosen;
            using (var menu = BuildMenu())
            {
                Native.GetCursorPos(out var p);
                chosen = menu.Show(menuOwner.Handle, p.X, p.Y);
            }
            menuOpen = false;
            chosen?.Invoke();
        }

        NativeMenu BuildMenu()
        {
            var menu = new NativeMenu();
            int iconSize = (int)Math.Round(16 * lastDpi / 96.0);

            menu.Add("Show Pet", ToggleVisible, check: visible);

            var petMenu = menu.Submenu("Pet");
            foreach (var p in Pets.All)
            {
                var chosen = p;
                bool locked = p.Premium && !unlocked;
                bool current = p.Id == view.pet.Id;
                petMenu.Add(locked ? p.Name + "  🔒" : p.Name, () => PickPet(chosen), check: current, bold: current,
                            bitmap: PetImages.MenuBitmap(p, iconSize));
            }
            petMenu.Separator();
            if (unlocked) petMenu.Add("All Pets Unlocked", null, enabled: false);
            else petMenu.Add("Unlock All Pets…", UnlockFromMenu);

            if (view.pet.Tricks.Length > 0)
            {
                var trickMenu = menu.Submenu("Tricks");
                trickMenu.Add("Surprise Me", RandomTrick);
                trickMenu.Separator();
                foreach (var trick in view.pet.Tricks)
                {
                    var tr = trick;
                    trickMenu.Add(tr.Title(), () => { view.bubbleText = null; view.StartTrick(tr); });
                }
            }
            menu.Separator();

            var rangeMenu = menu.Submenu("Roam Range");
            for (int i = 0; i < rangeOptions.Length; i++)
            {
                int index = i;
                rangeMenu.Add(rangeOptions[i].title, () => PickRange(index), check: rangeOptions[i].value == range, radio: true);
            }
            menu.Add("Show Roam Area", () => view.ShowPen(3));
            menu.Add("Move Home to Corner", CenterHome);
            menu.Separator();

            var remindMenu = menu.Submenu("Reminders");
            remindMenu.Add("Drink Water (every 20–30 min)", ToggleWater, check: waterOn);
            remindMenu.Add("Main Meals (lunch 12:00, dinner 18:30)", ToggleMeals, check: mealsOn);
            remindMenu.Add("Stretch Break (every 40–60 min)", ToggleBreaks, check: breaksOn);

            // The setting can also change in Task Manager → Startup apps, so it's re-read every time.
            var login = Login.State;
            if (login == LoginState.TurnedOffInTaskManager)
                menu.Add("Open at Login (turned off in Task Manager)", ToggleLogin, check: true, radio: true);
            else
                menu.Add("Open at Login", ToggleLogin, check: login == LoginState.On);
            menu.Separator();

            menu.Add("Drag the pet to move its home · click to pet", null, enabled: false);
            menu.Add("Quit DeskPet", Quit);
            return menu;
        }

        void ToggleLogin()
        {
            try
            {
                if (Login.State == LoginState.On) Login.TurnOff(); else Login.TurnOn();
            }
            catch (Exception e)
            {
                MessageBox.Show("Couldn't change Open at Login.\n\n" + e.Message +
                                "\n\nYou can add DeskPet yourself in Settings → Apps → Startup.", "DeskPet",
                                MessageBoxButtons.OK, MessageBoxIcon.Warning);
            }
        }

        void ToggleVisible()
        {
            visible = !visible;
            Settings.SetBool("visible", visible);
            if (visible) view.Say("hi!", 1.5);
        }

        void PickRange(int index)
        {
            double absX = PetScreenX;
            range = rangeOptions[index].value;
            Settings.SetDouble("range", range);
            Layout();
            KeepPet(absX);
            if (view.mode == PetView.Mode.Walking || view.mode == PetView.Mode.Sweeping) view.SetMode(PetView.Mode.Idle);
            view.ShowPen(2.5);
        }

        void CenterHome()
        {
            double absX = PetScreenX;
            homeX = null;
            Settings.Remove("homeX");
            Layout();
            KeepPet(absX);
            view.ShowPen(2);
        }

        void ToggleBreaks()
        {
            breaksOn = !breaksOn;
            Settings.SetBool("breaks", breaksOn);
            streakStart = Uptime;
        }

        void ToggleWater()
        {
            waterOn = !waterOn;
            Settings.SetBool("water", waterOn);
            activeSinceWater = 0;
        }

        void ToggleMeals()
        {
            mealsOn = !mealsOn;
            Settings.SetBool("meals", mealsOn);
        }

        void RandomTrick()
        {
            view.bubbleText = null;
            var tricks = view.pet.Tricks;
            if (tricks.Length > 0) view.StartTrick(tricks[new Random().Next(tricks.Length)]);
        }

        void PickPet(Pet chosen)
        {
            if (chosen.Premium && !unlocked)
            {
                if (AskForCode(chosen)) SwitchTo(chosen);
                return;
            }
            SwitchTo(chosen);
        }

        void UnlockFromMenu()
        {
            if (AskForCode(null))
            {
                view.Petted();
                view.Say("thank you!", 2.5, BubbleIcons.Heart);
            }
        }

        /// Shows the code dialog until the code works or the person cancels. Returns true once unlocked.
        bool AskForCode(Pet wanting)
        {
            var names = Pets.All.Where(p => p.Premium).Select(p => p.FirstName).ToList();
            string list = string.Join(", ", names.Take(names.Count - 1)) + " and " + names.Last();
            string lead = wanting != null ? wanting.FirstName + " is a premium pet. " : "";
            using (var icon = PetImages.TrayIcon(wanting ?? view.pet, (int)Math.Round(32 * lastDpi / 96.0)))
            using (var dialog = new UnlockDialog($"{lead}Enter your DeskPet code to unlock {list}.", lastDpi / 96.0, icon))
            {
                if (dialog.ShowDialog() != DialogResult.OK) return false;
                Settings.SetString("license", dialog.Code);
                unlocked = true;
                return true;
            }
        }

        void SwitchTo(Pet chosen)
        {
            view.pet = chosen;
            Settings.SetString("pet", chosen.Id);
            UpdateTrayIcon();
            view.Petted();
            view.Say($"hi, I'm {chosen.FirstName}!", 2.5);
        }

        void Quit()
        {
            timer.Stop();
            SystemEvents.DisplaySettingsChanged -= OnSystemChange;
            SystemEvents.UserPreferenceChanged -= OnSystemChange;
            tray.Visible = false;
            tray.Dispose();
            window.Dispose();
            menuOwner.DestroyHandle();
            ExitThread();
        }

        /// An invisible window that owns the popup menu (a menu needs a window it can bring to the front).
        sealed class MenuOwner : NativeWindow
        {
            public MenuOwner()
            {
                CreateHandle(new CreateParams { Caption = "DeskPet menu", Style = Native.WS_POPUP, ExStyle = Native.WS_EX_TOOLWINDOW });
            }
        }
    }
}
