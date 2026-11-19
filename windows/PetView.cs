// The pet's behaviour and drawing, ported from PetView in main.swift.
// The pen (the pet's window) is `Width`×`Height` points; the pet moves inside it.
// Coordinates are points with y pointing up from the floor, as on the Mac.

using System;
using System.Collections.Generic;
using System.Drawing;
using System.Linq;
using static DeskPet.Art;

namespace DeskPet
{
    struct Pt
    {
        public double X, Y;
        public Pt(double x, double y) { X = x; Y = y; }
    }

    /// Something the pet holds or uses, drawn in front of it.
    /// `Col` is the sprite column of the prop's left edge (facing right); `Lift` is art pixels above the ground.
    struct Prop
    {
        public string[] Rows; public double Col, Lift;
        public Prop(string[] rows, double col, double lift) { Rows = rows; Col = col; Lift = lift; }
    }

    /// A grid drawn at a fixed spot in the pen (in points), not attached to the pet.
    struct FreeSprite
    {
        public string[] Rows; public double X, Y, Px;
        public FreeSprite(string[] rows, double x, double y, double px = Pixel) { Rows = rows; X = x; Y = y; Px = px; }
    }

    /// A second copy of the pet (Puddle's twin), centred at `CenterX`.
    sealed class Twin
    {
        public string[] Frame; public double CenterX, Scale, YOffset; public bool FacingLeft;
    }

    /// Everything drawn for one moment: the pet, how it is placed, and what is around it.
    sealed class Look
    {
        public string[] Frame;
        public Palette Palette;
        public double YOffset;          // art pixels; negative sinks into water or soil
        public double Scale = 1;
        public List<Prop> Props = new List<Prop>();
        public List<FreeSprite> Free = new List<FreeSprite>();
        public Twin Twin;
        public Pt? BeamOrigin; public double BeamAngle;   // flashlight light
        public Look(string[] frame, Palette palette) { Frame = frame; Palette = palette; }
    }

    sealed class Particle
    {
        public string Text; public Color Color; public double Size, X, Y, Vx, Vy, Life, MaxLife, Ay;
    }

    enum Reminder { Water, Lunch, Dinner, Stretch }

    sealed partial class PetView
    {
        public enum Mode { Idle, Walking, Sitting, Typing, Cheering, Sleeping, Sweeping, Reading, Drinking, Eating, Trick }

        static readonly Random rng = new Random();
        public static double Rand(double lo, double hi) => lo + rng.NextDouble() * (hi - lo);
        static T Pick<T>(IList<T> items) => items[rng.Next(items.Count)];

        public Mode mode = Mode.Idle;
        public double modeStart, modeUntil;
        public double petX, targetX;
        public bool facingLeft;
        public Pet pet = Pets.All[0];

        // Trick state
        public Trick? trick, lastTrick;
        HashSet<int> trickFlags = new HashSet<int>();
        Dictionary<int, double> trickTimers = new Dictionary<int, double>();
        bool trickStartFacing;
        int burstCycle = -1;
        double poolLeft, poolWidth, twinCenterX;
        int foodKind;
        bool chaseMoving;
        double bubbleScale = 1;
        double dotX;
        /// When the pet may do its next trick on its own.
        double? nextTrickAt;
        const double TrickGapMin = 4 * 60, TrickGapMax = 8 * 60;

        public double range = 80;
        public double Width, Height;     // the pen, in points

        public double clock, now;
        double lastPuff;
        readonly List<Particle> particles = new List<Particle>();
        public string bubbleText;
        string[] bubbleIcon;
        double bubbleUntil, showPenUntil;

        public bool dragging;
        public Action OnHomeMoved;

        const double WalkSpeed = 42;     // points per second
        const double SweepSpeed = 12;
        const double SleepAfter = 90;    // seconds of no input
        const double TypingLinger = 3;   // keep the laptop out this long after the last key

        public double MinPetX => PropRoom;
        public double MaxPetX => Math.Max(MinPetX, Width - PetSize - PropRoom);
        /// Clickable area of the sprite: x, y (up from the floor), width, height.
        public RectangleF PetRect => new RectangleF((float)(petX + Pixel), 0, (float)(PetSize - 2 * Pixel), (float)(PetSize - 2 * Pixel));
        public double PetCenterX => petX + PetSize / 2;
        /// Free to show a reminder: not asleep, not being dragged, nothing else being said.
        public bool CanRemind => mode != Mode.Sleeping && !dragging && bubbleText == null;

