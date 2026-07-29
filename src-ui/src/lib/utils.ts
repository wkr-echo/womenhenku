import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatDate(dateStr: string | null): string {
  if (!dateStr) return "";
  const date = new Date(dateStr);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (minutes < 1) return t("刚刚");
  if (minutes < 60) return `${minutes} ${t("分钟前")}`;
  if (hours < 24) return `${hours} ${t("小时前")}`;
  if (days < 7) return `${days} ${t("天前")}`;

  return date.toLocaleDateString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  });
}

export function truncate(str: string, len: number): string {
  if (str.length <= len) return str;
  return str.slice(0, len) + "...";
}

export let currentLang = "zh";

export async function loadLanguage(): Promise<void> {
  if (typeof window !== "undefined" && ("__TAURI__" in window || "__TAURI_INTERNALS__" in window)) {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const saved = await invoke<string | null>("get_setting", { key: "app_language" });
      if (saved) {
        currentLang = saved;
        return;
      }
    } catch {}
  }
  const savedFromStorage = localStorage.getItem("uiLang");
  if (savedFromStorage) {
    currentLang = savedFromStorage;
    return;
  }
  if (typeof navigator !== "undefined") {
    currentLang = navigator.language.startsWith("zh") ? "zh" : "en";
  }
}

import zh from "../locales/zh";
import en from "../locales/en";

export const translations: Record<string, Record<string, string>> = { zh, en };

export function setLang(lang: string) {
  currentLang = lang;
  localStorage.setItem("uiLang", lang);
}

export function t(key: string): string {
  return translations[currentLang]?.[key] || key;
}

