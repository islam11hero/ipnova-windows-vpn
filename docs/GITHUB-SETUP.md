# IPNOVA VPN — GitHub (حساب islam11hero)

## المستودع

| | |
|---|---|
| **الرابط** | https://github.com/islam11hero/ipnova-windows-vpn |
| **Releases (تحميل العميل)** | https://github.com/islam11hero/ipnova-windows-vpn/releases/latest |
| **Actions (البناء)** | https://github.com/islam11hero/ipnova-windows-vpn/actions |

---

## أول إصدار للعميل (setup.exe)

### 1) انتظر اكتمال workflow «Release»

بعد رفع الوسم `v0.1.0` (أو إعادة رفعه بعد إصلاح CI)، افتح **Actions** → **Release** → انتظر ✅ أخضر (~20–40 دقيقة أول مرة).

إن فشل البناء سابقاً: تحقق أن الخطوة **Prepare sing-box** نجحت وأن **npm ci** يعمل قبل `tauri build` (لا تستخدم `npx tauri icon` قبل تثبيت الحزم).

### 2) أو شغّل يدوياً

**Actions** → **Release** → **Run workflow** → version: `0.1.0`

### 3) رابط التحميل للعميل (Wi‑Fi) — جاهز

**آخر إصدار (موصى به):**

https://github.com/islam11hero/ipnova-windows-vpn/releases/latest

حمّل: **`IPNOVA-VPN-Setup.exe`**

رابط مباشر (v0.1.0):

https://github.com/islam11hero/ipnova-windows-vpn/releases/download/v0.1.0/IPNOVA-VPN-Setup.exe

### 4) رسالة للعميل

```
حمّل IPNOVA VPN لـ Windows:

https://github.com/islam11hero/ipnova-windows-vpn/releases/latest

1) افتح الرابط (Wi‑Fi)
2) حمّل IPNOVA-VPN-Setup.exe
3) شغّل الملف → Install
4) افتح IPNOVA VPN من قائمة ابدأ
```

---

## تحديث إصدار جديد

```bash
cd "/Users/imsi/Desktop/windws app"
git add .
git commit -m "وصف التعديل"
git push
git tag v0.1.1
git push origin v0.1.1
```

---

## أسرار GitHub (للبناء في Actions)

في **Settings → Secrets and variables → Actions** أضف:

| Secret | مثال |
|--------|------|
| `VITE_SUPABASE_URL` | `https://xxx.supabase.co` |
| `VITE_SUPABASE_ANON_KEY` | مفتاح anon |
| `VITE_API_BASE_URL` | `https://api.vpnnovo.example` |
| `VITE_SITE_URL` | `https://vpnnovo.example` |

بدونها يُبنى المثبت لكن التطبيق يطلب `.env` عند التشغيل.

**من جهازك (مرة واحدة):**

```bash
gh auth login
./scripts/setup-github-secrets.sh
```

يقرأ القيم من `.env` المحلي ويرفعها إلى GitHub Secrets.

## أسرار لا تُرفع

- `.env` — محلي فقط (للتطوير)
- انسخ من `.env.example` على جهاز التطوير

---

## إن فشل البناء على GitHub

1. **Actions** → فشل → افتح الـ log
2. أو ابنِ على Windows: `scripts/INSTALL-ON-WINDOWS.ps1`
3. ارفع يدوياً: **Releases** → **Draft new release** → اسحب `IPNOVA-VPN-Setup.exe`

---

## جعل المستودع خاصاً (اختياري)

```bash
gh repo edit islam11hero/ipnova-windows-vpn --visibility private
```

العميل يحتاج رابط عام — ارفع `setup.exe` على Drive أو اجعل Releases من repo عام.

---

## أوامر مفيدة

```bash
# حالة المشروع
git status
git pull

# تشغيل البناء من GitHub فقط
git tag v0.1.0
git push origin v0.1.0
```

---

## ملفات مهمة في المشروع

| ملف | الغرض |
|-----|--------|
| `docs/GITHUB-WIFI-AR.md` | شرح Wi‑Fi + Releases بالعربية |
| `docs/INSTALL-WINDOWS-AR.md` | بناء محلي على Windows |
| `.github/workflows/release.yml` | بناء setup.exe تلقائياً |
| `.github/workflows/windows-ci.yml` | فحص CI عند كل push |
