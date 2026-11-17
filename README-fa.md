# OneMax Battery

[![CI](https://github.com/MHDLabs/OneMax-Battery/actions/workflows/ci.yml/badge.svg)](https://github.com/MHDLabs/OneMax-Battery/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/MHDLabs/OneMax-Battery?display_name=tag)](https://github.com/MHDLabs/OneMax-Battery/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

ابزاری سبک با Rust برای خواندن وضعیت باتری گیرنده‌های بی‌سیم ۲٫۴ گیگاهرتز از طریق HID در لینوکس.

> **سخت‌افزار هدف فعلی:** ماوس ONE MAX OM-W45B با شناسه USB برابر با `1EA7:0066`.

## ✨ قابلیت‌ها

- 🔎 **تشخیص خودکار گیرنده:** شناسایی رابط صحیح ماوس با بررسی شناسه USB و توصیفگر HID؛ رابط کیبورد با ماوس اشتباه گرفته نمی‌شود.
- 🔋 **خواندن باتری:** دریافت درصد گزارش‌شده از طریق فرمان آزمایش‌شده اختصاصی HID یعنی `0xB5 / 0x01`.
- 🖥️ **یکپارچگی با Waybar:** خروجی JSON زنده، وضعیت رنگی و راهنمای قابل‌خواندن.
- 🔐 **دسترسی کاربر عادی:** قانون udev اختیاری، دسترسی نشست گرافیکی فعال را از طریق `uaccess` فراهم می‌کند؛ نیازی به سرویس دائمی با دسترسی root نیست.
- 🪶 **سبک و متمرکز:** برنامه Rust در فضای کاربر، بدون ماژول کرنل و وابستگی سنگین.
- 🧪 **پروتکل آزمایش‌شده:** ارتباط واقعی با گیرنده ONE MAX OM-W45B تأیید شده است.

## 📦 نصب

باینری Linux x86_64 را از [GitHub Releases](https://github.com/MHDLabs/OneMax-Battery/releases) دریافت کن. انتشار عمومی فعلی یعنی `v0.1.0` نسخه اولیه با مسیر دستی دستگاه است؛ تشخیص خودکار و پایش Waybar در شاخه توسعه نسخه `v0.2.0` قرار دارند و تا انتشار آن تگ، نسخه عمومی محسوب نمی‌شوند.

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

مقدار مشاهده‌شده باتری `0x38` یا ۵۶ ده‌دهی است. بایت وضعیت فعلاً فقط به‌شکل خام نمایش داده می‌شود؛ معنای آن هنوز مستند نشده و به‌عنوان «در حال شارژ» یا «شارژ کامل» تفسیر نمی‌شود.

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
- درصد باتری از بایت گزارش‌شده دستگاه استخراج می‌شود؛ آزمایش در سطح شارژهای مختلف هنوز ارزشمند است.
- سازگاری با گیرنده‌های دیگر یا پروتکل‌های مشابه فرض نمی‌شود.
- این برنامه ابزار HID در فضای کاربر است، نه درایور کرنل.

## 📄 مجوز

MIT — متن کامل در فایل [LICENSE](LICENSE) قرار دارد.
