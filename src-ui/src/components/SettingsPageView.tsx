import { useState, useEffect } from "react";
import { Button, Input, Dropdown } from "@/components/ui";
import {
  listProviders,
  addProvider as apiAddProvider,
  updateProvider as apiUpdateProvider,
  deleteProvider as apiDeleteProvider,
  listProviderModels,
  addProviderModel,
  validateProvider,
} from "@/api/provider";
import type { Provider, AgentConfig, ImportResult, Tag, TagAlias, DuplicateTagPair } from "@/lib/types";
import { useTheme } from "@/contexts/ThemeContext";
import { useApp } from "@/contexts/AppContext";
import { t } from "@/lib/utils";
import { isTauri, exportOpml, importOpml, listTags, addTag, updateTag, deleteTag, getTagStats, mergeTags, addTagAlias, removeTagAlias, getTagAliases, findPotentialDuplicates, findUnusedTags, deleteUnusedTags } from "@/api/feed";
import { toast } from "@/components/ui/Toast";

const TAG_COLORS = [
  "#ef4444", "#f97316", "#f59e0b", "#84cc16", "#22c55e",
  "#10b981", "#14b8a6", "#06b6d4", "#0ea5e9", "#3b82f6",
  "#6366f1", "#8b5cf6", "#a855f7", "#d946ef", "#ec4899",
];

export function SettingsPageView() {
  const { theme, toggleTheme } = useTheme();
  const [activeSection, setActiveSection] = useState<string>("provider");

  const sections = [
    { key: "provider", label: t("AI 服务") },
    { key: "agent", label: t("Agent 参数") },
    { key: "appearance", label: t("外观") },
    { key: "sync", label: t("同步") },
    { key: "tags", label: t("标签") },
    { key: "about", label: t("关于") },
  ];

  return (
    <div className="flex-1 flex overflow-hidden">
      {/* Settings sidebar */}
      <div className="w-[200px] border-r border-[var(--border-color)] py-4 bg-[var(--bg-secondary)]">
        <h2 className="px-5 text-sm font-semibold mb-3">{t("设置")}</h2>
        <nav className="space-y-0.5">
          {sections.map((s) => (
            <button
              key={s.key}
              onClick={() => setActiveSection(s.key)}
              className={`w-full text-left px-5 py-2 text-sm transition-colors ${
                activeSection === s.key
                  ? "bg-[var(--bg-tertiary)] text-[var(--text-primary)] font-medium border-r-2 border-r-[var(--accent-color)]"
                  : "text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)] hover:text-[var(--text-primary)]"
              }`}
            >
              {s.label}
            </button>
          ))}
        </nav>
      </div>

      {/* Settings content */}
      <div className="flex-1 overflow-y-auto px-8 py-6">
        {activeSection === "provider" && <ProviderSettings />}
        {activeSection === "agent" && <AgentSettings />}
        {activeSection === "appearance" && <AppearanceSettings theme={theme} onToggleTheme={toggleTheme} />}
        {activeSection === "sync" && <SyncSettings />}
        {activeSection === "tags" && <TagManagement />}
        {activeSection === "about" && <AboutSection />}
      </div>
    </div>
  );
}

