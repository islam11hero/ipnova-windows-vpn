# دليل العميل — تجاوز تحذيرات Windows بسهولة

## 1) SmartScreen («Windows protected your PC»)

هذا طبيعي للتطبيقات الجديدة خارج Microsoft Store.

| الخطوة | ماذا يفعل العميل |
|--------|------------------|
| 1 | عند التثبيت يظهر تحذير أزرق |
| 2 | انقر **More info** (مزيد من المعلومات) |
| 3 | انقر **Run anyway** (التشغيل على أي حال) |

بعد عدة أسابيع من التحميلات النظيفة والتوقيع الرقمي، يقل التحذير تلقائياً. شهادة EV وحدها **لا تكفي** بعد 2024.

**للشركة:** وقّع كل إصدار بـ `scripts/sign-release.ps1` ونفس شهادة OV/EV.

## 2) Windows Defender

قد يحجب `sing-box.exe` أو `wintun.dll` (أدوات شبكة شرعية).

| الحل | من يفعله |
|------|----------|
| من التطبيق | الإعداد الأولى أو **Settings → Check Defender** → **Add Defender exclusion** (UAC) |
| يدوياً | `scripts/install-defender-exclusions.ps1` كمسؤول (مسارات + عملية + CFA) |
| إبلاغ Microsoft | **Copy hashes (WDSI)** ثم [filesubmission](https://www.microsoft.com/en-us/wdsi/filesubmission) |
| Smart App Control | الإعدادات تعرض `on` / `evaluation` — وقّع التطبيق أو عطّل SAC |
| Tamper Protection | الاستثناءات تحتاج UAC كمسؤول — رسالة واضحة في التطبيق |

## 3) UAC (صلاحيات المسؤول)

| وضع الاتصال | UAC |
|-------------|-----|
| **بروكسي النظام** (افتراضي) | لا — WinINet + WinHTTP + sing-box `set_system_proxy` |
| **TUN تلقائي** | نعم — لـ sing-box فقط عند الحاجة |
| **TUN إلزامي** | نعم — في كل اتصال |

التطبيق نفسه يعمل كمستخدم عادي (`asInvoker`). لا يطلب «تشغيل كمسؤول» للتطبيق كاملاً.

## 4) توصيات للنشر

1. **وقّع** المثبّت + `sing-box.exe` + `wintun.dll`.
2. **لا تغيّر** الشهادة كل إصدار بدون سبب (السمعة تُبنى من جديد).
3. **Microsoft Store** = بدون SmartScreen (اختياري مستقبلياً).
4. للشركات: سياسة GPO لاستثناء مسار التثبيت.

## 5) استكشاف الأخطاء

| العرض | الحل |
|-------|------|
| Connect فشل + UAC | جرّب «بروكسي النظام» أو وافق على UAC |
| sing-box not found | `.\scripts\download-singbox.ps1` |
| تم حذف الملف | استثناء Defender + إبلاغ WDSI |
| لا إنترنت بعد الاتصال | تحديث → قطع → Connect مع وضع آخر |
| بروكسي لا يعمل 24H2 | راجع `docs/WINDOWS_PROXY_AR.md` — WinHTTP / WcmSvc |
| تطبيق معيّن لا يمر عبر VPN | استخدم TUN — البروكسي لا يغطي UDP/QUIC |
