// Each pet's own tricks and typing activities, ported from main.swift.

using System;
using System.Collections.Generic;
using System.Linq;
using static DeskPet.Art;

namespace DeskPet
{
    sealed partial class PetView
    {
        static double Lerp(double a, double b, double t) => a + (b - a) * Math.Min(1, Math.Max(0, t));

        /// Centre x of a sprite column, honouring which way the pet faces.
        double CenterX(double col) => petX + (facingLeft ? 15.5 - col : col + 0.5) * Pixel;

        /// Velocity range pointing the way the pet faces.
        (double, double) Forward(double a, double b) => facingLeft ? (-b, -a) : (a, b);

        /// True once per trick for each id.
        bool Once(int id) => trickFlags.Add(id);

        /// True every `seconds` for each timer id.
        bool Every(int id, double seconds)
        {
            trickTimers.TryGetValue(id, out double last);
            if (now - last >= seconds) { trickTimers[id] = now; return true; }
            return false;
        }

        public void StartTrick(Trick t)
        {
            trick = t;
            lastTrick = t;
            trickFlags = new HashSet<int>();
            trickTimers = new Dictionary<int, double>();
            SetMode(Mode.Trick, t.Duration());
            nextTrickAt = now + t.Duration() + Rand(TrickGapMin, TrickGapMax);
            trickStartFacing = facingLeft;
            switch (t)
            {
                case Trick.Swim:
                    SetUpPool();
                    break;
                case Trick.Laser:
                    dotX = PetCenterX;
                    break;
                case Trick.Twin:
                    double roomRight = Width - (petX + PetSize), roomLeft = petX;
                    double side = roomRight >= roomLeft ? 1 : -1;
                    twinCenterX = Math.Min(Math.Max(PetCenterX + side * 68, PetSize / 2), Width - PetSize / 2);
                    break;
                case Trick.PopFood:
                    foodKind = rng.Next(TrickArt.Food.Length);
                    break;
            }
        }

        (double lo, double hi) SwimRange()
        {
            double lo = Math.Max(0, poolLeft - 8);
            double hi = Math.Max(lo, Math.Min(Width - PetSize, poolLeft + poolWidth - PetSize + 8));
            return (lo, hi);
        }

        Pt ButterflyPoint(double t, double d)
        {
            double lo = 16, hi = Math.Max(lo + 1, Width - 16);
            double mid = (lo + hi) / 2, half = Math.Max(0, (hi - lo) / 2 - 10);
            double x = mid + half * Math.Sin(t * 0.8) + 8 * Math.Sin(t * 3.1);
            double y = 46 + 14 * Math.Sin(t * 2.2);
            if (t > d - 1.5) y += (t - (d - 1.5)) * 70;   // flies away at the end
            return new Pt(x, y);
        }

        void Firework(Pt p)
        {
            for (int i = 0; i < 16; i++)
            {
                double a = i / 16.0 * 2 * Math.PI;
                double speed = Rand(34, 52);
                particles.Add(new Particle
                {
                    Text = i % 2 == 0 ? "✦" : "•", Color = Pick(SysColor.Sparkles), Size = 11,
                    X = p.X - 4, Y = p.Y, Vx = Math.Cos(a) * speed, Vy = Math.Sin(a) * speed,
                    Life = 1.3, MaxLife = 1.3, Ay = -24,
                });
            }
        }

        // MARK: Trick motion and particles