        // MARK: Behaviour

        public void SetMode(Mode m, double seconds = 0)
        {
            mode = m;
            modeStart = now;
            modeUntil = now + seconds;
        }

        public void Say(string text, double seconds, string[] icon = null)
        {
            bubbleText = text;
            bubbleIcon = icon;
            bubbleUntil = now + seconds;
        }

        public void ShowPen(double seconds) => showPenUntil = Math.Max(showPenUntil, now + seconds);

        /// Screen x of a sprite column, honouring which way the pet faces.
        public double XForColumn(double col) => petX + (facingLeft ? 15 - col : col) * Pixel;

        public void Spawn(string text, Color color, Pt at, double size = 14, (double, double)? vx = null, (double, double)? vy = null,
                          double life = 1.4, double ay = 0)
        {
            var (vx0, vx1) = vx ?? (-10, 10);
            var (vy0, vy1) = vy ?? (22, 36);
            particles.Add(new Particle
            {
                Text = text, Color = color, Size = size, X = at.X, Y = at.Y,
                Vx = Rand(vx0, vx1), Vy = Rand(vy0, vy1), Life = life, MaxLife = life, Ay = ay,
            });
        }

        public void SpawnAboveHead(string text, Color color, int count, double life = 1.4)
        {
            for (int i = 0; i < count; i++)
                Spawn(text, color, new Pt(PetCenterX + Rand(-14, 10), PetSize - 14 + Rand(-4, 4)), life: life);
        }

        public void Petted()
        {
            bubbleText = null;   // a click also dismisses a reminder
            SetMode(Mode.Cheering, 1.6);
            SpawnAboveHead("♥", SysColor.Pink, 3);
        }

        public void Remind(Reminder reminder)
        {
            switch (reminder)
            {
                case Reminder.Water:
                    SetMode(Mode.Drinking, 7);
                    Say("Time for some water", 9, BubbleIcons.Drop);
                    break;
                case Reminder.Lunch:
                    SetMode(Mode.Eating, 9);
                    Say("Lunch time! Go eat", 12, BubbleIcons.Rice);
                    break;
                case Reminder.Dinner:
                    SetMode(Mode.Eating, 9);
                    Say("Dinner time!", 12, BubbleIcons.Stew);
                    break;
                case Reminder.Stretch:
                    SetMode(Mode.Cheering, 3);
                    Say("Stretch break?", 9, BubbleIcons.Stretch);
                    break;
            }
        }

        public double RandomTarget(double x, double reach)
        {
            double lo = Math.Max(MinPetX, x - reach), hi = Math.Min(MaxPetX, x + reach);
            if (!(hi > lo)) return x;
            double t = x;
            for (int i = 0; i < 6; i++)
            {
                t = Rand(lo, hi);
                if (Math.Abs(t - x) > 20) break;
            }
            return t;
        }

        void DecideNext()
        {
            // A trick now and then: the first 1–3 minutes after launch, then 4–8 minutes apart.
            if (nextTrickAt == null) nextTrickAt = now + Rand(60, 180);
            if (pet.Tricks.Length > 0 && now >= nextTrickAt.Value)
            {
                var choices = pet.Tricks.Where(t => t != lastTrick).ToList();
                StartTrick(choices.Count > 0 ? Pick(choices) : pet.Tricks[0]);
                return;
            }
            bool canWalk = MaxPetX - MinPetX > 24;
            double r = rng.NextDouble();
            if (r < 0.45 && canWalk)
            {
                targetX = RandomTarget(petX, double.MaxValue);
                SetMode(Mode.Walking);
            }
            else if (r < 0.57 && pet.Sweeps)
            {
                SetMode(Mode.Sweeping, Rand(6, 10));
                targetX = RandomTarget(petX, 50);
            }
            else if (r < 0.70) SetMode(Mode.Reading, Rand(8, 14));
            else if (r < 0.90) SetMode(Mode.Sitting, Rand(3, 8));
            else SetMode(Mode.Idle, Rand(2, 4));
        }

