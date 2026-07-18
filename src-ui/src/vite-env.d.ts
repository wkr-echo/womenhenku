/// <reference types="vite/client" />

// Stub @tauri-apps/api/core for browser dev mode.
// In Tauri runtime, the actual module is injected by Tauri's webview.
declare module "@tauri-apps/api/core" {
  export function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T>;
}