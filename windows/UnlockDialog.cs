// The unlock code dialog. Laid out by hand at the monitor's DPI, Windows 11 style:
// a white content area and a grey button bar along the bottom.

using System;
using System.Drawing;
using System.Windows.Forms;

namespace DeskPet
{
    sealed class UnlockDialog : Form
    {
        const string Problem = "Check the code and try again. It has 20 letters and numbers; dashes, spaces and capitals don't matter.";

        readonly Label heading, body;
        readonly TextBox box;
        readonly Panel footer;
        readonly Button unlock, cancel;
        readonly double s;
        readonly int pad, width;

        public string Code { get; private set; }

        public UnlockDialog(string intro, double scale, Icon icon)
        {
            s = scale;
            int P(double v) => (int)Math.Round(v * s);
            pad = P(24);
            width = P(420);

            AutoScaleMode = AutoScaleMode.None;
            Text = "DeskPet";
            Icon = icon;
            FormBorderStyle = FormBorderStyle.FixedDialog;
            MaximizeBox = MinimizeBox = false;
            StartPosition = FormStartPosition.CenterScreen;
            ShowInTaskbar = true;
            TopMost = true;
            BackColor = Color.White;
            Font = new Font("Segoe UI", (float)(12 * s), GraphicsUnit.Pixel);

            heading = new Label
            {
                AutoSize = false, Font = new Font("Segoe UI Semibold", (float)(20 * s), GraphicsUnit.Pixel),
                ForeColor = Color.FromArgb(26, 26, 26), Text = "Unlock All Pets", UseMnemonic = false,
            };
            body = new Label { AutoSize = false, ForeColor = Color.FromArgb(70, 70, 70), Text = intro, UseMnemonic = false };
            box = new TextBox
            {
                Font = new Font("Consolas", (float)(15 * s), GraphicsUnit.Pixel),
                CharacterCasing = CharacterCasing.Upper, MaxLength = 40,
            };
            footer = new Panel { BackColor = Color.FromArgb(243, 243, 243) };
            unlock = new Button { Text = "Unlock", FlatStyle = FlatStyle.System };
            cancel = new Button { Text = "Cancel", FlatStyle = FlatStyle.System, DialogResult = DialogResult.Cancel };
            unlock.Click += (_, __) => TryUnlock();

            footer.Controls.Add(unlock);
            footer.Controls.Add(cancel);
            Controls.AddRange(new Control[] { heading, body, box, footer });
            AcceptButton = unlock;
            CancelButton = cancel;
            DoLayout();
        }

        void DoLayout()
        {
            int P(double v) => (int)Math.Round(v * s);
            int inner = width - 2 * pad;
            int y = P(20);

            var hs = TextRenderer.MeasureText(heading.Text, heading.Font, new Size(inner, 0), TextFormatFlags.WordBreak);
            heading.SetBounds(pad, y, inner, hs.Height);
            y += hs.Height + P(8);

            var bs = TextRenderer.MeasureText(body.Text, body.Font, new Size(inner, 0), TextFormatFlags.WordBreak);
            body.SetBounds(pad, y, inner, bs.Height + P(2));
            y += bs.Height + P(16);

            box.SetBounds(pad, y, inner, box.PreferredHeight);
            y += box.PreferredHeight + P(24);

            int footerH = P(72), bw = P(100), bh = P(32);
            footer.SetBounds(0, y, width, footerH);
            cancel.SetBounds(width - pad - bw, (footerH - bh) / 2, bw, bh);
            unlock.SetBounds(width - pad - 2 * bw - P(8), (footerH - bh) / 2, bw, bh);
            ClientSize = new Size(width, y + footerH);
        }

        protected override void OnShown(EventArgs e)
        {
            base.OnShown(e);
            Native.SendMessage(box.Handle, 0x1501 /* EM_SETCUEBANNER */, (IntPtr)1, "XXXXX-XXXXX-XXXXX-XXXXX");
            Activate();
            box.Focus();
        }

        void TryUnlock()
        {
            if (License.IsValid(box.Text))
            {
                Code = box.Text.Trim();
                DialogResult = DialogResult.OK;
                Close();
                return;
            }
            heading.Text = "That code didn't work";
            heading.ForeColor = Color.FromArgb(196, 43, 28);
            body.Text = Problem;
            DoLayout();
            box.Focus();
            box.SelectAll();
        }
    }
}