        public void Step(double dt, double now, double idle, double keyIdle, Pt mouse)
        {
            this.now = now;
            clock += dt;

            foreach (var p in particles)
            {
                p.X += p.Vx * dt;
                p.Y += p.Vy * dt;
                p.Vy += p.Ay * dt;
                p.Life -= dt;
            }
            particles.RemoveAll(p => p.Life <= 0);
            if (bubbleText != null && now > bubbleUntil) bubbleText = null;

            if (dragging) return;

            // React to what the human is doing.
            bool busyWithReminder = mode == Mode.Drinking || mode == Mode.Eating || mode == Mode.Cheering;
            if (mode != Mode.Sleeping && idle > SleepAfter)
            {
                SetMode(Mode.Sleeping);
            }
            else if (mode == Mode.Sleeping && idle < 1)
            {
                SetMode(Mode.Idle, 1.5);
                Say("!", 1.2);
            }
            else if (mode != Mode.Sleeping && !busyWithReminder && keyIdle < 0.6)
            {
                if (mode != Mode.Typing) BeginTyping();   // pull out the laptop, pan, shovel…
                modeUntil = now + TypingLinger;
            }

            switch (mode)
            {
                case Mode.Walking:
                    if (MoveToward(targetX, WalkSpeed, dt)) SetMode(Mode.Idle, Rand(2, 5));
                    break;

                case Mode.Sweeping:
                    if (MoveToward(targetX, SweepSpeed, dt)) targetX = RandomTarget(petX, 50);
                    if (now - lastPuff > 0.35)
                    {
                        lastPuff = now;
                        Spawn("·", SysColor.Brown.WithAlpha(0.8), new Pt(XForColumn(19), 0),
                              size: 16, vx: (-14, 14), vy: (6, 16), life: 0.9);
                    }
                    if (now >= modeUntil) DecideNext();
                    break;

                case Mode.Sleeping:
                    if (now - lastPuff > 1.3)
                    {
                        lastPuff = now;
                        SpawnAboveHead("z", SysColor.SecondaryLabel, 1, life: 2.2);
                    }
                    break;

                case Mode.Eating:
                    if (now - lastPuff > 0.8)
                    {
                        lastPuff = now;
                        Spawn("~", SysColor.SecondaryLabel, new Pt(XForColumn(18), 18), size: 12, vx: (-3, 3), vy: (10, 16), life: 1.2);
                    }
                    if (now >= modeUntil) PetDoneEating();
                    break;

                case Mode.Drinking:
                    if (now >= modeUntil) PetDoneEating();
                    break;

                case Mode.Trick:
                    if (trick != null) StepTrick(trick.Value, dt);
                    if (now >= modeUntil) { trick = null; DecideNext(); }
                    break;

                case Mode.Typing:
                    StepTyping(dt);
                    if (now >= modeUntil) DecideNext();
                    break;

                default:   // idle, sitting, cheering, reading
                    if (now >= modeUntil) DecideNext();
                    break;
            }
            petX = Math.Min(Math.Max(petX, MinPetX), MaxPetX);   // never leave the pen

            // Look at the cursor when it comes close (not while using a prop).
            if ((mode == Mode.Idle || mode == Mode.Sitting || mode == Mode.Cheering)
                && mouse.Y < Height + 160 && Math.Abs(mouse.X - PetCenterX) < 220 && Math.Abs(mouse.X - PetCenterX) > 6)
            {
                facingLeft = mouse.X < PetCenterX;
            }
        }

        void PetDoneEating()
        {
            SetMode(Mode.Cheering, 1.2);
            SpawnAboveHead("♥", SysColor.Pink, 1);
        }

        /// Walks toward x; returns true on arrival.
        bool MoveToward(double x, double speed, double dt)
        {
            double dx = x - petX;
            double stride = speed * dt;
            if (Math.Abs(dx) <= stride) { petX = x; return true; }
            facingLeft = dx < 0;
            petX += dx > 0 ? stride : -stride;
            return false;
        }

        int StepOf(double period) => (int)(clock / period) % 2;
        bool Alt(double period) => (int)(clock / period) % 2 == 0;
        Eyes BlinkEyes => clock % 4.2 < 0.14 ? Eyes.Closed : Eyes.Open;

