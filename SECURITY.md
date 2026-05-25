# الأمان والتوزيع — Windows

## للعميل (عربي)

راجع [docs/WINDOWS_SECURITY_AR.md](docs/WINDOWS_SECURITY_AR.md).

## تخزين الجلسة (DPAPI)

على Windows تُشفَّر جلسة Supabase بـ **CryptProtectData** في:

`%AppData%\com.ipnova.windows-vpn\session.dat`

## تقليل احتكاك Windows (مطبّق في التطبيق)

| المشكلة | الحل في المشروع |
|---------|-----------------|
| تشغيل كمسؤول دائماً | Manifest `asInvoker` — الرفع لـ sing-box فقط |
| UAC مزعج | وضع **بروكسي النظام** بدون مسؤول (افتراضي) |
| Defender يحذف sing-box | زر استثناء + `install-defender-exclusions.ps1` |
| SmartScreen | شاشة إعداد أولى + توقيع كل الملفات |
| TUN فشل | رفع UAC لـ sing-box ثم رسالة «جرّب بروكسي النظام» |

## توقيع التطبيق (Authenticode)

1. شهادة **OV** كافية لـ SmartScreen (EV لا يعطي سمعة فورية منذ 2024).
2. بعد البناء:

```powershell
.\scripts\sign-release.ps1 -Thumbprint "YOUR_SHA1_THUMBPRINT"
```

3. في `src-tauri/tauri.conf.json` (env أو CI):

```json
"windows": {
  "certificateThumbprint": "${WINDOWS_CERT_THUMBPRINT}",
  "digestAlgorithm": "sha256",
  "timestampUrl": "http://timestamp.digicert.com"
}
```

4. وقّع **كل** ملف: المثبّت، `sing-box.exe`، `wintun.dll`.

## إبلاغ False Positive

https://www.microsoft.com/en-us/wdsi/filesubmission

## صلاحيات

- لا تضمّن `MARZBAN_PASSWORD` في التطبيق.
- لا تخزّن subscription URL في السجلات.

## CORS (vpnnovo)

```
WINDOWS_APP_CORS_ORIGINS=tauri://localhost,http://localhost:1420
```
