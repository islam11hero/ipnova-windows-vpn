# IPNOVA VPN — حزمة التحميل (Windows)

## المتطلبات

- **Windows 10** (إصدار 1809 أو أحدث) أو **Windows 11** — 64-bit  
- اتصال إنترنت لتسجيل الدخول وتفعيل الاشتراك  
- **WebView2** (مثبت مسبقاً على معظم أجهزة Windows 10/11)

## التثبيت

1. فك ضغط `IPNOVA-VPN-0.1.0-win64.zip`
2. شغّل **`IPNOVA VPN_0.1.0_x64-setup.exe`** (NSIS) **أو** ملف **`.msi`**
3. إذا ظهر SmartScreen: **More info → Run anyway** (حتى يُوقَّع التطبيق رسمياً)
4. افتح التطبيق → سجّل دخولك أو جرّب الضيف → اختر الدولة → **Connect**

## أول تشغيل (موصى به)

| الخطوة | السبب |
|--------|--------|
| وضع **System proxy** (افتراضي) | يعمل بدون تشغيل كمسؤول |
| إن حُجب sing-box | **Settings** → Defender → **Add exclusion** (UAC) |
| إن ظهر شريط عربي عن **صلاحيات المسؤول** | وافق على **WinHTTP (UAC)** أو **إصلاح 2026** |

## محتويات الحزمة

| ملف | الوصف |
|-----|--------|
| `*-setup.exe` / `*.msi` | مثبّت التطبيق |
| `sing-box\` | sing-box + wintun (مضمّن في المثبّت أيضاً) |
| `install-defender-exclusions.ps1` | استثناءات Defender (كمسؤول) |
| `WINDOWS_*.md` | استكشاف الأخطاء |

## البناء من المصدر (للمطوّرين)

على **Windows** مع Node 20 + Rust:

```powershell
cd "windws app"
.\scripts\build-release.ps1
```

الناتج: `release\dist\IPNOVA-VPN-0.1.0-win64.zip`

## الدعم

- سجلات الدعم: **Settings → Proxy diagnostics → Open support logs**
- نسخ التشخيص: **Copy diagnostics**
