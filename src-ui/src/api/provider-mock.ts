import type { Provider, LlmUsageStats } from "@/lib/types";

let providers: Provider[] = [
  { id: 1, name: "OpenAI", baseUrl: "https://api.openai.com/v1", apiKeyRef: "", isDefault: true, createdAt: "2026-07-01T00:00:00Z", updatedAt: "2026-07-01T00:00:00Z" },
  { id: 2, name: "DeepSeek", baseUrl: "https://api.deepseek.com/v1", apiKeyRef: "", isDefault: false, createdAt: "2026-07-02T00:00:00Z", updatedAt: "2026-07-02T00:00:00Z" },
];

let nextProviderId = 3;

export async function mockListProviders(): Promise<Provider[]> {
  return [...providers];
}

export async function mockAddProvider(provider: {
  name: string;
  baseUrl: string;
  apiKeyRef: string;
  isDefault: boolean;
}): Promise<Provider> {
  if (provider.isDefault) {
    providers.forEach(p => p.isDefault = false);
  }
  const now = new Date().toISOString();
  const newProvider: Provider = {
    id: nextProviderId++,
    name: provider.name,
    baseUrl: provider.baseUrl,
    apiKeyRef: provider.apiKeyRef,
    isDefault: provider.isDefault,
    createdAt: now,
    updatedAt: now,
  };
  providers.push(newProvider);
  return newProvider;
}

export async function mockUpdateProvider(
  id: number,
  update: {
    name?: string;
    baseUrl?: string;
    apiKeyRef?: string;
    isDefault?: boolean;
  }
): Promise<Provider> {
  const index = providers.findIndex(p => p.id === id);
  if (index === -1) throw new Error(`Provider id=${id} not found`);
  if (update.isDefault) {
    providers.forEach(p => p.isDefault = false);
  }
  providers[index] = { ...providers[index], ...update };
  return providers[index];
}

export async function mockDeleteProvider(id: number): Promise<void> {
  const index = providers.findIndex(p => p.id === id);
  if (index === -1) throw new Error(`Provider id=${id} not found`);
  providers.splice(index, 1);
}

export async function mockGetLlmUsageStats(days: number): Promise<LlmUsageStats> {
  return {
    totalTokens: 125000,
    promptTokens: 85000,
    completionTokens: 40000,
    requestCount: 320,
    successRate: 98.5,
    avgTokensPerRequest: 390,
  };
}

export async function mockGetDailyLlmUsage(days: number): Promise<{ date: string; totalTokens: number; promptTokens: number; completionTokens: number; requestCount: number }[]> {
  const result = [];
  const now = new Date();
  for (let i = days - 1; i >= 0; i--) {
    const date = new Date(now);
    date.setDate(date.getDate() - i);
    const total = Math.floor(Math.random() * 5000) + 2000;
    const prompt = Math.floor(total * 0.7);
    const completion = total - prompt;
    result.push({
      date: date.toISOString().split("T")[0],
      totalTokens: total,
      promptTokens: prompt,
      completionTokens: completion,
      requestCount: Math.floor(Math.random() * 20) + 5,
    });
  }
  return result;
}

export async function mockGetProviderStats(): Promise<{ providerId: number; providerName: string; totalTokens: number; requestCount: number; successRate: number }[]> {
  return providers.map(p => ({
    providerId: p.id,
    providerName: p.name,
    totalTokens: p.id === 1 ? 85000 : 40000,
    requestCount: p.id === 1 ? 220 : 100,
    successRate: p.id === 1 ? 99.1 : 97.5,
  }));
}

export async function mockGetModelStats(): Promise<{ modelId: number; modelName: string; totalTokens: number; requestCount: number }[]> {
  return [
    { modelId: 1, modelName: "gpt-4o", totalTokens: 65000, requestCount: 150 },
    { modelId: 2, modelName: "gpt-4o-mini", totalTokens: 20000, requestCount: 70 },
    { modelId: 3, modelName: "deepseek-chat", totalTokens: 40000, requestCount: 100 },
  ];
}