function ProviderSettings() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [providersLoading, setProvidersLoading] = useState(true);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [showAdd, setShowAdd] = useState(false);
  const [form, setForm] = useState({ name: "", baseUrl: "", apiKeyRef: "", isDefault: false, defaultModel: "" });
  const [models, setModels] = useState<Record<number, string[]>>({});
  const [validated, setValidated] = useState<Record<number, boolean | null>>(() => {
    try {
      const saved = localStorage.getItem("providerValidated");
      return saved ? JSON.parse(saved) : {};
    } catch { return {}; }
  });
  const [validating, setValidating] = useState<Record<number, boolean>>({});

  const doValidate = async (p: Provider) => {
    setValidating(prev => ({ ...prev, [p.id]: true }));
    try {
      const modelName = (models[p.id] && models[p.id].length > 0) ? models[p.id][0] : "gpt-3.5-turbo";
      const ok = await validateProvider(p.baseUrl, p.apiKeyRef, modelName);
      setValidated(prev => {
        const next = { ...prev, [p.id]: ok };
        localStorage.setItem("providerValidated", JSON.stringify(next));
        return next;
      });
    } catch {
      setValidated(prev => {
        const next = { ...prev, [p.id]: false as boolean | null };
        localStorage.setItem("providerValidated", JSON.stringify(next));
        return next;
      });
    } finally {
      setValidating(prev => ({ ...prev, [p.id]: false }));
    }
  };

  const loadProviders = async () => {
    setProvidersLoading(true);
    if (!isTauri()) {
      setProviders([]);
      setModels({});
      setProvidersLoading(false);
      return;
    }
    try {
      const data = await listProviders();
      setProviders(data);

      // 加载每个 provider 的模型列表
      const modelsMap: Record<number, string[]> = {};
      for (const p of data) {
        try {
          const ms = await listProviderModels(p.id);
          modelsMap[p.id] = ms.map((m: any) => m.modelName);
        } catch {
          modelsMap[p.id] = [];
        }
      }
      setModels(modelsMap);
    } catch (e: any) {
      console.error("Failed to load providers", e);
      setProviders([]);
      setModels({});
    } finally {
      setProvidersLoading(false);
    }
  };

  useEffect(() => {
    loadProviders();
  }, []);

  const handleAdd = async () => {
    if (!form.name || !form.baseUrl) return;
    if (!isTauri()) return;

    try {
      const newProv = await apiAddProvider({
        name: form.name,
        baseUrl: form.baseUrl,
        apiKeyRef: form.apiKeyRef,
        isDefault: providers.length === 0 ? true : form.isDefault,
      });

      // 如果填写了默认模型，添加到 provider_models
      if (form.defaultModel) {
        await addProviderModel({
          providerId: newProv.id,
          modelName: form.defaultModel,
          isDefault: true,
        });
      }

      setForm({ name: "", baseUrl: "", apiKeyRef: "", isDefault: false, defaultModel: "" });
      setShowAdd(false);
      await loadProviders();
      // Auto-validate the new provider
      doValidate(newProv);
    } catch (e: any) {
      toast(t("添加失败: ") + String(e), "error");
    }
  };

  const handleDelete = async (id: number) => {
    if (!isTauri()) return;
    try {
      await apiDeleteProvider(id);
      await loadProviders();
    } catch (e: any) {
      toast(t("删除失败: ") + String(e), "error");
    }
  };

  const handleUpdate = async (id: number, update: any) => {
    if (!isTauri()) return;
    try {
      await apiUpdateProvider(id, update);
      setEditingId(null);
      await loadProviders();
      toast(t("更新成功"), "success");
    } catch (e: any) {
      toast(t("更新失败: ") + String(e), "error");
    }
  };

  if (providersLoading) {
    return <div className="text-center py-8 text-sm text-[var(--text-tertiary)]">{t("加载中...")}</div>;
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <div>
          <h3 className="text-lg font-semibold">{t("AI 服务配置")}</h3>
          <p className="text-sm text-[var(--text-tertiary)] mt-1">
            {t("配置 OpenAI 兼容的 API 服务，支持云端和本地大模型")}
          </p>
        </div>
        <Button size="sm" onClick={() => setShowAdd(true)}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
          </svg>
          {t("添加 Provider")}
        </Button>
      </div>

      {/* Provider list */}
      <div className="space-y-3">
        {providers.length === 0 && (
          <div className="text-center py-8 text-sm text-[var(--text-tertiary)]">
            {t("还没有配置 AI 服务，点击上方按钮添加")}
          </div>
        )}
        {providers.map((p) => (
          <div key={p.id} className="rounded-xl border border-[var(--border-color)] bg-[var(--bg-secondary)] p-4">
            {editingId === p.id ? (
              <EditProviderFormInline
                provider={p}
                models={models[p.id] || []}
                onSave={(update) => handleUpdate(p.id, update)}
                onCancel={() => setEditingId(null)}
              />
            ) : (
              <div className="flex items-center justify-between">
                <div>
                  <div className="flex items-center gap-2">
                    <h4 className="font-medium text-sm">{p.name}</h4>
                    {p.isDefault && (
                      <span className="text-xs px-2 py-0.5 rounded-full bg-[var(--accent-color)]/10 text-[var(--accent-color)]">
                        {t("默认")}
                      </span>
                    )}
                    {validated[p.id] === true && (
                      <span className="text-xs px-2 py-0.5 rounded-full bg-green-100 dark:bg-green-900 text-green-700 dark:text-green-300">
                        {t("已连接")}
                      </span>
                    )}
                    {validated[p.id] === false && (
                      <span className="text-xs px-2 py-0.5 rounded-full bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-300">
                        {t("未连接")}
                      </span>
                    )}
                    {validated[p.id] === undefined && (
                      <span className="text-xs px-2 py-0.5 rounded-full bg-gray-100 dark:bg-gray-800 text-gray-500">
                        {t("未验证")}
                      </span>
                    )}
                  </div>
                  <p className="text-xs text-[var(--text-tertiary)] mt-1">{p.baseUrl}</p>
                  {models[p.id] && models[p.id]!.length > 0 && (
                    <div className="flex items-center gap-2 mt-2 text-xs text-[var(--text-tertiary)]">
                      <span>{t("模型：")}</span>
                      {models[p.id]!.slice(0, 3).join(", ")}
                      {models[p.id]!.length > 3 && <span>+{models[p.id]!.length - 3}</span>}
                    </div>
                  )}
                </div>
                <div className="flex items-center gap-1">
                  <Button variant="ghost" size="sm" onClick={() => doValidate(p)} disabled={validating[p.id]}>
                    {validating[p.id] ? "..." : t("验证")}
                  </Button>
                  <Button variant="ghost" size="sm" onClick={() => setEditingId(p.id)}>
                    {t("编辑")}
                  </Button>
                  <Button variant="ghost" size="sm" onClick={() => handleDelete(p.id)}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <path strokeLinecap="round" strokeLinejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </Button>
                </div>
              </div>
            )}
          </div>
        ))}
      </div>

      {/* Add form */}
      {showAdd && (
        <div className="mt-4 rounded-xl border border-[var(--border-color)] bg-[var(--bg-secondary)] p-4">
          <h4 className="font-medium text-sm mb-3">{t("新建 Provider")}</h4>
          <div className="grid grid-cols-2 gap-3">
            <Input placeholder={t("名称（如 Ollama）")} value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} />
            <Input placeholder={t("Base URL（如 http://localhost:11434/v1）")} value={form.baseUrl} onChange={(e) => setForm({ ...form, baseUrl: e.target.value })} />
            <Input placeholder={t("API Key（可选）")} type="password" value={form.apiKeyRef} onChange={(e) => setForm({ ...form, apiKeyRef: e.target.value })} />
            <Input placeholder={t("默认模型名")} value={form.defaultModel} onChange={(e) => setForm({ ...form, defaultModel: e.target.value })} />
          </div>
          <div className="flex justify-end gap-2 mt-3">
            <Button variant="ghost" size="sm" onClick={() => setShowAdd(false)}>{t("取消")}</Button>
            <Button size="sm" onClick={handleAdd}>{t("添加")}</Button>
          </div>
        </div>
      )}
    </div>
  );
}

