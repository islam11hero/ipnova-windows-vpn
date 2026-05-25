# تثبيت IPNOVA VPN على Windows (الطريقة الصحيحة)

## لماذا لم يُثبَّت التطبيق من ZIP السابق؟

الملفات التي أرسلناها (`IPNOVA-Windows-Install.zip`) كانت **أدوات تثبيت فارغة** — بدون ملف `IPNOVA VPN.exe` أو `setup.exe`.

لا يمكن **صنع** ملف التثبيت الحقيقي على Mac. يجب البناء **على جهاز Windows**.

---

## الطريقة 1 — مثبّت حقيقي (موصى بها)

### المتطلبات (مرة واحدة)

1. [Node.js 20 LTS](https://nodejs.org) — ثبّت مع npm  
2. [Rust](https://rustup.rs) — ثبّت ثم أعد تشغيل الحاسوب  
3. **Visual Studio Build Tools** — حمّل "Build Tools" واختر **Desktop development with C++**  
4. انسخ مجلد المشروع كاملاً: `windws app` إلى Windows (فلاشة / OneDrive / شبكة)

### خطوة واحدة

1. افتح مجلد `windws app`  
2. دبل كليك: **`ثبّت IPNOVA على ويندوز.bat`**  
3. انتظر حتى ينتهي (أول مرة 15–30 دقيقة)  
4. على سطح المكتب سيظهر: **`IPNOVA-VPN-Setup.exe`**  
5. دبل كليك عليه → Next → Install  
6. افتح **IPNOVA VPN** من قائمة ابدأ

### يدوياً (PowerShell)

```powershell
cd "C:\Users\DELL\Desktop\windws app"
powershell -ExecutionPolicy Bypass -File .\scripts\INSTALL-ON-WINDOWS.ps1
```

---

## الطريقة 2 — تجربة سريعة بدون مثبّت (للمطوّر)

```powershell
cd "C:\path\to\windws app"
.\scripts\download-singbox.ps1
npm ci
npm run tauri:dev
```

يفتح التطبيق مباشرة (نافذة تطوير).

---

## ملفات لا تستخدمها

| خطأ | صحيح |
|-----|------|
| `Install-IPNOVA.ps1` على Desktop | احذفه |
| ZIP قديم بدون setup.exe | استخدم `INSTALL-ON-WINDOWS.ps1` |

---

## بعد التثبيت

- وضع الاتصال: **System proxy** (بدون admin)  
- إن طلب UAC: وافق لـ WinHTTP أو TUN  
- الإعدادات: `.env` أو متغيرات `VITE_API_BASE_URL` و Supabase

---

## إن فشل البناء

أرسل لنا آخر 30 سطراً من نافذة PowerShell الحمراء بعد `npm run tauri:build`.
