// Pixel art and pet designs, ported from main.swift. Grids are 16×16, facing right, row 0 at the top.

using System;
using System.Collections.Generic;
using System.Drawing;
using System.Linq;

namespace DeskPet
{
    sealed class Palette : Dictionary<char, Color>
    {
        public Palette() { }
        public Palette(IDictionary<char, Color> other) : base(other) { }
    }

    enum Eyes { Open, Closed, Happy }
    enum TypingStyle { Laptop, Cooking, Swimming, ReadingGlasses, Digging }

    /// A pose is a walking-feet row (0 or 1) when standing, or `Pose.Sit`.
    static class Pose { public const int Sit = -1; }

    static class Art
    {
        public const double Pixel = 4;                 // points per art pixel
        public const double PetSize = 16 * Pixel;      // sprite is 16×16 art pixels
        public const double PropRoom = 36;             // space kept inside the pen for props held in front
        public const string EmptyRow = "................";

        public static Color Rgb(double r, double g, double b) =>
            Color.FromArgb(255, (int)Math.Round(r * 255), (int)Math.Round(g * 255), (int)Math.Round(b * 255));

        /// Colours for props. "w" (the paw on the laptop) comes from the pet's own body colour.
        public static readonly Palette PropPalette = new Palette
        {
            ['k'] = Rgb(0.20, 0.16, 0.24),  // outline
            ['g'] = Rgb(0.66, 0.68, 0.74),  // laptop
            ['b'] = Rgb(0.45, 0.78, 1.00),  // laptop screen
            ['B'] = Rgb(0.85, 0.97, 1.00),  // laptop screen, bright line
            ['n'] = Rgb(0.58, 0.38, 0.22),  // broom handle
            ['y'] = Rgb(0.96, 0.78, 0.36),  // broom straw
            ['r'] = Rgb(0.87, 0.34, 0.38),  // book cover
            ['s'] = Rgb(0.85, 0.82, 0.78),  // turning page
            ['c'] = Rgb(0.55, 0.82, 1.00),  // water
            ['o'] = Rgb(0.93, 0.46, 0.32),  // bowl
            ['i'] = Rgb(1.00, 1.00, 1.00),  // rice, book pages, egg white, marshmallow
            ['d'] = Rgb(0.30, 0.30, 0.35),  // frying pan
            ['C'] = Rgb(0.80, 0.94, 1.00),  // water surface
            ['P'] = Rgb(1.00, 0.62, 0.76),  // petals, butterfly, ice cream
            ['G'] = Rgb(0.33, 0.64, 0.30),  // stems
            ['N'] = Rgb(0.40, 0.26, 0.15),  // dark soil
            ['W'] = Rgb(0.95, 0.96, 0.99),  // cloud
            ['T'] = Rgb(0.82, 0.64, 0.42),  // cardboard
            ['U'] = Rgb(0.66, 0.49, 0.31),  // cardboard tape and flaps
        };
    }

    /// Apple's light-mode system colours, which the Mac version uses for particles.
    static class SysColor
    {
        public static readonly Color Pink = Color.FromArgb(255, 45, 85);
        public static readonly Color Brown = Color.FromArgb(162, 132, 94);
        public static readonly Color Yellow = Color.FromArgb(255, 204, 0);
        public static readonly Color Orange = Color.FromArgb(255, 149, 0);
        public static readonly Color Teal = Color.FromArgb(48, 176, 199);
        public static readonly Color Purple = Color.FromArgb(175, 82, 222);
        public static readonly Color Green = Color.FromArgb(52, 199, 89);
        public static readonly Color Blue = Color.FromArgb(0, 122, 255);
        public static readonly Color PlainBrown = Color.FromArgb(153, 102, 51);   // NSColor.brown
        public static readonly Color SecondaryLabel = Color.FromArgb(110, 110, 118);
        public static readonly Color[] Sparkles = { Yellow, Pink, Orange, Teal, Purple, Green };
    }