function EditProviderFormInline({
  provider,
  models,
  onSave,
  onCancel,
}: {
  provider: Provider;
  models: string[];
  onSave: (update: any) => void;
  onCancel: () => void;
}) {
  const [name, setName] = useState(provider.name);
  const [baseUrl, setBaseUrl] = useState(provider.baseUrl);
  const [apiKeyRef, setApiKeyRef] = useState(provider.apiKeyRef);
  return (
    <div>
      <div className="grid grid-cols-2 gap-3">
        <Input placeholder={t("名称")} value={name} onChange={(e) => setName(e.target.value)} />
        <Input placeholder="Base URL" value={baseUrl} onChange={(e) => setBaseUrl(e.target.value)} />
        <Input placeholder="API Key" type="password" value={apiKeyRef} onChange={(e) => setApiKeyRef(e.target.value)} />
      </div>
      {models.length > 0 && (
        <p className="text-xs text-[var(--text-tertiary)] mt-2">
          {t("模型：")}
          {models.join(", ")}
        </p>
      )}
      <div className="flex justify-end gap-2 mt-3">
        <Button variant="ghost" size="sm" onClick={onCancel}>{t("取消")}</Button>
        <Button size="sm" onClick={() => onSave({ name, baseUrl, apiKeyRef })}>{t("保存")}</Button>
      </div>
    </div>
  );
}

function EditProviderForm({
  provider,
  onSave,
  onCancel,
}: {
  provider: Provider;
  onSave: (p: Provider) => void;
  onCancel: () => void;
}) {
  const [form, setForm] = useState(provider);
  return (
    <div>
      <div className="grid grid-cols-2 gap-3">
        <Input placeholder={t("名称")} value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} />
        <Input placeholder="Base URL" value={form.baseUrl} onChange={(e) => setForm({ ...form, baseUrl: e.target.value })} />
        <Input placeholder="API Key" type="password" value={form.apiKeyRef || ""} onChange={(e) => setForm({ ...form, apiKeyRef: e.target.value })} />
      </div>
      <div className="flex justify-end gap-2 mt-3">
        <Button variant="ghost" size="sm" onClick={onCancel}>{t("取消")}</Button>
        <Button size="sm" onClick={() => onSave(form)}>{t("保存")}</Button>
      </div>
    </div>
  );
}

function AgentSettings() {
  const [config, setConfig] = useState<AgentConfig>(() => {
    try {
      const saved = localStorage.getItem("agentConfig");
      if (saved) return JSON.parse(saved);
    } catch {}
    return {
      summaryLanguage: "zh-CN",
      summaryDetail: "standard",
      translationLanguage: "中文",
      concurrencyDegree: 3,
    };
  });

  useEffect(() => {
    localStorage.setItem("agentConfig", JSON.stringify(config));
  }, [config]);

  const languages = [
    { label: t("中文"), value: "zh-CN" },
    { label: "English", value: "en" },
    { label: "日本語", value: "ja" },
    { label: "한국어", value: "ko" },
  ];

  const detailLevels = [
    { label: t("简洁"), value: "brief" },
    { label: t("标准"), value: "standard" },
    { label: t("详细"), value: "detailed" },
  ];

  return (
    <div>
      <h3 className="text-lg font-semibold mb-1">{t("Agent 参数配置")}</h3>
      <p className="text-sm text-[var(--text-tertiary)] mb-6">{t("配置 AI 摘要和翻译的默认参数")}</p>

      <div className="space-y-6 max-w-md">
        <div>
          <label className="block text-sm font-medium mb-2">{t("摘要目标语言")}</label>
          <Dropdown
            items={languages}
            value={config.summaryLanguage}
            onChange={(v) => setConfig({ ...config, summaryLanguage: v })}
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">{t("摘要详细程度")}</label>
          <Dropdown
            items={detailLevels}
            value={config.summaryDetail}
            onChange={(v) => setConfig({ ...config, summaryDetail: v as AgentConfig["summaryDetail"] })}
          />
        </div>

        <div className="border-t border-[var(--border-color)] pt-4">
          <label className="block text-sm font-medium mb-2">{t("翻译目标语言")}</label>
          <Dropdown
            items={[
              { label: t("中文"), value: "zh-CN" },
              { label: "English", value: "en" },
              { label: "日本語", value: "ja" },
              { label: "한국어", value: "ko" },
              { label: "Français", value: "fr" },
              { label: "Deutsch", value: "de" },
            ]}
            value={config.translationLanguage}
            onChange={(v) => setConfig({ ...config, translationLanguage: v })}
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">{t("翻译并发度：")}{config.concurrencyDegree}</label>
          <input
            type="range"
            min="1"
            max="5"
            value={config.concurrencyDegree}
            onChange={(e) => setConfig({ ...config, concurrencyDegree: Number(e.target.value) })}
            className="w-full accent-[var(--accent-color)]"
          />
          <div className="flex justify-between text-xs text-[var(--text-tertiary)] mt-1">
            <span>{t("1（慢）")}</span>
            <span>{t("5（快）")}</span>
          </div>
        </div>
      </div>
    </div>
  );
}

