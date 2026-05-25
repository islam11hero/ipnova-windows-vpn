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

### 3) رابط التحميل للعميل (Wi‑Fi)

```
https://github.com/islam11hero/ipnova-windows-vpn/releases/download/v0.1.0/IPNOVA-VPN-Setup.exe
```

أو دائماً آخر إصدار:

```
https://github.com/islam11hero/ipnova-windows-vpn/releases/latest
```

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

## أسرار لا تُرفع

- `.env` — محلي فقط (Supabase و API)
- انسخ من `.env.example` على كل جهاز

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
