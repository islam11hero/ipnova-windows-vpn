/** True when running inside the Tauri desktop shell (not a plain browser). */
export function isTauriRuntime(): boolean {
  if (typeof window === "undefined") return false;
  return (
    "__TAURI_INTERNALS__" in window ||
    "__TAURI__" in window
  );
}

export function isBrowserDevMode(): boolean {
  return import.meta.env.DEV && !isTauriRuntime();
}