function AppearanceSettings({ theme, onToggleTheme }: { theme: string; onToggleTheme: () => void }) {
  const { fontFamily, setFontFamily, codeFontFamily, setCodeFontFamily } = useTheme();
  const [systemFonts, setSystemFonts] = useState<string[]>([
    "system-ui, sans-serif",
    "Georgia, Noto Serif SC, serif",
    "Inter, Noto Sans SC, sans-serif",
    "Source Han Sans SC, sans-serif",
    "LXGW WenKai, serif",
    "Noto Serif SC, serif",
  ]);

  useEffect(() => {
    if (isTauri()) {
      import("@/api/feed").then(({ listSystemFonts }) => {
        listSystemFonts().then(setSystemFonts).catch(() => {});
      });
    }
  }, []);

  return (
    <div>
      <h3 className="text-lg font-semibold mb-1">{t("外观设置")}</h3>
      <p className="text-sm text-[var(--text-tertiary)] mb-6">{t("自定义阅读体验")}</p>

      <div className="space-y-6 max-w-md">
        <div>
          <label className="block text-sm font-medium mb-2">{t("主题模式")}</label>
          <div className="flex items-center gap-3">
            <button
              onClick={onToggleTheme}
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                theme === "dark" ? "bg-[var(--accent-color)]" : "bg-[var(--border-color)]"
              }`}
            >
              <span
                className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                  theme === "dark" ? "translate-x-6" : "translate-x-1"
                }`}
              />
            </button>
            <span className="text-sm text-[var(--text-secondary)]">
              {theme === "dark" ? t("暗色模式") : t("亮色模式")}
            </span>
          </div>
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">{t("阅读字体")}</label>
          <Dropdown
            items={systemFonts.map(f => {
              const primaryName = f.split(",")[0].trim();
              return { label: primaryName, value: f, fontFamily: f };
            })}
            value={fontFamily}
            onChange={(v) => setFontFamily(v)}
          />
          <p className="text-xs text-[var(--text-tertiary)] mt-2" style={{ fontFamily }}>
            {t("预览：")}The quick brown fox  jumps over the lazy dog. 静态动词优化 了编译速度。
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">{t("代码字体")}</label>
          <Dropdown
            items={[
              { label: "JetBrains Mono", value: "JetBrains Mono", fontFamily: "JetBrains Mono" },
              { label: "Fira Code", value: "Fira Code", fontFamily: "Fira Code" },
              { label: "Cascadia Code", value: "Cascadia Code", fontFamily: "Cascadia Code" },
              { label: "Consolas", value: "Consolas", fontFamily: "Consolas" },
              { label: "monospace", value: "monospace", fontFamily: "monospace" },
            ]}
            value={codeFontFamily}
            onChange={(v) => setCodeFontFamily(v)}
          />
        </div>
      </div>
    </div>
  );
}

