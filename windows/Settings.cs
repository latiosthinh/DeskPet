// Settings in HKCU\Software\DeskPet, and Open at Login through the user's Run key.

using System;
using System.Globalization;
using System.Windows.Forms;
using Microsoft.Win32;

namespace DeskPet
{
    static class Settings
    {
        const string KeyPath = @"Software\DeskPet";

        static object Read(string name)
        {
            using (var key = Registry.CurrentUser.OpenSubKey(KeyPath)) return key?.GetValue(name);
        }

        static void Write(string name, object value, RegistryValueKind kind)
        {
            using (var key = Registry.CurrentUser.CreateSubKey(KeyPath)) key.SetValue(name, value, kind);
        }

        public static bool Has(string name) => Read(name) != null;

        public static void Remove(string name)
        {
            using (var key = Registry.CurrentUser.OpenSubKey(KeyPath, true)) key?.DeleteValue(name, false);
        }

        public static string GetString(string name) => Read(name) as string;
        public static void SetString(string name, string value) => Write(name, value ?? "", RegistryValueKind.String);

        public static bool GetBool(string name, bool fallback) => Read(name) is int v ? v != 0 : fallback;
        public static void SetBool(string name, bool value) => Write(name, value ? 1 : 0, RegistryValueKind.DWord);

        public static double? GetDouble(string name) =>
            double.TryParse(GetString(name), NumberStyles.Float, CultureInfo.InvariantCulture, out var v) ? v : (double?)null;
        public static void SetDouble(string name, double value) => SetString(name, value.ToString("R", CultureInfo.InvariantCulture));
    }

    enum LoginState { Off, On, TurnedOffInTaskManager }

    static class Login
    {
        const string RunPath = @"Software\Microsoft\Windows\CurrentVersion\Run";
        const string ApprovedPath = @"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
        const string Name = "DeskPet";
        static string Command => "\"" + Application.ExecutablePath + "\"";

        /// Task Manager's Startup apps page can switch it off; that's kept separately from the Run key.
        public static LoginState State
        {
            get
            {
                using (var run = Registry.CurrentUser.OpenSubKey(RunPath))
                    if (run?.GetValue(Name) == null) return LoginState.Off;
                using (var approved = Registry.CurrentUser.OpenSubKey(ApprovedPath))
                    if (approved?.GetValue(Name) is byte[] b && b.Length > 0 && (b[0] & 1) == 1) return LoginState.TurnedOffInTaskManager;
                return LoginState.On;
            }
        }

        public static void TurnOn()
        {
            using (var run = Registry.CurrentUser.CreateSubKey(RunPath)) run.SetValue(Name, Command, RegistryValueKind.String);
            using (var approved = Registry.CurrentUser.OpenSubKey(ApprovedPath, true)) approved?.DeleteValue(Name, false);
        }

        public static void TurnOff()
        {
            using (var run = Registry.CurrentUser.OpenSubKey(RunPath, true)) run?.DeleteValue(Name, false);
            using (var approved = Registry.CurrentUser.OpenSubKey(ApprovedPath, true)) approved?.DeleteValue(Name, false);
        }

        /// If DeskPet was moved, point the login entry at where it is now.
        public static void RefreshPath()
        {
            using (var run = Registry.CurrentUser.OpenSubKey(RunPath, true))
            {
                if (run?.GetValue(Name) is string cmd && !string.Equals(cmd, Command, StringComparison.OrdinalIgnoreCase))
                    run.SetValue(Name, Command, RegistryValueKind.String);
            }
        }
    }
}