        void StepTrick(Trick tr, double dt)
        {
            double t = now - modeStart, d = modeUntil - modeStart;
            switch (tr)
            {
                case Trick.Firework:
                {
                    int cycle = (int)(t / 3.5);
                    double p = t % 3.5;
                    if (p < 1.2 && Every(1, 0.08))
                        Spawn("✦", SysColor.Orange, new Pt(CenterX(17) - 4, 22), size: 9, vx: (-10, 10), vy: (4, 14), life: 0.45);
                    if (p >= 2.0 && burstCycle != cycle)
                    {
                        burstCycle = cycle;
                        Firework(new Pt(CenterX(17), 98));
                    }
                    break;
                }

                case Trick.Flashlight:
                    facingLeft = (((int)(t / 2.7)) % 2 == 1) != trickStartFacing;
                    if (Every(1, 2.2)) SpawnAboveHead("?", SysColor.SecondaryLabel, 1);
                    break;

                case Trick.FlipEgg:
                    if (t % 2.0 < 0.7 && Every(1, 0.3))
                        Spawn("~", SysColor.SecondaryLabel, new Pt(CenterX(19) - 4, 8 * Pixel), size: 11, vx: (-3, 3), vy: (10, 16), life: 0.8);
                    break;

                case Trick.Campfire:
                    if (Every(1, 0.45))
                        Spawn("~", SysColor.SecondaryLabel, new Pt(CenterX(18) - 4, 6 * Pixel), size: 12, vx: (-4, 4), vy: (12, 20), life: 1.6);
                    if (Every(2, 0.3))
                        Spawn("·", SysColor.Orange, new Pt(CenterX(18) - 3, 5 * Pixel), size: 16, vx: (-10, 10), vy: (18, 32), life: 0.8);
                    break;

                case Trick.HideInWater:
                    if (t > 0.8 && t < 3.5 && Every(1, 0.4))
                        Spawn("°", SysColor.Blue, new Pt(PetCenterX + Rand(-12, 6), 6), size: 13, vx: (-3, 3), vy: (14, 22), life: 1.0);
                    if (t > 5.2 && Once(1)) facingLeft = !facingLeft;
                    if (t > 6.4 && Once(2)) facingLeft = !facingLeft;
                    if (t > 7.9 && Once(3))
                        for (int i = 0; i < 10; i++)
                            Spawn("•", SysColor.Blue, new Pt(PetCenterX - 4, 10), size: 10, vx: (-55, 55), vy: (35, 75), life: 0.9, ay: -170);
                    break;

                case Trick.SpitWater:
                    if (t % 0.9 < 0.1 && Once(100 + (int)(t / 0.9)))
                        for (int i = 0; i < 7; i++)
                            Spawn("•", SysColor.Blue, new Pt(CenterX(15), 5.5 * Pixel), size: 9, vx: Forward(55, 95), vy: (25, 55), life: 1.1, ay: -170);
                    break;

                case Trick.Swim:
                    SwimStep(dt);
                    break;

                case Trick.Bubbles:
                    if (Every(1, 0.45))
                        Spawn("○", SysColor.Teal, new Pt(CenterX(15), 5.5 * Pixel), size: Rand(9, 15), vx: Forward(8, 22), vy: (12, 24), life: 2.4);
                    break;

                case Trick.IceCream:
                    if (Every(1, 1.8)) SpawnAboveHead("♥", SysColor.Pink, 1);
                    break;

                case Trick.Twin:
                    facingLeft = twinCenterX < PetCenterX;
                    if (t > 0.6 && Once(1)) Say("pop!", 1);
                    if (t > 1.4 && Every(1, 1.1))
                        Spawn("♪", SysColor.Pink, new Pt((PetCenterX + twinCenterX) / 2 - 4, PetSize - 8), size: 13);
                    if (t > d - 0.6 && Once(2)) Say("pop!", 1);
                    break;

                case Trick.ColorShift:
                    if (Every(1, 0.22))
                        Spawn("✦", Pick(SysColor.Sparkles), new Pt(PetCenterX + Rand(-32, 26), Rand(6, 50)), size: 10,
                              vx: (-4, 4), vy: (4, 10), life: 0.8);
                    break;

                case Trick.PopFood:
                    if (t > 1.2 && Once(1)) Say("!", 0.8);
                    if (t > 2.0 && Once(2))
                    {
                        Say("a snack for you!", 3);
                        Spawn("✦", SysColor.Yellow, new Pt(CenterX(20) - 4, 12), size: 11, life: 0.9);
                    }
                    break;

                case Trick.GiveFlower:
                    if (Once(1)) Say("for you!", 4, BubbleIcons.Tulip);
                    if (t > 1 && Every(1, 1.1)) SpawnAboveHead("♥", SysColor.Pink, 1);
                    break;

                case Trick.Shower:
                {
                    double cloudLeft = PetCenterX - 20;
                    if (Every(1, 0.06))
                        Spawn("|", SysColor.Blue.WithAlpha(0.8), new Pt(cloudLeft + Rand(4, 34), 17 * Pixel - 6), size: 9,
                              vx: (0, 0), vy: (-95, -80), life: 0.65);
                    if (Every(2, 1.4)) Spawn("♪", SysColor.Pink, new Pt(PetCenterX + 22, 40), size: 13);
                    break;
                }

                case Trick.PlantSelf:
                    if ((t < 1 || (t > 7.5 && t < 8.3)) && Every(1, 0.08))
                        Spawn("·", SysColor.PlainBrown, new Pt(PetCenterX + Rand(-26, 20), 8), size: 16,
                              vx: (-40, 40), vy: (30, 60), life: 0.7, ay: -150);
                    if (t > 8.0 && Once(1)) Say("!", 1);
                    if (t > 8.3 && t < 9.3) facingLeft = (int)(t / 0.2) % 2 == 0;   // shakes off the dirt
                    break;

                case Trick.GrowUp:
                    if (((t > 0.5 && t < 3) || (t > 6 && t < 7)) && Every(1, 0.15))
                        Spawn("✦", SysColor.Yellow, new Pt(PetCenterX + Rand(-40, 34), Rand(4, 70)), size: 10,
                              vx: (-4, 4), vy: (4, 12), life: 0.8);
                    if (t > 3 && Once(1)) Say("so big!", 2);
                    break;

                case Trick.ChaseButterfly:
                {
                    var b = ButterflyPoint(t, d);
                    double behind = b.X > PetCenterX ? 16 : -16;
                    double target = Math.Min(Math.Max(b.X - PetSize / 2 - behind, 0), Width - PetSize);
                    chaseMoving = !MoveToward(target, 55, dt);
                    if (!chaseMoving) facingLeft = b.X < PetCenterX;
                    if (t > d - 1.5 && Once(1)) Say("bye!", 1.5);
                    break;
                }

                case Trick.Yarn:
                {
                    double ball = YarnX(t);
                    double behind = ball > PetCenterX ? 22 : -22;
                    double target = Math.Min(Math.Max(ball - PetSize / 2 - behind, 0), Width - PetSize);
                    chaseMoving = !MoveToward(target, 50, dt);
                    if (!chaseMoving) facingLeft = ball < PetCenterX;
                    if (t > 1 && Every(1, 2.4)) SpawnAboveHead("♪", SysColor.Pink, 1);
                    break;
                }

                case Trick.BoxNap:
                    if (t > 1.4 && t < d - 1.4 && Every(1, 1.3)) SpawnAboveHead("z", SysColor.SecondaryLabel, 1, life: 2.2);
                    if (t > d - 1.2 && Once(1)) Say("!", 1);
                    break;

                case Trick.Laser:
                {
                    // The dot jumps every 1.6 s; Mochi crouches, wiggles, then pounces on it.
                    int cycle = (int)(t / 1.6);
                    if (Once(200 + cycle) && cycle > 0) dotX = Rand(12, Math.Max(13, Width - 12));
                    if (t % 1.6 > 0.55)
                    {
                        double target = Math.Min(Math.Max(dotX - PetSize / 2 - (dotX > PetCenterX ? 10 : -10), 0), Width - PetSize);
                        chaseMoving = !MoveToward(target, 120, dt);
                    }
                    else
                    {
                        chaseMoving = false;
                        facingLeft = dotX < PetCenterX;
                    }
                    break;
                }

                case Trick.Fishing:
                {
                    double p = t % 3.0;
                    if (p > 2.0 && Once(300 + (int)(t / 3.0)))
                        for (int i = 0; i < 6; i++)
                            Spawn("•", SysColor.Blue, new Pt(CenterX(21) - 4, 12), size: 9, vx: (-30, 30), vy: (30, 55), life: 0.7, ay: -160);
                    if (p > 2.45 && Once(400 + (int)(t / 3.0))) Say("!", 0.6);
                    break;
                }
            }
        }