        string[] CurrentFrame()
        {
            var eyes = BlinkEyes;
            bool beat = (int)(clock / 0.16) % 2 == 0;
            double t = now - modeStart;
            if (dragging) return pet.Frame(Eyes.Happy, StepOf(0.1));
            switch (mode)
            {
                case Mode.Walking: return pet.Frame(Eyes.Open, StepOf(0.18));
                case Mode.Idle: return pet.Frame(eyes, 0);
                case Mode.Sitting:
                case Mode.Typing:
                case Mode.Reading: return pet.Frame(eyes, Pose.Sit);
                case Mode.Cheering: return pet.Frame(Eyes.Happy, beat ? 0 : Pose.Sit);
                case Mode.Sleeping: return pet.Frame(Eyes.Closed, Pose.Sit);
                case Mode.Sweeping: return pet.Frame(eyes, StepOf(0.3));
                case Mode.Drinking: return pet.Frame(IsSipping(t) ? Eyes.Happy : Eyes.Open, Pose.Sit);
                case Mode.Eating: return pet.Frame(StepOf(0.4) == 0 ? Eyes.Happy : Eyes.Open, Pose.Sit);
                default: return pet.Frame(eyes, Pose.Sit);
            }
        }

        static bool IsSipping(double t) => t % 1.6 > 1.0;

        Prop? CurrentProp()
        {
            if (dragging) return null;
            double t = now - modeStart;
            double progress = modeUntil > modeStart ? Math.Min(1, t / (modeUntil - modeStart)) : 0;
            switch (mode)
            {
                case Mode.Typing:
                    if (t < 0.3) return new Prop(Props.LaptopClosed, 14, 0);
                    return new Prop(Props.LaptopOpen(Alt(0.12), Alt(0.5)), 14, 0);
                case Mode.Sweeping:
                    return new Prop(Props.Broom(Alt(0.3)), 14, 0);
                case Mode.Reading:
                    return new Prop(Props.Book(t % 3 < 0.25), 13, BookLift);
                case Mode.Drinking:
                    return new Prop(Props.Cup(1 - progress), 14, IsSipping(t) ? 5 : 3);
                case Mode.Eating:
                    return new Prop(Props.Bowl(1 - progress), 14, 0);
                default:
                    return null;
            }
        }

        // MARK: Drawing

        const string SymbolFont = "Segoe UI Symbol";
        const string UiFont = "Segoe UI Semibold";

        public void Draw(Canvas c)
        {
            if (dragging || now < showPenUntil) DrawPen(c);

            var look = CurrentLook();
            bubbleScale = look.Scale;   // a grown pet gets its speech bubble above its head
            if (look.BeamOrigin != null) DrawBeam(c, look.BeamOrigin.Value, look.BeamAngle);

            var colors = new Palette(PropPalette);
            if (look.Palette.TryGetValue(pet.Paw, out var paw)) colors['w'] = paw; else colors.Remove('w');
            colors['k'] = look.Palette['k'];

            if (look.Twin != null)
                DrawSprite(c, look.Twin.Frame, look.Palette, look.Twin.CenterX, look.Twin.Scale, look.Twin.YOffset, look.Twin.FacingLeft);
            DrawSprite(c, look.Frame, look.Palette, PetCenterX, look.Scale, look.YOffset, facingLeft);
            foreach (var prop in look.Props)
                DrawGrid(c, prop.Rows, colors, c.Snap(petX), prop.Col, prop.Lift * Pixel, Pixel, facingLeft);
            foreach (var f in look.Free)
                DrawGrid(c, f.Rows, colors, c.Snap(f.X), 0, c.Snap(f.Y), f.Px, false);

            foreach (var p in particles)
            {
                double alpha = Math.Max(0, Math.Min(1, p.Life / p.MaxLife * 1.5));
                c.Text(p.Text, SymbolFont, p.Size, FontStyle.Bold, p.Color.WithAlpha(alpha), p.X, p.Y);
            }

            if (bubbleText != null) DrawBubble(c, bubbleText, bubbleIcon);
        }

        /// Draws the 16×16 pet sprite scaled around its centre, standing on the ground.
        /// Art row 13 (feet) sits on the ground line, so the sprite starts 2 art pixels below it.
        static void DrawSprite(Canvas c, string[] rows, Palette palette, double centerX, double scale, double yOffset, bool mirrored)
        {
            double px = Math.Max(0.5, Math.Round(Pixel * scale * 2) / 2);
            DrawGrid(c, rows, palette, c.Snap(centerX - 8 * px), 0, c.Snap(-2 * px + yOffset * Pixel), px, mirrored);
        }

