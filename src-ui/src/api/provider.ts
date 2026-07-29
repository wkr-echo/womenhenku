// Provider API — Stage 3
//
// 封装与 Provider 和 Agent 相关的 Tauri Command 调用。

import type { Provider } from "@/lib/types";
import { isTauri } from "./feed";
import { mockListProviders, mockAddProvider, mockUpdateProvider, mockDeleteProvider } from "./provider-mock";

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri()) {
    const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
    return tauriInvoke<T>(cmd, args);
  }
  throw new Error("Not in Tauri environment");
}

export async function addProvider(provider: {
  name: string;
  baseUrl: string;
  apiKeyRef: string;
  isDefault: boolean;
}): Promise<Provider> {
  if (isTauri()) {
    return invoke<Provider>("add_provider", { provider });
  }
  return mockAddProvider(provider);
}

export async function listProviders(): Promise<Provider[]> {
  if (isTauri()) {
    return invoke<Provider[]>("list_providers");
  }
  return mockListProviders();
}

export async function updateProvider(
  id: number,
  update: {
    name?: string;
    baseUrl?: string;
    apiKeyRef?: string;
    isDefault?: boolean;
  }
): Promise<Provider> {
  if (isTauri()) {
    return invoke<Provider>("update_provider", { id, update });
  }
  return mockUpdateProvider(id, update);
}

export async function deleteProvider(id: number): Promise<void> {
  if (isTauri()) {
    return invoke("delete_provider", { id });
  }
  return mockDeleteProvider(id);
}

export async function validateProvider(
  baseUrl: string,
  apiKey: string,
  model: string
): Promise<boolean> {
  return invoke<boolean>("validate_provider", { baseUrl, apiKey, model });
}

// ============ Provider Models ============

export async function addProviderModel(model: {
  providerId: number;
  modelName: string;
  isDefault: boolean;
}): Promise<void> {
  return invoke("add_provider_model", { model });
}

export async function listProviderModels(providerId: number): Promise<
  { id: number; providerId: number; modelName: string; isDefault: boolean; createdAt: string }[]
> {
  return invoke("list_provider_models", { providerId });
}

export async function deleteProviderModel(id: number): Promise<void> {
  return invoke("delete_provider_model", { id });
}

// ============ Summary Agent ============

export async function generateSummary(entryId: number, targetLanguage?: string, detailLevel?: string, force?: boolean): Promise<void> {
  return invoke("generate_summary", { entryId, targetLanguage, detailLevel, force });
}

export async function getSummaryText(entryId: number): Promise<string | null> {
  return invoke<string | null>("get_summary", { entryId });
}

export async function cancelSummary(entryId: number): Promise<void> {
  return invoke("cancel_summary", { entryId });
}

export async function clearSummary(entryId: number): Promise<void> {
  return invoke("clear_summary", { entryId });
}

// ============ Translation Agent ============

export async function translateEntry(entryId: number, targetLanguage?: string, concurrency?: number, force?: boolean): Promise<void> {
  return invoke("translate_entry", { entryId, targetLanguage, concurrency, force });
}

export async function getTranslationText(entryId: number): Promise<string | null> {
  return invoke<string | null>("get_translation", { entryId });
}

export async function cancelTranslation(entryId: number): Promise<void> {
  return invoke("cancel_translation", { entryId });
}

export async function clearTranslation(entryId: number): Promise<void> {
  return invoke("clear_translation", { entryId });
}

export async function retryFailedSegments(entryId: number): Promise<void> {
  return invoke("retry_failed_segments", { entryId });
}

// ============ Agent 事件监听 ============

/**
 * Tauri Event 名称：AI 流式推送
 * 事件数据结构: { taskId: number, content: string, isDone: boolean, agentType: string, error?: string }
 */
export const AI_STREAM_EVENT = "ai-stream";

export interface AiStreamEvent {
  taskId: number;
  entryId: number;
  content: string;
  isDone: boolean;
  agentType: string;
  error?: string;
}

/**
 * 监听 AI 流式事件
 */
export async function listenAiStream(
  callback: (event: AiStreamEvent) => void
): Promise<() => void> {
  if (isTauri()) {
    const { listen } = await import("@tauri-apps/api/event");
    const unlisten = await listen<AiStreamEvent>(AI_STREAM_EVENT, (event) => {
      callback(event.payload);
    });
    return unlisten;
  }
  return () => {};
}
