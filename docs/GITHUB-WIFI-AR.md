# GitHub + تثبيت عبر Wi‑Fi (للعميل)

## الفكرة

| أنت (مرة واحدة) | العميل (عبر Wi‑Fi / إنترنت) |
|-----------------|------------------------------|
| ترفع المشروع إلى GitHub | يفتح رابط التحميل على جواله أو PC |
| GitHub Actions يبني `IPNOVA-VPN-Setup.exe` | يحمّل الملف |
| يظهر في **Releases** | دبل كليك → Install |

**لا ترسل** مجلد المشروع للعميل. **أرسل الرابط فقط.**

---

## 1) إنشاء مستودع GitHub

1. [github.com/new](https://github.com/new) → اسم مثلاً `ipnova-windows-vpn`
2. **Private** = الكود مخفي (العملاء لا يرون المصدر)
3. **Public** = الكود ظاهر (التحميل من Releases أسهل للجميع)

> للتحميل بدون تسجيل GitHub: استخدم **Public** أو [GitHub Releases على repo عام](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository).

---

## 2) رفع المشروع من Mac

```bash
cd "/Users/imsi/Desktop/windws app"
git init
git add .
git commit -m "IPNOVA VPN Windows app"
git branch -M main
git remote add origin https://github.com/YOUR_USER/ipnova-windows-vpn.git
git push -u origin main
```

**لا ترفع:** `.env` (فيه مفاتيح Supabase) — موجود في `.gitignore`.

---

## 3) بناء المثبّت على GitHub (بدون PC ويندوز عندك)

### طريقة أ — وسم إصدار (موصى بها)

```bash
git tag v0.1.0
git push origin v0.1.0
```

يفتح تلقائياً workflow **Release** ويبني على `windows-latest`.

### طريقة ب — يدوي

GitHub → **Actions** → **Release** → **Run workflow** → version `0.1.0`

انتظر ~20–40 دقيقة (أول مرة).

---

## 4) رابط التحميل للعميل (Wi‑Fi)

بعد نجاح الـ workflow:

**GitHub → Releases → v0.1.0**

الرابط يكون مثل:

```
https://github.com/YOUR_USER/ipnova-windows-vpn/releases/download/v0.1.0/IPNOVA-VPN-Setup.exe
```

العميل:
1. يفتح الرابط من الجوال أو الكمبيوتر (Wi‑Fi)
2. يحمّل `IPNOVA-VPN-Setup.exe`
3. يشغّله → Install
4. يفتح **IPNOVA VPN**

---

## 5) رسالة جاهزة للعميل (واتساب / إيميل)

```
تحميل IPNOVA VPN (Windows 10/11):

https://github.com/YOUR_USER/ipnova-windows-vpn/releases/latest

1) افتح الرابط
2) حمّل IPNOVA-VPN-Setup.exe
3) شغّل الملف واضغط Install
4) افتح التطبيق من قائمة ابدأ
```

`releases/latest` دائماً يوجّه لآخر إصدار.

---

## 6) تحديث لاحق (إصدار جديد)

```bash
# بعد تعديل الكود
git add .
git commit -m "fix: ..."
git tag v0.1.1
git push origin main
git push origin v0.1.1
```

العميل يحمّل من Releases الجديدة (أو نفس رابط `/latest`).

---

## 7) Wi‑Fi محلي (نفس الشبكة — اختياري)

إن أردت التوزيع **بدون إنترنت** في مكتب واحد:

- شارك مجلد فيه `IPNOVA-VPN-Setup.exe` عبر **مشاركة Windows** على الشبكة المحلية، أو
- انسخ الملف على فلاشة

GitHub = تحميل عبر **إنترنت Wi‑Fi**. الشبكة المحلية = ملف على مشاركة LAN.

---

## 8) ملاحظات

| موضوع | توضيح |
|--------|--------|
| SmartScreen | طبيعي بدون توقيع رقمي — More info → Run anyway |
| Repo خاص | العميل يحتاج دعوة GitHub أو ارفع الملف على Drive وضع الرابط في Release description |
| فشل Actions | Actions → Release → افتح الـ run → انسخ الخطأ الأحمر |

---

## الخلاصة

```
GitHub (كود) → tag v0.1.0 → Actions تبني setup.exe → Releases
→ ترسل الرابط للعميل → يحمّل عبر Wi‑Fi → يثبت
```

**لا** ترسل ZIP القديم `IPNOVA-Windows-Install` بدون `setup.exe`.