        // MARK: Typing activities

        /// Starts the pet's own activity for while you type (laptop, cooking, swimming, reading, digging).
        public void BeginTyping(double seconds = 3)
        {
            SetMode(Mode.Typing, seconds);
            trickTimers = new Dictionary<int, double>();
            trickFlags = new HashSet<int>();
            if (pet.Typing == TypingStyle.Swimming) SetUpPool();
        }

        void SetUpPool()
        {
            poolWidth = 30 * Pixel;
            poolLeft = Math.Min(Math.Max(PetCenterX - poolWidth / 2, 0), Math.Max(0, Width - poolWidth));
            targetX = SwimRange().hi;
        }

        void SwimStep(double dt)
        {
            var r = SwimRange();
            if (MoveToward(targetX, 22, dt))
                targetX = Math.Abs(targetX - r.lo) < 1 ? r.hi : r.lo;
            if (Every(1, 0.35))
                Spawn("~", SysColor.Blue, new Pt(PetCenterX + (facingLeft ? 22 : -30), 6), size: 12, vx: (-4, 4), vy: (2, 6), life: 0.8);
        }

        void StepTyping(double dt)
        {
            switch (pet.Typing)
            {
                case TypingStyle.Cooking:
                    if (Every(1, 0.35))
                        Spawn("~", SysColor.SecondaryLabel, new Pt(CenterX(20) - 4, 8 * Pixel), size: 11, vx: (-3, 3), vy: (10, 16), life: 0.8);
                    break;
                case TypingStyle.Swimming:
                    SwimStep(dt);
                    break;
                case TypingStyle.Digging:
                    if ((int)(clock / 0.3) % 2 == 1 && Every(1, 0.3))
                        for (int i = 0; i < 2; i++)
                            Spawn("·", SysColor.PlainBrown, new Pt(CenterX(18) - 3, 8), size: 16, vx: Forward(-10, 40), vy: (30, 55), life: 0.7, ay: -150);
                    break;
            }
        }