    /// A pet design on the 16×16 grid, facing right. Sprite column 14 is the front outline,
    /// where props are held; eyes are drawn on rows 6–7 at `EyeCols`.
    sealed class Pet
    {
        public string Id, Name;
        public Palette Palette;
        public string[] Body;              // 13 rows, eyes left out
        public string[] Feet;              // two walking rows
        public string SitBase;             // a pet without legs keeps this row under it when it sits
        public char Paw = 'w';
        public int[] EyeCols = { 9, 12 };
        public Trick[] Tricks = new Trick[0];
        public TypingStyle Typing = TypingStyle.Laptop;   // what the pet does while you type
        public bool Sweeps, ReadsWithGlasses;
        public bool Premium;               // needs an unlock code

        public string FirstName => Name.Split(' ')[0];

        public string[] Rows(Eyes eyes)
        {
            var grid = Body.Select(r => r.ToCharArray()).ToArray();
            foreach (var c in EyeCols)
            {
                switch (eyes)
                {
                    case Eyes.Open: grid[6][c] = 'k'; grid[7][c] = 'k'; break;
                    case Eyes.Closed: grid[7][c - 1] = 'k'; grid[7][c] = 'k'; break;
                    case Eyes.Happy: grid[6][c] = 'k'; grid[7][c - 1] = 'k'; grid[7][c + 1] = 'k'; break;
                }
            }
            return grid.Select(g => new string(g)).ToArray();
        }

        public string[] Frame(Eyes eyes, int pose)
        {
            var rows = Rows(eyes).ToList();
            int drop = 0;
            if (pose >= 0) rows.Add(Feet[pose % 2]);
            else if (SitBase != null) rows.Add(SitBase);
            else drop = 1;
            while (rows.Count < 16) rows.Add(Art.EmptyRow);
            for (int i = 0; i < drop; i++) { rows.Insert(0, Art.EmptyRow); rows.RemoveAt(rows.Count - 1); }
            return rows.ToArray();
        }
    }

    static class Pets
    {
        static Color Rgb(double r, double g, double b) => Art.Rgb(r, g, b);

