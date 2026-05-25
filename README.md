# IPNOVA VPN — Windows (Tauri + sing-box)

تطبيق سطح مكتب يتصل بـ **vpnnovo** و **Marzban VLESS** بدون لصق روابط الاشتراك.

## تحميل العميل (Wi‑Fi)

**https://github.com/islam11hero/ipnova-windows-vpn/releases/latest** → `IPNOVA-VPN-Setup.exe`

| توثيق | |
|--------|---|
| [docs/GITHUB-SETUP.md](docs/GITHUB-SETUP.md) | GitHub + Releases + islam11hero |
| [docs/GITHUB-WIFI-AR.md](docs/GITHUB-WIFI-AR.md) | شرح التحميل بالعربية |

## المتطلبات

- Windows 10/11 (amd64)
- حساب **vpnnovo** (Supabase)
- **vpnnovo** يعمل (`npm run dev` أو نشر إنتاج)
- Marzban متاح من السيرفر — راجع [vpnnovo/docs/MARZBAN_LAN.md](../vpnnovo/docs/MARZBAN_LAN.md)

## إعداد سريع

```powershell
cd "windws app"
copy .env.example .env
# عدّل VITE_API_BASE_URL و Supabase

npm install
.\scripts\download-singbox.ps1
npm run tauri:dev
```

**وضع بروكسي النظام** (افتراضي) لا يحتاج مسؤول. **TUN** يطلب UAC لـ sing-box فقط.

عند أول تشغيل: اتبع شاشة «إعداد Windows» أو راجع [docs/WINDOWS_SECURITY_AR.md](docs/WINDOWS_SECURITY_AR.md).

## إثبات sing-box (POC)

```powershell
# 1) سجّل دخول في الموقع أو التطبيق واحصل على access_token من Supabase
.\scripts\fetch-profile.ps1 -AccessToken "eyJ..."
.\scripts\run-singbox.ps1
```

## البناء للإنتاج

```powershell
npm run tauri:build
```

راجع [SECURITY.md](./SECURITY.md) للتوقيع وDefender.

## API المستخدمة

| Method | Path |
|--------|------|
| GET | `/api/client/vpn/status` |
| GET | `/api/client/vpn/profile` |
| POST | `/api/client/vpn/refresh` |

جميع الطلبات: `Authorization: Bearer <supabase_access_token>`.