        /// A pet that doesn't crouch to sit (Puddle) holds the book lower, below its eyes.
        double BookLift => pet.SitBase == null ? 3 : 1;

        FreeSprite GlassesSprite()
        {
            double half = Pixel / 2, start = 10, width = TrickArt.Glasses[0].Length;
            var rows = facingLeft ? TrickArt.Glasses.Select(r => new string(r.Reverse().ToArray())).ToArray() : TrickArt.Glasses;
            double x = petX + (facingLeft ? 32 - start - width : start) * half;
            return new FreeSprite(rows, x, 11 * half, half);
        }

        Look TypingLook()
        {
            double t = now - modeStart;
            var eyes = BlinkEyes;
            var look = new Look(pet.Frame(eyes, Pose.Sit), pet.Palette);
            switch (pet.Typing)
            {
                case TypingStyle.Laptop:
                {
                    var prop = CurrentProp();
                    if (prop != null) look.Props.Add(prop.Value);
                    break;
                }
                case TypingStyle.Cooking:
                {
                    // An egg in a pan over a little campfire, tossed now and then.
                    double p = t % 2.5;
                    var egg = TrickArt.EggUp;
                    double lift = 7;
                    if (p > 1.6 && p < 2.3)
                    {
                        lift = 7 + 6 * Math.Sin(Math.PI * (p - 1.6) / 0.7);
                        egg = Alt(0.1) ? TrickArt.EggUp : TrickArt.EggDown;
                    }
                    look.Frame = pet.Frame(p > 2.3 ? Eyes.Happy : eyes, 0);
                    look.Props.Add(new Prop(Alt(0.25) ? TrickArt.FireA : TrickArt.FireB, 17, 0));
                    look.Props.Add(new Prop(TrickArt.Pan, 13, 5));
                    look.Props.Add(new Prop(egg, 18, lift));
                    break;
                }
                case TypingStyle.Swimming:
                    look.YOffset = -3;
                    look.Frame = pet.Frame(eyes, StepOf(0.25));
                    look.Free.Add(new FreeSprite(TrickArt.Pool((int)(poolWidth / Pixel)), poolLeft, 0));
                    break;
                case TypingStyle.ReadingGlasses:
                    look.Props.Add(new Prop(Props.Book(t % 3 < 0.25), 13, BookLift));
                    look.Free.Add(GlassesSprite());
                    break;
                case TypingStyle.Digging:
                {
                    bool down = !Alt(0.3);
                    int level = Math.Min(3, 1 + (int)(t / 2));
                    look.Frame = pet.Frame(eyes, down ? 1 : 0);
                    look.Props.Add(new Prop(TrickArt.Pile.Skip(TrickArt.Pile.Length - level).ToArray(), 15, 0));
                    look.Props.Add(new Prop(TrickArt.Shovel, 13, down ? 1 : 3));
                    break;
                }
            }
            return look;
        }

