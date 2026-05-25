const API_ERROR_EN: Record<string, string> = {
  "No active VPN subscription linked to this account":
    "No active VPN plan on this account — purchase a plan or link your order",
  "Complete checkout or link your order from the dashboard":
    "Complete checkout on the website or link your order from the dashboard",
  "Invalid guest session":
    "Guest session expired — a new free trial will start automatically",
  "Missing guest credentials": "Guest session incomplete — restart the app",
  "تم استخدام اليوم المجاني على هذا الجهاز. أنشئ حساباً أو اشترِ باقة.":
    "Free trial already used on this device. Create an account or buy a plan.",
  "لا توجد باقة نشطة — اشترِ عرض 3 أشهر أو أنشئ حساباً بعد التجربة":
    "No active plan — buy a plan or sign up after your trial",
  "لا توجد باقة نشطة على هذا الحساب — اشترِ عرض 3 أشهر أو اربط طلبك":
    "No active plan on this account — buy a plan or link your order",
  "بصمة الجهاز غير صالحة": "Invalid device fingerprint",
  "انتهت جلسة التجربة — سيتم تفعيل يوم مجاني جديد":
    "Trial session expired — starting a new free day",
};

export function translateApiError(message: string): string {
  const trimmed = message.trim();
  return API_ERROR_EN[trimmed] ?? trimmed;
}