function SyncSettings() {
  const { reloadFeeds } = useApp();
  const [importing, setImporting] = useState(false);
  const [importProgress, setImportProgress] = useState<ImportResult[]>([]);
  const handleOpmlExport = async () => {
    try {
      if (!isTauri()) { toast(t("仅在桌面应用中可用"), "error"); return; }
      const { save } = await import("@tauri-apps/plugin-dialog");
      const { homeDir } = await import("@tauri-apps/api/path");
      const home = await homeDir();
      const filePath = await save({
        defaultPath: `${home}subscriptions.opml`,
        filters: [{ name: "OPML", extensions: ["opml"] }],
      });
      if (!filePath) return; // user cancelled
      await exportOpml(filePath);
      toast(t("已导出到 ") + filePath, "success");
    } catch (e: any) {
      toast(t("导出失败: ") + String(e), "error");
    }
  };

  const handleOpmlImport = async () => {
    try {
      if (!isTauri()) { toast(t("仅在桌面应用中可用"), "error"); return; }
      const { open } = await import("@tauri-apps/plugin-dialog");
      const filePath = await open({
        multiple: false,
        filters: [{ name: "OPML", extensions: ["opml", "xml"] }],
      });
      if (!filePath) return;
      setImporting(true);
      setImportProgress([]);
      const { listen } = await import("@tauri-apps/api/event");
      const unlisten = await listen<ImportResult>("opml-import-progress", (event) => {
        setImportProgress((prev) => [...prev, event.payload]);
        reloadFeeds();
      });
      const results = await importOpml(filePath as string);
      unlisten();
      const ok = results.filter((r) => r.success).length; 
      const fail = results.length - ok;
      toast(fail > 0 ? t(`导入完成: ${ok} 成功, ${fail} 失败`) : t(`导入完成: ${ok} 个订阅源`), ok > 0 ? "success" : "error");
      reloadFeeds();
      setImporting(false);
    } catch (e: any) {
      toast(t("导入失败: ") + String(e), "error");
      setImporting(false);
    }
  };

  return (
    <div>
      <h3 className="text-lg font-semibold mb-1">{t("同步设置")}</h3>
      <p className="text-sm text-[var(--text-tertiary)] mb-6">{t("配置订阅源自动同步")}</p>

      <div className="space-y-6 max-w-md">
        <div>
          <label className="block text-sm font-medium mb-2">{t("自动同步间隔")}</label>
          <Dropdown
            items={[
              { label: t("手动同步"), value: "0" },
              { label: t("每 15 分钟"), value: "15" },
              { label: t("每 30 分钟"), value: "30" },
              { label: t("每小时"), value: "60" },
              { label: t("每 2 小时"), value: "120" },
            ]}
            value="30"
            onChange={() => {}}
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">{t("并发同步数")}</label>
          <Dropdown
            items={[
              { label: "1", value: "1" },
              { label: "3", value: "3" },
              { label: "5", value: "5" },
            ]}
            value="5"
            onChange={() => {}}
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">{t("OPML 导入")}</label>
          <Button variant="secondary" size="sm" onClick={() => handleOpmlImport()} disabled={importing}>
            {importing ? t("导入中...") : t("导入订阅源")}
          </Button>
          {importing && (
            <div className="mt-2 text-xs text-[var(--text-tertiary)] max-h-32 overflow-y-auto">
              {importProgress.map((r, i) => (
                <div key={i} className="flex items-center gap-1 py-0.5">
                  <span className={r.success ? "text-green-500" : "text-red-500"}>
                    {r.success ? "✓" : "✗"}
                  </span>
                  <span className="truncate">{r.title || r.xmlUrl}</span>
                </div>
              ))}
              {importProgress.length === 0 && <span>{t("准备中...")}</span>}
            </div>
          )}
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">{t("OPML 导出")}</label>
          <Button variant="secondary" size="sm" onClick={() => handleOpmlExport()}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            {t("导出订阅源")}
          </Button>
        </div>
      </div>
    </div>
  );
}

function AboutSection() {
  return (
    <div>
      <h3 className="text-lg font-semibold mb-1">关于</h3>
      <p className="text-sm text-[var(--text-tertiary)] mb-6">Platinum — Mercury 跨平台复刻</p>

      <div className="space-y-3 text-sm text-[var(--text-secondary)] max-w-md">
        <div className="flex justify-between py-2 border-b border-[var(--border-color)]">
          <span className="text-[var(--text-tertiary)]">{t("版本")}</span>
          <span>v0.2.0 (Stage 2)</span>
        </div>
        <div className="flex justify-between py-2 border-b border-[var(--border-color)]">
          <span className="text-[var(--text-tertiary)]">{t("技术栈")}</span>
          <span>Tauri 2 + Rust + React + SQLite</span>
        </div>
        <div className="flex justify-between py-2 border-b border-[var(--border-color)]">
          <span className="text-[var(--text-tertiary)]">{t("平台")}</span>
          <span>Windows / macOS / Linux</span>
        </div>
        <div className="flex justify-between py-2 border-b border-[var(--border-color)]">
          <span className="text-[var(--text-tertiary)]">{t("许可证")}</span>
          <span>MIT</span>
        </div>
      </div>
    </div>
  );
}