        double YarnX(double t)
        {
            double lo = 20, hi = Math.Max(lo + 1, Width - 20);
            double mid = (lo + hi) / 2, half = Math.Max(0, (hi - lo) / 2 - 6);
            return mid + half * Math.Sin(t * 0.9 + 0.6);
        }

        // MARK: Trick looks

        Look CurrentLook()
        {
            if (mode == Mode.Trick && trick != null && !dragging) return TrickLook(trick.Value);
            if (mode == Mode.Typing && !dragging) return TypingLook();
            var look = new Look(CurrentFrame(), pet.Palette);
            var prop = CurrentProp();
            if (prop != null) look.Props.Add(prop.Value);
            if (mode == Mode.Reading && pet.ReadsWithGlasses && !dragging) look.Free.Add(GlassesSprite());
            return look;
        }

        Look TrickLook(Trick tr)
        {
            double t = now - modeStart, d = modeUntil - modeStart;
            var eyes = BlinkEyes;
            var look = new Look(pet.Frame(eyes, Pose.Sit), pet.Palette);

            switch (tr)
            {
                case Trick.Firework:
                {
                    double p = t % 3.5;
                    look.Frame = pet.Frame(p > 2.0 ? Eyes.Happy : eyes, Pose.Sit);
                    if (p < 1.2) look.Props.Add(new Prop(TrickArt.Rocket, 16, 0));
                    else if (p < 2.0) look.Props.Add(new Prop(TrickArt.Rocket, 16, (p - 1.2) / 0.8 * 20));
                    break;
                }

                case Trick.Flashlight:
                    look.Frame = pet.Frame(eyes, 0);
                    look.Props.Add(new Prop(TrickArt.Flashlight, 14, 5));
                    look.BeamOrigin = new Pt(CenterX(18) + (facingLeft ? -2 : 2), 6.5 * Pixel);
                    look.BeamAngle = 0.05 + 0.22 * Math.Sin(t * 1.6);
                    break;

                case Trick.FlipEgg:
                {
                    double p = t % 2.0;
                    var egg = TrickArt.EggUp;
                    double lift = 7;
                    if (p >= 0.7 && p < 1.6)
                    {
                        lift = 7 + 10 * Math.Sin(Math.PI * (p - 0.7) / 0.9);
                        egg = Alt(0.1) ? TrickArt.EggUp : TrickArt.EggDown;
                    }
                    look.Frame = pet.Frame(p >= 1.6 ? Eyes.Happy : eyes, 0);
                    look.Props.Add(new Prop(TrickArt.Pan, 13, 5));
                    look.Props.Add(new Prop(egg, 18, lift));
                    break;
                }

                case Trick.Campfire:
                    look.Frame = pet.Frame(t > d - 2 ? Eyes.Happy : eyes, Pose.Sit);
                    look.Props.Add(new Prop(Alt(0.25) ? TrickArt.FireA : TrickArt.FireB, 15, 0));
                    look.Props.Add(new Prop(TrickArt.Stick(t > d * 0.5), 13, 3));
                    break;

                case Trick.HideInWater:
                {
                    // Sink, wait under water, peek out with just the eyes, then jump out with a splash.
                    double y;
                    if (t < 0.8) y = -12 * t / 0.8;
                    else if (t < 3.5) y = -12;
                    else if (t < 4.3) y = -12 + 8 * (t - 3.5) / 0.8;
                    else if (t < 7.5) y = -4;
                    else if (t < 7.9) y = -4 + 7 * (t - 7.5) / 0.4;
                    else if (t < 8.2) y = 3 - 3 * (t - 7.9) / 0.3;
                    else y = 0;
                    look.YOffset = y;
                    look.Frame = pet.Frame(t > 7.5 ? Eyes.Happy : eyes, 0);
                    if (t < 8.6) look.Props.Add(new Prop(TrickArt.Pool(20), -2, 0));
                    break;
                }

                case Trick.SpitWater:
                    look.Frame = pet.Frame(t % 0.9 < 0.2 ? Eyes.Happy : Eyes.Open, Pose.Sit);
                    break;

                case Trick.Swim:
                    look.YOffset = -3;
                    look.Frame = pet.Frame(Eyes.Open, StepOf(0.25));
                    look.Free.Add(new FreeSprite(TrickArt.Pool((int)(poolWidth / Pixel)), poolLeft, 0));
                    break;

                case Trick.Bubbles:
                    look.Frame = pet.Frame(Alt(0.45) ? Eyes.Happy : Eyes.Open, Pose.Sit);
                    break;

                case Trick.IceCream:
                    look.Frame = pet.Frame(Alt(0.5) ? Eyes.Happy : Eyes.Open, Pose.Sit);
                    look.Props.Add(new Prop(TrickArt.IceCream(1 - t / d), 14, 3));
                    break;

                case Trick.Twin:
                {
                    double start = 0.6, end = d - 1.2;
                    if (t < start)
                    {
                        look.Frame = pet.Frame(Eyes.Closed, Pose.Sit);
                        break;
                    }
                    double hop = Math.Sin((t - start) * 2 * Math.PI / 0.8);
                    look.Frame = pet.Frame(Eyes.Happy, 0);
                    look.YOffset = Math.Max(0, hop) * 3;
                    double scale = 1, cx = twinCenterX;
                    if (t < start + 0.5)
                    {
                        scale = 0.4 + 0.6 * ((t - start) / 0.5);
                        cx = Lerp(PetCenterX, twinCenterX, (t - start) / 0.5);
                    }
                    else if (t > end)
                    {
                        double q = (t - end) / 0.6;
                        scale = 1 - 0.6 * Math.Min(1, q);
                        cx = Lerp(twinCenterX, PetCenterX, q);
                    }
                    if (t < end + 0.6)
                    {
                        look.Twin = new Twin
                        {
                            Frame = pet.Frame(Eyes.Happy, 0), CenterX = cx, Scale = scale,
                            YOffset = Math.Max(0, -hop) * 3, FacingLeft = twinCenterX > PetCenterX,
                        };
                    }
                    if (t > start + 0.9 && t < end - 0.2)   // a ball passed back and forth
                    {
                        double f = ((t - start - 0.9) / 1.2) % 2;
                        double q = f < 1 ? f : 2 - f;
                        look.Free.Add(new FreeSprite(TrickArt.Ball, Lerp(PetCenterX, twinCenterX, q) - 4, 30 + 26 * Math.Sin(Math.PI * q)));
                    }
                    break;
                }

                case Trick.ColorShift:
                    look.Palette = ColorMath.HueShifted(pet.Palette, t / d, new[] { 'c', 'd', 'k' });
                    look.Frame = pet.Frame(Eyes.Happy, Alt(0.5) ? 0 : 1);
                    break;

                case Trick.PopFood:
                    if (t < 1.2)
                    {
                        look.Frame = pet.Frame(Eyes.Closed, StepOf(0.12));
                    }
                    else
                    {
                        look.Frame = pet.Frame(Eyes.Happy, Pose.Sit);
                        double q = Math.Min(1, (t - 1.2) / 0.8);
                        look.Props.Add(new Prop(TrickArt.Food[foodKind], 6 + 13 * q, (1 - q) * 10 + 12 * Math.Sin(Math.PI * q)));
                    }
                    break;

                case Trick.GiveFlower:
                    look.Frame = pet.Frame(Eyes.Happy, 0);
                    look.Props.Add(new Prop(TrickArt.Flower, 14, Alt(0.6) ? 4 : 5));
                    break;

                case Trick.Shower:
                    look.Frame = pet.Frame(Eyes.Happy, Alt(0.4) ? 0 : Pose.Sit);
                    look.Free.Add(new FreeSprite(TrickArt.Cloud, PetCenterX - 20, 17 * Pixel));
                    break;

                case Trick.PlantSelf:
                {
                    // Dig, sink until only the leaves show, nap, then pop out and shake off.
                    double y;
                    if (t < 1) y = 0;
                    else if (t < 2.5) y = -8 * (t - 1) / 1.5;
                    else if (t < 7.5) y = -8;
                    else if (t < 8.0) y = -8 + 10 * (t - 7.5) / 0.5;
                    else if (t < 8.3) y = 2 - 2 * (t - 8.0) / 0.3;
                    else y = 0;
                    look.YOffset = y;
                    look.Frame = pet.Frame(t > 2.5 && t < 7.5 ? Eyes.Closed : Eyes.Open, t < 1 ? StepOf(0.15) : 0);
                    if (t < 9.2) look.Props.Add(new Prop(TrickArt.Soil, -2, 0));
                    break;
                }

                case Trick.GrowUp:
                {
                    double scale;
                    if (t < 1) scale = 1;
                    else if (t < 3) scale = 1 + 0.125 * Math.Min(4, (int)((t - 1) / 0.5) + 1);
                    else if (t < 6) scale = 1.5;
                    else if (t < 7) scale = 1.5 - 0.125 * Math.Min(4, (int)((t - 6) / 0.25) + 1);
                    else scale = 1;
                    look.Scale = scale;
                    look.Frame = pet.Frame(scale > 1 ? Eyes.Happy : eyes, 0);
                    break;
                }

                case Trick.ChaseButterfly:
                {
                    look.Frame = pet.Frame(Eyes.Happy, chaseMoving ? StepOf(0.14) : 0);
                    var b = ButterflyPoint(t, d);
                    look.Free.Add(new FreeSprite(Alt(0.15) ? TrickArt.ButterflyA : TrickArt.ButterflyB, b.X - 7.5, b.Y, 3));
                    break;
                }

                case Trick.Yarn:
                {
                    look.Frame = pet.Frame(Eyes.Happy, chaseMoving ? StepOf(0.14) : 0);
                    double x = YarnX(t);
                    bool rolling = (int)(x / 6) % 2 == 0;
                    look.Free.Add(new FreeSprite(rolling ? TrickArt.YarnA : TrickArt.YarnB, x - 8, 0));
                    break;
                }

                case Trick.BoxNap:
                {
                    // Hop into the box, nap with just the head showing, then pop up.
                    double y = t < 0.8 ? 2 + 8 * Math.Sin(Math.PI * t / 0.8) : (t > d - 1.2 ? 3 : 2);
                    look.YOffset = y;
                    look.Frame = pet.Frame(t > 1.2 && t < d - 1.2 ? Eyes.Closed : Eyes.Happy, Pose.Sit);
                    look.Props.Add(new Prop(TrickArt.Box, -1, 0));
                    break;
                }

                case Trick.Laser:
                {
                    double p = t % 1.6;
                    if (chaseMoving)
                    {
                        look.Frame = pet.Frame(Eyes.Open, 0);
                        look.YOffset = 4 * Math.Sin(Math.PI * Math.Min(1, (p - 0.55) / 0.5));
                    }
                    else
                    {
                        look.Frame = pet.Frame(Eyes.Open, Alt(0.12) ? Pose.Sit : 0);   // wiggles before pouncing
                    }
                    look.Free.Add(new FreeSprite(TrickArt.LaserDot, dotX - 2.5, 0, 5));
                    break;
                }

                case Trick.Fishing:
                {
                    double p = t % 3.0;
                    look.Frame = pet.Frame(p > 2.3 ? Eyes.Happy : eyes, Pose.Sit);
                    look.Props.Add(new Prop(TrickArt.Pool(10), 15, 0));
                    look.Props.Add(new Prop(TrickArt.Rod, 13, 3));
                    if (p > 2.0 && p < 2.9)
                    {
                        double q = (p - 2.0) / 0.9;
                        look.Props.Add(new Prop(TrickArt.Fish, 18, 3 + 9 * Math.Sin(Math.PI * q)));
                    }
                    break;
                }
            }
            return look;
        }
    }
}