        public static readonly Pet[] All =
        {
            new Pet
            {
                Id = "mochi", Name = "Mochi",
                Palette = new Palette { ['k'] = Rgb(0.20, 0.16, 0.24), ['w'] = Rgb(0.99, 0.95, 0.89),
                                        ['s'] = Rgb(0.89, 0.82, 0.76), ['p'] = Rgb(1.00, 0.62, 0.70) },
                Body = new[] { "................", "...kk......kk...", "..kppk....kppk..", "..kpwkkkkkkwpk..",
                               ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.",
                               ".kwwwwwwpwwwwpk.", ".kswwwwwwwwwwwk.", ".ksswwwwwwwwwwk.", "..ksssswwwwwwk..",
                               "...kkkkkkkkkk..." },
                Feet = new[] { "..kwk.....kwk...", "....kwk.kwk....." },
                Tricks = new[] { Trick.Yarn, Trick.BoxNap, Trick.Laser, Trick.Fishing }, Sweeps = true,
            },
            new Pet
            {
                Id = "ember", Name = "Ember the fox",
                Palette = new Palette { ['k'] = Rgb(0.23, 0.14, 0.13), ['w'] = Rgb(0.95, 0.53, 0.23),
                                        ['s'] = Rgb(0.83, 0.42, 0.16), ['c'] = Rgb(1.00, 0.95, 0.86) },
                Body = new[] { ".....k.....k....", "....kwk...kwk...", "....kwwk.kwwk...", "...kwwwkkkwwwk..",
                               "...kwwwwwwwwwwk.", "...kwwwwwwwwwwk.", ".k.kwwwwwwwwwwk.", "kckkwwwwwwwwwwk.",
                               "kwkkwwwwccccckk.", "kwwkswwwcccccck.", "kwwksswwwccccck.", ".kkkssswwwwwwk..",
                               "....kkkkkkkkk..." },
                Feet = new[] { "....kwk...kwk...", ".....kwk.kwk...." },
                Tricks = new[] { Trick.Firework, Trick.Flashlight, Trick.FlipEgg, Trick.Campfire },
                Typing = TypingStyle.Cooking, Premium = true,
            },
            new Pet
            {
                Id = "pip", Name = "Pip the penguin",
                Palette = new Palette { ['k'] = Rgb(0.12, 0.13, 0.20), ['b'] = Rgb(0.24, 0.29, 0.43), ['w'] = Rgb(1.00, 1.00, 1.00),
                                        ['o'] = Rgb(1.00, 0.65, 0.24), ['p'] = Rgb(1.00, 0.62, 0.70) },
                Body = new[] { "................", "................", ".....kkkkkk.....", "...kkbbbbbbkk...",
                               "..kbbbbbbwwwwk..", ".kbbbbbbwwwwwwk.", ".kbbbbbwwwwwwwk.", ".kbbbbbwwwwwwwko",
                               ".kbbbbbwwwwwwpk.", ".kbbkbbwwwwwwwk.", ".kbbbkbwwwwwwwk.", "..kbbbkwwwwwwk..",
                               "...kkkkkkkkkk..." },
                Feet = new[] { "...kook..kook...", "....kook.kook..." },
                Tricks = new[] { Trick.HideInWater, Trick.SpitWater, Trick.Swim, Trick.Bubbles },
                Typing = TypingStyle.Swimming, Premium = true,
            },
            new Pet
            {
                Id = "puddle", Name = "Puddle the slime",
                Palette = new Palette { ['k'] = Rgb(0.14, 0.31, 0.42), ['c'] = Rgb(0.56, 0.85, 0.95), ['H'] = Rgb(1.00, 1.00, 1.00),
                                        ['d'] = Rgb(0.39, 0.71, 0.86), ['p'] = Rgb(1.00, 0.62, 0.70) },
                Body = new[] { "................", "................", "................", ".....kkkkkk.....",
                               "...kkcccccckk...", "..kcHcccccccck..", ".kcHcccccccccck.", ".kcccccccccccck.",
                               ".kccccccpccccpk.", ".kcccccccccccck.", ".kdccccccccccck.", ".kddcccccccccck.",
                               ".kddddcccccccck." },
                Feet = new[] { "..kkkkkkkkkkkk..", ".kkkkkkkkkkkkkk." },
                SitBase = "..kkkkkkkkkkkk..", Paw = 'c',
                Tricks = new[] { Trick.IceCream, Trick.Twin, Trick.ColorShift, Trick.PopFood },
                Typing = TypingStyle.ReadingGlasses, ReadsWithGlasses = true, Premium = true,
            },
            new Pet
            {
                Id = "sprout", Name = "Sprout the seedling",
                Palette = new Palette { ['k'] = Rgb(0.14, 0.25, 0.16), ['w'] = Rgb(0.71, 0.88, 0.54), ['s'] = Rgb(0.55, 0.76, 0.40),
                                        ['L'] = Rgb(0.36, 0.70, 0.32), ['S'] = Rgb(0.25, 0.54, 0.23), ['p'] = Rgb(1.00, 0.62, 0.70) },
                Body = new[] { "..kkk...kkk.....", ".kLLLk.kLLLk....", "..kLLLkLLLk.....", "..kkkkSkkkkkkk..",
                               ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.",
                               ".kwwwwwwpwwwwpk.", ".kswwwwwwwwwwwk.", ".ksswwwwwwwwwwk.", "..ksssswwwwwwk..",
                               "...kkkkkkkkkk..." },
                Feet = new[] { "..kwk.....kwk...", "....kwk.kwk....." },
                Tricks = new[] { Trick.GiveFlower, Trick.Shower, Trick.PlantSelf, Trick.GrowUp, Trick.ChaseButterfly },
                Typing = TypingStyle.Digging, Premium = true,
            },
        };

        public static Pet WithId(string id) => All.FirstOrDefault(p => p.Id == id) ?? All[0];
    }

    /// Props sit just in front of the body (sprite column 14 is the body's front outline).
    static class Props
    {
        public static readonly string[] LaptopClosed = { "kkkkkkkkk.", "kgggggggk.", "kkkkkkkkk." };

