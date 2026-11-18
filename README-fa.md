# OneMax Battery

[![CI](https://github.com/MHDLabs/OneMax-Battery/actions/workflows/ci.yml/badge.svg)](https://github.com/MHDLabs/OneMax-Battery/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/MHDLabs/OneMax-Battery?display_name=tag)](https://github.com/MHDLabs/OneMax-Battery/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

ابزاری سبک با Rust برای خواندن وضعیت باتری گیرنده‌های بی‌سیم ۲٫۴ گیگاهرتز از طریق HID در لینوکس.

> **سخت‌افزار هدف فعلی:** ماوس ONE MAX OM-W45B با شناسه USB برابر با `1EA7:0066`.

## ✨ قابلیت‌ها

- 🔎 **تشخیص خودکار گیرنده:** شناسایی رابط صحیح ماوس با بررسی شناسه USB و توصیفگر HID؛ رابط کیبورد با ماوس اشتباه گرفته نمی‌شود.
- 🔋 **خواندن باتری:** استخراج مقدار BCD مشاهده‌شده در بایت [7] پاسخ فرمان اختصاصی HID یعنی `0xB5 / 0x01`. بایت [3] که قبلاً اشتباهاً درصد فرض شده بود، دیگر استفاده نمی‌شود.
- 🖥️ **یکپارچگی با Waybar:** خروجی JSON زنده، وضعیت رنگی و راهنمای قابل‌خواندن.
- 🔐 **دسترسی کاربر عادی:** قانون udev اختیاری، دسترسی نشست گرافیکی فعال را از طریق `uaccess` فراهم می‌کند؛ نیازی به سرویس دائمی با دسترسی root نیست.
- 🪶 **سبک و متمرکز:** برنامه Rust در فضای کاربر، بدون ماژول کرنل و وابستگی سنگین.
- 🧪 **پروتکل آزمایش‌شده:** ارتباط واقعی با گیرنده ONE MAX OM-W45B تأیید شده است.

## 📦 نصب

آخرین باینری Linux x86_64 را از [GitHub Releases](https://github.com/MHDLabs/OneMax-Battery/releases) دریافت کن. انتشارها شامل تشخیص خودکار گیرنده، پایش پیوسته و یکپارچگی با Waybar هستند.

نصب باینری:

```sh
install -Dm755 one-max-battery-linux-x86_64 "$HOME/.local/bin/one-max-battery"
```

مطمئن شو مسیر `$HOME/.local/bin` در `PATH` قرار دارد.

### دسترسی بدون sudo

قانون udev را نصب کن:

```sh
sudo install -Dm644 contrib/udev/70-one-max-battery.rules \
  /etc/udev/rules.d/70-one-max-battery.rules
sudo udevadm control --reload-rules
sudo udevadm trigger --subsystem-match=hidraw
```

دانگل را یک‌بار جدا و دوباره وصل کن. این قانون با برچسب `uaccess` در systemd-logind فقط به نشست کاربری فعال دسترسی می‌دهد و دسترسی عمومی به همه دستگاه‌های HID ایجاد نمی‌کند.

## 🚀 استفاده

تشخیص خودکار به‌صورت پیش‌فرض فعال است:

```sh
one-max-battery
```

روش دستی نسخه قبلی هم همچنان پشتیبانی می‌شود:

```sh
one-max-battery /dev/hidraw3
one-max-battery --device /dev/hidraw3
```

پایش پیوسته در ترمینال:

```sh
one-max-battery --watch --interval 60
```

دریافت یک خروجی JSON برای Waybar:

```sh
one-max-battery --waybar
```

راهنمای کامل:

```sh
one-max-battery --help
```

## 🖥️ راه‌اندازی Waybar

ماژول Waybar برنامه را به‌صورت یک پردازش پیوسته اجرا می‌کند و در هر نوبت، یک خط JSON دریافت می‌کند. فاصله پیش‌فرض بررسی باتری ۶۰ ثانیه است.

۱. مقدار `custom/onemax-battery` را به آرایه `modules-right` در تنظیمات Waybar اضافه کن.

۲. بخش موجود در [contrib/waybar/onemax-battery.jsonc](contrib/waybar/onemax-battery.jsonc) را به تنظیمات Waybar منتقل کن.

۳. محتوای [contrib/waybar/style.css](contrib/waybar/style.css) را به فایل CSS فعلی Waybar اضافه کن.

۴. Waybar را Reload کن.

با شروع Waybar، برنامه هم خودکار اجرا می‌شود و در صورت خروج ناگهانی دوباره راه‌اندازی خواهد شد. اگر Sway از قبل Waybar را اجرا می‌کند، نیازی به خط دوم `exec` در Sway یا سرویس جداگانه systemd نیست. خود Waybar مدیریت‌کننده پردازش خواهد بود.

وضعیت‌ها:
- **Normal:** باتری بیشتر از ۳۰٪.
- **Warning:** باتری بین ۱۶ تا ۳۰٪.
- **Critical:** باتری ۱۵٪ یا کمتر.
- **Disconnected / Error:** دانگل پیدا نشده، دسترسی وجود ندارد یا پاسخی دریافت نشده است.

نمونه از آیکن Font Awesome برای باتری (``) استفاده می‌کند. در صورت نبود فونت مربوط، آن را با آیکن Nerd Font دلخواه جایگزین کن.

## 🔬 پروتکل سخت‌افزار

پاسخ واقعی گیرنده آزمایش‌شده:

```text
USB ID: 1EA7:0066
TX:     B5 01 00 00 00 00 00 00
RX:     B5 01 01 38 2E FF 09 70
```

پاسخ مشاهده‌شده روی OM-W45B برابر `B5 01 01 38 2E FF 09 70` است. در مقایسه‌ی حالت‌های مختلف، بایت [3] روی `0x38` ثابت ماند، اما بایت آخر همراه وضعیت باتری تغییر کرد: `0x00` با باتری خارج‌شده، `0x30` با باتری ضعیف/قدیمی و `0x70` با باتری تقریباً نو. برنامه اکنون بایت [7] را به‌صورت BCD فشرده می‌خواند؛ بنابراین `0x70` برابر ۷۰٪ است. بایت‌های [4..6] تفسیر نمی‌شوند. این نگاشت بر اساس همین سه وضعیت مشاهده‌شده است و برای سطوح دیگر شارژ، به‌ویژه شارژ کامل، هنوز به آزمایش نیاز دارد. مقدار `0x00` به‌تنهایی باتری خارج‌شده را از باتری کاملاً تخلیه‌شده متمایز نمی‌کند.

تشخیص خودکار، هم شناسه USB و هم توصیفگر گزارش اختصاصی را بررسی می‌کند تا رابط ماوس را از رابط کیبورد گیرنده جدا کند.

## 🛠️ ساخت و آزمون

به Rust نسخه ۱٫۷۴ یا جدیدتر نیاز است:

```sh
cargo fmt --all -- --check
cargo test --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo build --release --locked
```

GitHub Actions قالب‌بندی، آزمون‌ها، Clippy و ساخت باینری Linux x86_64 را بررسی می‌کند. با Push تگ نسخه‌ای مثل `v0.2.0`، فایل اجرایی و `SHA256SUMS` منتشر می‌شوند.

## ⚠️ محدوده پشتیبانی

- فعلاً فقط ONE MAX OM-W45B با شناسه `1EA7:0066` روی سخت‌افزار واقعی آزمایش شده است.
- تفسیر BCD بایت [7] بر اساس سه وضعیت مشاهده‌شده (باتری خارج‌شده، ضعیف و تقریباً نو) است؛ آزمایش در سطوح دیگر، به‌ویژه باتری کاملاً شارژشده، همچنان لازم است. قالب تک‌بایتی فعلی فقط مقادیر ۰۰ تا ۹۹ را می‌پذیرد.
- سازگاری با گیرنده‌های دیگر یا پروتکل‌های مشابه فرض نمی‌شود.
- این برنامه ابزار HID در فضای کاربر است، نه درایور کرنل.

## 📄 مجوز

MIT — متن کامل در فایل [LICENSE](LICENSE) قرار دارد.
