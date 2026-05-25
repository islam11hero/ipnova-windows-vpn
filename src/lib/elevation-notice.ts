/** Mirrors Rust `ElevationNotice` — admin/UAC signal from backend. */
export type ElevationNotice = {
  required: boolean;
  reason: string;
  message_en: string;
  message_ar: string;
  can_use_uac: boolean;
  suggested_action: string;
};

export function elevationPrimaryMessage(notice: ElevationNotice): string {
  return notice.message_ar?.trim() || notice.message_en;
}

export function elevationSecondaryMessage(notice: ElevationNotice): string | null {
  if (!notice.message_ar?.trim() || !notice.message_en?.trim()) return null;
  if (notice.message_ar.trim() === notice.message_en.trim()) return null;
  return notice.message_en;
}