        /// Side view: keyboard on the floor, lid tilted back with its glowing screen facing the pet.
        public static string[] LaptopOpen(bool pawUp, bool flicker) => new[]
        {
            "........kk",
            ".......kbk",
            flicker ? ".......kBk" : ".......kbk",
            "......kbk.",
            pawUp ? "kwk...kbk." : ".kwk..kBk.",
            "kkkkkkkkk.",
            "kgggggggk.",
            "kkkkkkkkk.",
        };

        public static string[] Broom(bool swish)
        {
            var handle = new[] { "n......", ".n.....", "..n....", "...n..." };
            var straw = new[] { "..kkk..", ".kyyyk.", "kyyyyyk" };
            return handle.Concat(swish ? straw.Select(s => "." + s.Substring(0, s.Length - 1)) : straw).ToArray();
        }

        /// An open book held up in front.
        public static string[] Book(bool flip)
        {
            var page = flip ? "kiiksik" : "kiikiik";
            return new[] { ".kk.kk.", page, "kiikiik", "kiikiik", "krrrrrk" };
        }

        /// `left` goes from 1 (full) to 0 (empty).
        public static string[] Cup(double left)
        {
            int full = Math.Max(0, Math.Min(3, (int)Math.Ceiling(left * 3)));
            var rows = new List<string> { "k...k" };
            for (int i = 0; i < 3; i++) rows.Add(i >= 3 - full ? "kccck" : "k...k");
            rows.Add(".kkk.");
            return rows.ToArray();
        }

        public static string[] Bowl(double left)
        {
            var rice = left > 0.5 ? new[] { "..iiiii..", ".iiiiiii." }
                     : left > 0.05 ? new[] { ".........", ".iiiiiii." }
                     : new[] { ".........", "........." };
            return rice.Concat(new[] { "koooooook", ".koooook." }).ToArray();
        }
    }

    enum Trick
    {
        Firework, Flashlight, FlipEgg, Campfire,              // Ember
        HideInWater, SpitWater, Swim, Bubbles,                // Pip
        IceCream, Twin, ColorShift, PopFood,                  // Puddle
        GiveFlower, Shower, PlantSelf, GrowUp, ChaseButterfly,// Sprout
        Yarn, BoxNap, Laser, Fishing,                         // Mochi
    }

    static class TrickInfo
    {
        public static double Duration(this Trick t)
        {
            switch (t)
            {
                case Trick.Firework: return 8;
                case Trick.Flashlight: return 8;
                case Trick.FlipEgg: return 9;
                case Trick.Campfire: return 10;
                case Trick.HideInWater: return 9.5;
                case Trick.SpitWater: return 6;
                case Trick.Swim: return 10;
                case Trick.Bubbles: return 7;
                case Trick.IceCream: return 9;
                case Trick.Twin: return 11;
                case Trick.ColorShift: return 8;
                case Trick.PopFood: return 7;
                case Trick.GiveFlower: return 7;
                case Trick.Shower: return 9;
                case Trick.PlantSelf: return 10;
                case Trick.GrowUp: return 9;
                case Trick.ChaseButterfly: return 12;
                case Trick.Yarn: return 10;
                case Trick.BoxNap: return 10;
                case Trick.Laser: return 10;
                case Trick.Fishing: return 11;
            }
            return 8;
        }

        public static string Title(this Trick t)
        {
            switch (t)
            {
                case Trick.Firework: return "Light Fireworks";
                case Trick.Flashlight: return "Explore with a Flashlight";
                case Trick.FlipEgg: return "Flip an Egg";
                case Trick.Campfire: return "Toast a Marshmallow";
                case Trick.HideInWater: return "Hide in the Water";
                case Trick.SpitWater: return "Spit Water";
                case Trick.Swim: return "Go for a Swim";
                case Trick.Bubbles: return "Blow Bubbles";
                case Trick.IceCream: return "Eat Ice Cream";
                case Trick.Twin: return "Split in Two and Play";
                case Trick.ColorShift: return "Change Colour";
                case Trick.PopFood: return "Pop Out a Snack";
                case Trick.GiveFlower: return "Give You a Flower";
                case Trick.Shower: return "Take a Shower";
                case Trick.PlantSelf: return "Plant Itself";
                case Trick.GrowUp: return "Grow Up";
                case Trick.ChaseButterfly: return "Chase a Butterfly";
                case Trick.Yarn: return "Play with Yarn";
                case Trick.BoxNap: return "Nap in a Box";
                case Trick.Laser: return "Chase a Laser Dot";
                case Trick.Fishing: return "Go Fishing";
            }
            return t.ToString();
        }
    }

