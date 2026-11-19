// Ember, Pip, Puddle and Sprout are unlocked with a 20-character code (XXXXX-XXXXX-XXXXX-XXXXX):
// a 40-bit serial plus 60 bits of HMAC-SHA256 over it, made by Scripts/make-codes.ps1 (or .swift).
// The secret comes from CodeSecret, which build.ps1 generates outside the repo. Same check as main.swift.

using System.Collections.Generic;
using System.Linq;
using System.Security.Cryptography;

namespace DeskPet
{
    static class License
    {
        const string Alphabet = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

        public static bool IsValid(string code)
        {
            // Ignore dashes and spaces, and accept common misreadings (O for 0, I or L for 1).
            var chars = new List<char>();
            foreach (char raw in (code ?? "").ToUpperInvariant())
            {
                char ch = raw == 'O' ? '0' : (raw == 'I' || raw == 'L') ? '1' : raw;
                if (Alphabet.IndexOf(ch) >= 0) chars.Add(ch);
            }
            if (chars.Count != 20) return false;

            var bits = chars.SelectMany(ch =>
            {
                int v = Alphabet.IndexOf(ch);
                return Enumerable.Range(0, 5).Select(i => (v & (16 >> i)) != 0);
            }).ToArray();

            var message = new byte[6];
            message[0] = 1;
            for (int b = 0; b < 5; b++)
            {
                byte value = 0;
                for (int i = 0; i < 8; i++) value = (byte)((value << 1) | (bits[b * 8 + i] ? 1 : 0));
                message[1 + b] = value;
            }

            var secret = CodeSecret.Masked.Zip(CodeSecret.Mask, (a, m) => (byte)(a ^ m)).ToArray();
            byte[] mac;
            using (var hmac = new HMACSHA256(secret)) mac = hmac.ComputeHash(message);
            for (int i = 0; i < 60; i++)
                if (bits[40 + i] != ((mac[i / 8] & (0x80 >> (i % 8))) != 0)) return false;
            return true;
        }
    }
}
