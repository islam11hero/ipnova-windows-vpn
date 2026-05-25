# بروكسي النظام على Windows — التغطية والمشاكل الشائعة

## ماذا يغطي وضع «بروكسي النظام»؟

| يعمل عادةً | لا يعمل / يحتاج TUN |
|------------|---------------------|
| Edge، Chrome، Firefox (إن كان «استخدام إعدادات النظام») | ألعاب UDP / معظم Anti-Cheat |
| تطبيقات WinINet / HTTP(S) | QUIC ثابت بدون بروكسي |
| خدمات WinHTTP (بعد `netsh winhttp`) | DNS لكل التطبيقات (تسريب DNS محتمل) |
| curl مع متغيرات البيئة إن ضُبطت | تطبيقات تتجاهل البروكسي عمداً |

التطبيق يضبط:

1. **sing-box** `mixed` على `127.0.0.1:2080` (`set_system_proxy: false` — التطبيق يضبط Windows يدوياً)
2. **WinINet** (مستخدم): `http` + `https` + `socks` على نفس المنفذ
3. **WinHTTP** (نظام): `advproxy` ثم `set proxy` ثم `import proxy source=ie`
4. تعطيل **WPAD/AutoDetect** مؤقتاً لتجنب تعارض Windows 11 24H2
5. عند Disconnect: استعادة WinINet + WinHTTP من `proxy-backup.json` (بدون `reset` أعمى فقط)

عند قطع الاتصال يُستعاد إعداد البروكسي السابق من نسخة احتياطية.

## Windows 11 24H2 — مشاكل معروفة

### 1) تعطيل WinHttpAutoProxySvc

في بيئات مُقسّاة أمنياً تُعطّل خدمة **WinHTTP Web Proxy Auto-Discovery**. في 24H2 قد يضيف **WcmSvc** اعتماداً عليها فيسبب:

- انقطاع Wi‑Fi
- «لا إنترنت»
- تطبيقات تتجاهل البروكسي

**فحص:**

```powershell
reg query HKLM\SYSTEM\CurrentControlSet\Services\WcmSvc /v DependOnService
reg query HKLM\SYSTEM\CurrentControlSet\Services\WinHttpAutoProxySvc /v Start
```

إن كان `WinHttpAutoProxySvc` في قائمة التبعيات و`Start = 0x4` (معطّل):

- اجعل الخدمة **Manual** (`Start = 3`)، أو
- أزل `WinHttpAutoProxySvc` من `DependOnService` لـ WcmSvc (سياسة IT)

### 2) WinHTTP ≠ WinINet

بعد 24H2 بعض الخدمات لا تقرأ إعدادات Edge فقط. لذلك IPNOVA يضبط **الاثنين**.

### 3) DNS Cache معطّل

تعطيل `Dnscache` عبر السجل قد يؤخر الشبكة بعد 24H2. اتركه Automatic أو Manual.

## متى يُنصح بـ TUN؟

- ألعاب أونلاين
- حماية DNS كاملة
- تطبيقات لا تحترم بروكسي النظام

يتطلب **UAC** مرة عند الاتصال (sing-box كمسؤول فقط).

## تشخيص من التطبيق

- الإعدادات → **5-minute connection checklist** + **Run live checks**
- **Check proxy & WinHTTP** (`windows_proxy_diagnostics`)
- كشف VPN آخر (عمليات / محولات TAP-TUN)
- **Proxy for all users (admin)** — HKLM للأجهزة المشتركة
- دليل إنجليزي: [`WINDOWS_TROUBLESHOOTING_EN.md`](WINDOWS_TROUBLESHOOTING_EN.md)