    /// Pixel art for tricks, facing right, coloured from `Art.PropPalette`.
    static class TrickArt
    {
        public static readonly string[] Rocket = { ".k.", "krk", "krk", "kyk", ".n." };
        public static readonly string[] Flashlight = { "kkkkk", "kgggy", "kkkkk" };
        public static readonly string[] Pan = { "....kkkkkk", "nnnnkddddk" };
        public static readonly string[] EggUp = { ".iyi.", "iiiii" };
        public static readonly string[] EggDown = { ".iii.", "iiiii" };
        public static readonly string[] FireA = { "...y...", "..yoy..", ".yooy..", ".yoooy.", "nknnnkn" };
        public static readonly string[] FireB = { "....y..", "..yoy..", "..yooy.", ".yoooy.", "nknnnkn" };

        public static string[] Stick(bool toasted)
        {
            char m = toasted ? 'y' : 'i';
            return new[] { $"....{m}{m}", $"...n{m}{m}", "..n...", ".n....", "n....." };
        }

        public static string[] Pool(int width) => new[]
        {
            "." + new string('C', Math.Max(0, width - 2)) + ".",
            new string('c', width), new string('c', width),
        };

        public static string[] IceCream(double left)
        {
            var scoop = left > 0.66 ? new[] { "..PP.", ".PPPP", "PPPPP" }
                      : left > 0.33 ? new[] { ".....", ".PPP.", "PPPPP" }
                      : left > 0.05 ? new[] { ".....", ".....", ".PPP." }
                      : new[] { ".....", ".....", "....." };
            return scoop.Concat(new[] { "nyyyn", ".nyn.", "..n.." }).ToArray();
        }

        public static readonly string[][] Food =
        {
            new[] { "..G.", ".rr.", "rrrr", "rrrr", ".rr." },      // apple
            new[] { ".yy.", "ynyy", "yyny", ".yy." },              // cookie
            new[] { "..i..", ".iii.", "iiiii", "ikkki" },          // rice ball
        };
        public static readonly string[] Flower = { "..P..", ".PyP.", "..P..", "..G..", ".GG..", "..G.." };
        public static readonly string[] Cloud = { "...kkkk...", "..kWWWWk..", ".kWWWWWWkk", "kWWWWWWWWk", ".kkkkkkkk." };
        public static readonly string[] Soil = { "....nnnnnnnnnnnn....", "..nnnNnnnnNnnnnNnn..", "nnnnnnnnnnnnnnnnnnnn" };
        public static readonly string[] ButterflyA = { "P...P", "PPkPP", ".PkP." };
        public static readonly string[] ButterflyB = { "..k..", "PPkPP", "P.k.P" };
        public static readonly string[] Ball = { "rr", "rr" };

        // Typing activities
        public static readonly string[] Pile = { "..nnnn..", ".nnNnnn.", "nnnnnNnn" };
        public static readonly string[] Shovel = { "n....", ".n...", "..n..", "..ggg", "..gg." };
        /// Red-framed glasses on a half-size grid (2 pt cells), so each thin rim fits around an eye
        /// (eyes at sprite columns 9 and 12). Starts at half-column 10; the arm runs back toward the ear.
        public static readonly string[] Glasses = { ".......rrrr..rrrr", "rrrrrrrr..rrrr..r", ".......r..r..r..r",
                                                    ".......r..r..r..r", ".......r..r..r..r", ".......rrrr..rrrr" };