        /// Draws pixel rows whose left edge is sprite column `col` and whose bottom row sits at `bottomY`.
        /// Mirrored grids flip around the 16-column sprite, so props stay in front of a pet facing left.
        static void DrawGrid(Canvas c, string[] rows, Palette palette, double x, double col, double bottomY, double px, bool mirrored)
        {
            for (int r = 0; r < rows.Length; r++)
            {
                double y = bottomY + (rows.Length - 1 - r) * px;
                string row = rows[r];
                for (int i = 0; i < row.Length; i++)
                {
                    if (!palette.TryGetValue(row[i], out var color)) continue;
                    double spriteCol = col + i;
                    double drawCol = mirrored ? 15 - spriteCol : spriteCol;
                    c.FillEdges(x + drawCol * px, y, x + (drawCol + 1) * px, y + px, color);
                }
            }
        }

        void DrawBeam(Canvas c, Pt p, double angle)
        {
            double dir = facingLeft ? -1 : 1;
            const double length = 96, spread = 0.34;
            var a = c.P(p.X, p.Y);
            var b = c.P(p.X + dir * length * Math.Cos(angle - spread), p.Y + length * Math.Sin(angle - spread));
            var d = c.P(p.X + dir * length * Math.Cos(angle + spread), p.Y + length * Math.Sin(angle + spread));
            var light = Color.FromArgb(128, 255, 237, 140);
            c.FillGradientTriangle(a, b, d, light, light.WithAlpha(0), facingLeft);
        }

        void DrawPen(Canvas c)
        {
            double x = MinPetX, y = 1, w = MaxPetX - MinPetX + PetSize, h = PetSize - 4;
            c.FillRoundRect(x, y, w, h, 10, SysColor.Teal.WithAlpha(0.12));
            c.StrokeRoundRect(x, y, w, h, 10, SysColor.Teal.WithAlpha(0.8), 2, new[] { 3f, 2f });

            string label = range == 0 ? "stays put" : $"roams ±{(int)range} px";
            var size = c.Measure(label, UiFont, 10, FontStyle.Regular);
            c.Text(label, UiFont, 10, FontStyle.Regular, SysColor.Teal, x + w / 2 - size.Width / 2, y + h + 2);
        }

        void DrawBubble(Canvas c, string text, string[] icon)
        {
            const double iconPx = 2;
            var size = c.Measure(text, UiFont, 12, FontStyle.Regular);
            double iconW = icon == null ? 0 : icon.Max(r => r.Length) * iconPx;
            double iconH = icon == null ? 0 : icon.Length * iconPx;
            double contentW = size.Width + (icon == null ? 0 : 5 + iconW);
            double contentH = Math.Max(size.Height, iconH);

            double bx = PetCenterX - contentW / 2 - 8, by = PetSize * bubbleScale + 4;
            double bw = contentW + 16, bh = contentH + 8;
            bx = Math.Min(Math.Max(bx, 2), Width - bw - 2);

            c.FillRoundRect(bx, by, bw, bh, 8, Color.White);
            c.FillPolygon(Color.White, c.P(PetCenterX - 5, by + 1), c.P(PetCenterX, by - 6), c.P(PetCenterX + 5, by + 1));
            c.StrokeRoundRect(bx, by, bw, bh, 8, Color.FromArgb(51, 51, 51), 1.5);
            c.Text(text, UiFont, 12, FontStyle.Regular, Color.Black, bx + 8, by + 4 + (contentH - size.Height) / 2);
            if (icon != null)
            {
                double ix = c.Snap(bx + 8 + size.Width + 5), iy = c.Snap(by + 4 + (contentH - iconH) / 2);
                var colors = new Palette(PropPalette) { ['k'] = PropPalette['k'] };
                for (int r = 0; r < icon.Length; r++)
                    for (int i = 0; i < icon[r].Length; i++)
                    {
                        if (!colors.TryGetValue(icon[r][i], out var color)) continue;
                        double y0 = iy + (icon.Length - 1 - r) * iconPx;
                        c.FillEdges(ix + i * iconPx, y0, ix + (i + 1) * iconPx, y0 + iconPx, color);
                    }
            }
        }

        // MARK: Mouse (the window moves itself; it tells the view when a drag starts and ends)

        public void BeginDrag() => dragging = true;

        public void EndDrag(bool moved)
        {
            dragging = false;
            if (moved)
            {
                SetMode(Mode.Idle, 2);
                ShowPen(1.5);
                Say("new home!", 1.5);
                OnHomeMoved?.Invoke();
            }
            else
            {
                Petted();
            }
        }
    }
}