function TagManagement() {
  const { reloadTags } = useApp();
  const [tags, setTags] = useState<Tag[]>([]);
  const [stats, setStats] = useState<Record<number, number>>({});
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedTag, setSelectedTag] = useState<Tag | null>(null);
  const [selectedTab, setSelectedTab] = useState<"library" | "duplicates" | "unused">("library");
  const [isCreating, setIsCreating] = useState(false);
  
  const [editName, setEditName] = useState("");
  const [editColor, setEditColor] = useState("#3b82f6");
  
  const [aliases, setAliases] = useState<TagAlias[]>([]);
  const [newAlias, setNewAlias] = useState("");
  
  const [mergeTarget, setMergeTarget] = useState<number | null>(null);
  const [duplicates, setDuplicates] = useState<DuplicateTagPair[]>([]);
  const [unusedTags, setUnusedTags] = useState<Tag[]>([]);
  const [selectedUnused, setSelectedUnused] = useState<number[]>([]);

  const loadTags = async () => {
    setLoading(true);
    try {
      const data = await listTags();
      setTags(data);
      const statsMap: Record<number, number> = {};
      for (const tag of data) {
        try {
          const s = await getTagStats(tag.id);
          statsMap[tag.id] = s.entryCount;
        } catch {
          statsMap[tag.id] = tag.usageCount || 0;
        }
      }
      setStats(statsMap);
    } catch (e: any) {
      console.error("Failed to load tags", e);
      setTags([]);
      setStats({});
    } finally {
      setLoading(false);
    }
  };

  const loadAliases = async (tagId: number) => {
    try {
      const data = await getTagAliases(tagId);
      setAliases(data);
    } catch {
      setAliases([]);
    }
  };

  const loadDuplicates = async () => {
    try {
      const data = await findPotentialDuplicates();
      setDuplicates(data);
    } catch {
      setDuplicates([]);
    }
  };

  const loadUnusedTags = async () => {
    try {
      const data = await findUnusedTags();
      setUnusedTags(data);
      setSelectedUnused(data.map(t => t.id));
    } catch {
      setUnusedTags([]);
    }
  };

  useEffect(() => {
    loadTags();
  }, []);

  useEffect(() => {
    if (selectedTag) {
      setEditName(selectedTag.name);
      setEditColor(selectedTag.color);
      loadAliases(selectedTag.id);
    } else {
      setAliases([]);
    }
  }, [selectedTag]);

  useEffect(() => {
    if (selectedTab === "duplicates") {
      loadDuplicates();
    } else if (selectedTab === "unused") {
      loadUnusedTags();
    }
  }, [selectedTab]);

  const filteredTags = tags.filter(tag => 
    tag.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const handleAddTag = async () => {
    if (!editName.trim()) return;
    try {
      await addTag(editName.trim(), editColor);
      setEditName("");
      setEditColor("#3b82f6");
      setIsCreating(false);
      await loadTags();
      reloadTags();
      toast(t("标签已添加"), "success");
    } catch (e: any) {
      toast(t("添加失败: ") + String(e), "error");
    }
  };

  const handleUpdateTag = async () => {
    if (!selectedTag || !editName.trim()) return;
    try {
      await updateTag(selectedTag.id, editName.trim(), editColor);
      await loadTags();
      setSelectedTag(tags.find(t => t.id === selectedTag.id) || null);
      reloadTags();
      toast(t("标签已更新"), "success");
    } catch (e: any) {
      toast(t("更新失败: ") + String(e), "error");
    }
  };

  const handleDeleteTag = async () => {
    if (!selectedTag) return;
    try {
      await deleteTag(selectedTag.id);
      setSelectedTag(null);
      await loadTags();
      reloadTags();
      toast(t("标签已删除"), "success");
    } catch (e: any) {
      toast(t("删除失败: ") + String(e), "error");
    }
  };

  const handleAddAlias = async () => {
    if (!selectedTag || !newAlias.trim()) return;
    try {
      await addTagAlias(selectedTag.id, newAlias.trim());
      setNewAlias("");
      await loadAliases(selectedTag.id);
      toast(t("别名已添加"), "success");
    } catch (e: any) {
      toast(t("添加失败: ") + String(e), "error");
    }
  };

  const handleRemoveAlias = async (alias: string) => {
    if (!selectedTag) return;
    try {
      await removeTagAlias(selectedTag.id, alias);
      await loadAliases(selectedTag.id);
      toast(t("别名已删除"), "success");
    } catch (e: any) {
      toast(t("删除失败: ") + String(e), "error");
    }
  };

  const handleMerge = async () => {
    if (!selectedTag || mergeTarget === null) return;
    try {
      await mergeTags(selectedTag.id, mergeTarget);
      setSelectedTag(null);
      setMergeTarget(null);
      await loadTags();
      reloadTags();
      toast(t("标签已合并"), "success");
    } catch (e: any) {
      toast(t("合并失败: ") + String(e), "error");
    }
  };

  const handleDeleteUnused = async () => {
    if (selectedUnused.length === 0) return;
    try {
      await deleteUnusedTags();
      await loadUnusedTags();
      toast(t(`${selectedUnused.length} 个标签已删除`), "success");
    } catch (e: any) {
      toast(t("删除失败: ") + String(e), "error");
    }
  };

  const handleMergeDuplicate = async (pair: DuplicateTagPair) => {
    try {
      await mergeTags(pair.tagB.id, pair.tagA.id);
      await loadDuplicates();
      await loadTags();
      reloadTags();
      toast(t("标签已合并"), "success");
    } catch (e: any) {
      toast(t("合并失败: ") + String(e), "error");
    }
  };

  const getReasonLabel = (reason: string) => {
    switch (reason) {
      case "plural_variant": return t("复数变体");
      case "naming_variant": return t("命名变体");
      case "spelling_variant": return t("拼写变体");
      default: return reason;
    }
  };

  if (loading) {
    return <div className="text-center py-8 text-sm text-[var(--text-tertiary)]">{t("加载中...")}</div>;
  }

  return (
    <div className="flex h-full">
      <div className="w-80 border-r border-[var(--border-color)] flex flex-col">
        <div className="p-4 border-b border-[var(--border-color)]">
          <div className="flex items-center gap-2 mb-3">
            <Button variant="ghost" size="sm" onClick={() => setSelectedTab("library")}>
              {t("标签库")}
            </Button>
            <Button variant="ghost" size="sm" onClick={() => setSelectedTab("duplicates")}>
              {t("重复检测")}
            </Button>
            <Button variant="ghost" size="sm" onClick={() => setSelectedTab("unused")}>
              {t("未使用")}
            </Button>
          </div>
          {selectedTab === "library" && (
            <Input
              placeholder={t("搜索标签...")}
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          )}
        </div>

        <div className="flex-1 overflow-y-auto">
          {selectedTab === "library" && (
            <div className="p-2">
              <button
                onClick={() => { setSelectedTag(null); setIsCreating(true); setEditName(""); setEditColor("#3b82f6"); }}
                className={`w-full text-left px-3 py-2 rounded-lg text-sm mb-1 transition-colors ${
                  !selectedTag && isCreating ? "bg-[var(--accent-color)] text-white" : "hover:bg-[var(--bg-tertiary)]"
                }`}
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="inline-block mr-2" style={{ verticalAlign: "middle" }}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
                </svg>
                {t("新建标签")}
              </button>
              {filteredTags.length === 0 ? (
                <div className="text-center py-4 text-xs text-[var(--text-tertiary)]">
                  {t("没有找到标签")}
                </div>
              ) : (
                filteredTags.map((tag) => (
                  <button
                    key={tag.id}
                    onClick={() => setSelectedTag(tag)}
                    className={`w-full text-left px-3 py-2 rounded-lg text-sm mb-1 transition-colors flex items-center gap-2 ${
                      selectedTag?.id === tag.id ? "bg-[var(--accent-color)] text-white" : "hover:bg-[var(--bg-tertiary)]"
                    }`}
                  >
                    <div
                      className="w-3 h-3 rounded-full flex-shrink-0"
                      style={{ backgroundColor: tag.color }}
                    />
                    <span className="truncate">{tag.name}</span>
                    <span className={`text-xs ml-auto ${selectedTag?.id === tag.id ? "text-white/70" : "text-[var(--text-tertiary)]"}`}>
                      {stats[tag.id] || 0}
                    </span>
                    {tag.isProvisional && (
                      <span className={`text-xs flex-shrink-0 ${selectedTag?.id === tag.id ? "text-white/70" : "text-[var(--text-tertiary)]"}`}>
                        •
                      </span>
                    )}
                  </button>
                ))
              )}
            </div>
          )}

          {selectedTab === "duplicates" && (
            <div className="p-2">
              {duplicates.length === 0 ? (
                <div className="text-center py-8 text-sm text-[var(--text-tertiary)]">
                  {t("没有检测到重复标签")}
                </div>
              ) : (
                duplicates.map((pair, idx) => (
                  <div key={idx} className="mb-2 p-2 rounded-lg border border-[var(--border-color)]">
                    <div className="flex items-center gap-2 mb-1">
                      <div className="w-2 h-2 rounded-full" style={{ backgroundColor: pair.tagA.color }} />
                      <span className="text-sm">{pair.tagA.name}</span>
                    </div>
                    <div className="flex items-center gap-2 mb-2">
                      <div className="w-2 h-2 rounded-full" style={{ backgroundColor: pair.tagB.color }} />
                      <span className="text-sm">{pair.tagB.name}</span>
                    </div>
                    <div className="flex items-center justify-between">
                      <span className="text-xs text-[var(--text-tertiary)]">{getReasonLabel(pair.reason)}</span>
                      <Button size="sm" onClick={() => handleMergeDuplicate(pair)}>
                        {t("合并")}
                      </Button>
                    </div>
                  </div>
                ))
              )}
            </div>
          )}

          {selectedTab === "unused" && (
            <div className="p-2">
              <div className="flex items-center justify-between mb-2 px-1">
                <span className="text-xs text-[var(--text-tertiary)]">{t(`${unusedTags.length} 个未使用标签`)}</span>
                <button
                  onClick={() => setSelectedUnused(unusedTags.map(t => t.id))}
                  className="text-xs text-[var(--accent-color)] hover:underline"
                >
                  {t("全选")}
                </button>
              </div>
              {unusedTags.length === 0 ? (
                <div className="text-center py-8 text-sm text-[var(--text-tertiary)]">
                  {t("没有未使用的标签")}
                </div>
              ) : (
                unusedTags.map((tag) => (
                  <div key={tag.id} className="flex items-center gap-2 px-2 py-2 rounded-lg mb-1">
                    <input
                      type="checkbox"
                      checked={selectedUnused.includes(tag.id)}
                      onChange={(e) => {
                        if (e.target.checked) {
                          setSelectedUnused([...selectedUnused, tag.id]);
                        } else {
                          setSelectedUnused(selectedUnused.filter(id => id !== tag.id));
                        }
                      }}
                      className="accent-[var(--accent-color)]"
                    />
                    <div className="w-2 h-2 rounded-full" style={{ backgroundColor: tag.color }} />
                    <span className="text-sm">{tag.name}</span>
                  </div>
                ))
              )}
              {selectedUnused.length > 0 && (
                <div className="mt-4 px-2">
                  <Button size="sm" className="w-full" onClick={handleDeleteUnused}>
                    {t(`删除 ${selectedUnused.length} 个标签`)}
                  </Button>
                </div>
              )}
            </div>
          )}
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        {!selectedTag && !isCreating ? (
          <div className="flex flex-col items-center justify-center h-full text-[var(--text-tertiary)]">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1">
              <path strokeLinecap="round" strokeLinejoin="round" d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01" />
            </svg>
            <p className="mt-4 text-sm">{selectedTab === "library" ? t("选择一个标签查看详情，或创建新标签") : t("选择左侧标签查看详情")}</p>
          </div>
        ) : (
          <div className="max-w-xl">
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-lg font-semibold flex items-center gap-2">
                {isCreating ? (
                  <>
                    <div className="w-4 h-4 rounded-full" style={{ backgroundColor: editColor }} />
                    {t("新建标签")}
                  </>
                ) : (
                  <>
                    <div className="w-4 h-4 rounded-full" style={{ backgroundColor: selectedTag!.color }} />
                    {selectedTag!.name}
                    {selectedTag!.isProvisional && (
                      <span className="text-xs px-2 py-0.5 rounded-full bg-yellow-100 dark:bg-yellow-900 text-yellow-700 dark:text-yellow-300">
                        {t("临时")}
                      </span>
                    )}
                  </>
                )}
              </h3>
              {!isCreating && (
                <Button variant="ghost" size="sm" onClick={handleDeleteTag}>
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                  {t("删除")}
                </Button>
              )}
            </div>

            <div className="space-y-6">
              <div className="rounded-xl border border-[var(--border-color)] bg-[var(--bg-secondary)] p-4">
                <h4 className="font-medium text-sm mb-3">{t("基本信息")}</h4>
                <div className="space-y-3">
                  <div>
                    <label className="block text-xs text-[var(--text-tertiary)] mb-1">{t("标签名称")}</label>
                    <Input
                      value={editName}
                      onChange={(e) => setEditName(e.target.value)}
                      className="flex-1"
                    />
                  </div>
                  <div>
                    <label className="block text-xs text-[var(--text-tertiary)] mb-1">{t("颜色")}</label>
                    <div className="flex gap-1">
                      {TAG_COLORS.map((color) => (
                        <button
                          key={color}
                          onClick={() => setEditColor(color)}
                          className={`w-6 h-6 rounded-full border-2 transition-transform ${
                            editColor === color ? "border-[var(--accent-color)] scale-110" : "border-transparent"
                          }`}
                          style={{ backgroundColor: color }}
                        />
                      ))}
                    </div>
                  </div>
                  {!isCreating && (
                    <>
                      <div className="flex items-center justify-between py-2 border-t border-[var(--border-color)]">
                        <span className="text-sm text-[var(--text-tertiary)]">{t("使用次数")}</span>
                        <span className="text-sm font-medium">{stats[selectedTag!.id] || 0}</span>
                      </div>
                      <div className="flex items-center justify-between py-2 border-t border-[var(--border-color)]">
                        <span className="text-sm text-[var(--text-tertiary)]">{t("创建时间")}</span>
                        <span className="text-sm">{selectedTag!.createdAt}</span>
                      </div>
                    </>
                  )}
                </div>
                <div className="flex justify-end gap-2 mt-4">
                  <Button variant="ghost" size="sm" onClick={() => { setSelectedTag(null); setIsCreating(false); }}>{t("取消")}</Button>
                  <Button size="sm" onClick={isCreating ? handleAddTag : handleUpdateTag}>{t("保存")}</Button>
                </div>
              </div>

              {!isCreating && (
                <div className="rounded-xl border border-[var(--border-color)] bg-[var(--bg-secondary)] p-4">
                  <h4 className="font-medium text-sm mb-3">{t("别名")}</h4>
                  <div className="flex gap-2 mb-3">
                    <Input
                      placeholder={t("添加别名")}
                      value={newAlias}
                      onChange={(e) => setNewAlias(e.target.value)}
                      className="flex-1"
                    />
                    <Button size="sm" onClick={handleAddAlias}>{t("添加")}</Button>
                  </div>
                  {aliases.length === 0 ? (
                    <p className="text-xs text-[var(--text-tertiary)]">{t("暂无别名")}</p>
                  ) : (
                    <div className="flex flex-wrap gap-2">
                      {aliases.map((alias) => (
                        <span key={alias.id} className="inline-flex items-center gap-1 px-2 py-1 rounded-full bg-[var(--bg-tertiary)] text-sm">
                          {alias.alias}
                          <button onClick={() => handleRemoveAlias(alias.alias)} className="hover:text-red-500">
                            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                              <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                          </button>
                        </span>
                      ))}
                    </div>
                  )}
                </div>
              )}

              {!isCreating && (
                <div className="rounded-xl border border-[var(--border-color)] bg-[var(--bg-secondary)] p-4">
                  <h4 className="font-medium text-sm mb-3">{t("合并标签")}</h4>
                  <p className="text-xs text-[var(--text-tertiary)] mb-3">
                    {t("将此标签的所有文章移动到另一个标签，然后删除此标签")}
                  </p>
                  <Dropdown
                    items={tags.filter(t => t.id !== selectedTag!.id).map(t => ({
                      label: t.name,
                      value: String(t.id),
                    }))}
                    value={mergeTarget !== null ? String(mergeTarget) : ""}
                    onChange={(v) => setMergeTarget(v ? Number(v) : null)}
                    placeholder={t("选择目标标签")}
                  />
                  {mergeTarget !== null && (
                    <Button size="sm" className="mt-3" onClick={handleMerge}>
                      {t("合并到选中标签")}
                    </Button>
                  )}
                </div>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}