        // Mochi
        public static readonly string[] YarnA = { ".PP.", "PkPP", "PPkP", ".PP." };
        public static readonly string[] YarnB = { ".PP.", "PPkP", "PkPP", ".PP." };
        public static readonly string[] Box = { "Uk..............kU", "kkkkkkkkkkkkkkkkkk", "kTTTTTTTUUTTTTTTTk", "kTTTTTTTUUTTTTTTTk",
                                                "kTTTTTTTUUTTTTTTTk", "kTTTTTTTTTTTTTTTTk", "kkkkkkkkkkkkkkkkkk" };
        public static readonly string[] LaserDot = { "r" };
        public static readonly string[] Rod = { "......nn", ".....n.k", "....n..k", "...n...k", "..n....k", ".n.....k", "n......k", ".......r" };
        public static readonly string[] Fish = { ".ooo.o", "oooooo", ".ooo.o" };
    }

    /// Little pictures shown in speech bubbles in place of the Mac version's emoji
    /// (GDI+ can't draw colour emoji). Drawn at 2 pt per art pixel with `Art.PropPalette`.
    static class BubbleIcons
    {
        public static readonly string[] Drop = { "...k...", "..kck..", ".kccck.", "kcCccck", "kcCccck", "kccccck", ".kkkkk." };
        public static readonly string[] Rice = { "..kkkkk..", ".kiiiiik.", "kiiiiiiik", "kkkkkkkkk", "koooooook", ".koooook.", "..kkkkk.." };
        public static readonly string[] Stew = { "..g...g..", "...g...g.", "..g...g..", "kkkkkkkkk", "dkoyoNokd", ".kdddddk.", ".kdddddk.", "..kkkkk.." };
        public static readonly string[] Stretch = { "k.....k", ".k.o.k.", "..kok..", "...k...", "...k...", "..k.k..", ".k...k." };
        public static readonly string[] Tulip = TrickArt.Flower;
        public static readonly string[] Heart = { ".PP.PP.", "PPPPPPP", "PPPPPPP", ".PPPPP.", "..PPP..", "...P..." };
    }

    static class ColorMath
    {
        public static Color WithAlpha(this Color c, double a) =>
            Color.FromArgb((int)Math.Round(Math.Max(0, Math.Min(1, a)) * 255), c.R, c.G, c.B);

        /// Shifts the hue of the given palette keys (HSB, like NSColor), wrapping around.
        public static Palette HueShifted(Palette palette, double amount, char[] keys)
        {
            var result = new Palette(palette);
            foreach (var key in keys)
            {
                if (!palette.TryGetValue(key, out var c)) continue;
                ToHsb(c, out double h, out double s, out double v);
                h = (h + amount) % 1.0;
                if (h < 0) h += 1;
                result[key] = FromHsb(h, s, v, c.A);
            }
            return result;
        }

        static void ToHsb(Color c, out double h, out double s, out double v)
        {
            double r = c.R / 255.0, g = c.G / 255.0, b = c.B / 255.0;
            double max = Math.Max(r, Math.Max(g, b)), min = Math.Min(r, Math.Min(g, b)), d = max - min;
            v = max;
            s = max == 0 ? 0 : d / max;
            if (d == 0) h = 0;
            else if (max == r) h = ((g - b) / d) / 6;
            else if (max == g) h = ((b - r) / d + 2) / 6;
            else h = ((r - g) / d + 4) / 6;
            if (h < 0) h += 1;
        }

        static Color FromHsb(double h, double s, double v, int alpha)
        {
            double r, g, b;
            double hh = h * 6;
            int i = (int)Math.Floor(hh) % 6;
            double f = hh - Math.Floor(hh), p = v * (1 - s), q = v * (1 - f * s), t = v * (1 - (1 - f) * s);
            switch (i)
            {
                case 0: r = v; g = t; b = p; break;
                case 1: r = q; g = v; b = p; break;
                case 2: r = p; g = v; b = t; break;
                case 3: r = p; g = q; b = v; break;
                case 4: r = t; g = p; b = v; break;
                default: r = v; g = p; b = q; break;
            }
            return Color.FromArgb(alpha, (int)Math.Round(r * 255), (int)Math.Round(g * 255), (int)Math.Round(b * 255));
        }
    }
}
