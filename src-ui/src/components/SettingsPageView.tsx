import { useState } from "react";
import { Button, Input, Dropdown } from "@/components/ui";
import { mockProviders } from "@/api/mock";
import type { Provider, AgentConfig } from "@/lib/types";
import { useTheme } from "@/contexts/ThemeContext";
import { t } from "@/lib/utils";
import { isTauri, exportOpml, importOpml } from "@/api/feed";
import { toast } from "@/components/ui/Toast";

export function SettingsPageView() {
  const { theme, toggleTheme } = useTheme();
  const [activeSection, setActiveSection] = useState<string>("provider");

  const sections = [
    { key: "provider", label: t("AI 服务") },
    { key: "agent", label: t("Agent 参数") },
    { key: "appearance", label: t("外观") },
    { key: "sync", label: t("同步") },
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
        {activeSection === "about" && <AboutSection />}
      </div>
    </div>
  );
}

function ProviderSettings() {
  const [providers, setProviders] = useState<Provider[]>(mockProviders);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [showAdd, setShowAdd] = useState(false);
  const [form, setForm] = useState({ name: "", base_url: "", api_key: "", default_model: "", thinking_model: "" });

  const handleAdd = () => {
    if (!form.name || !form.base_url) return;
    const newProvider: Provider = {
      id: Date.now(),
      name: form.name,
      base_url: form.base_url,
      api_key: form.api_key,
      default_model: form.default_model,
      thinking_model: form.thinking_model,
      created_at: new Date().toISOString(),
    };
    setProviders([...providers, newProvider]);
    setForm({ name: "", base_url: "", api_key: "", default_model: "", thinking_model: "" });
    setShowAdd(false);
  };

  const handleDelete = (id: number) => {
    setProviders(providers.filter((p) => p.id !== id));
  };

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
        {providers.map((p) => (
          <div key={p.id} className="rounded-xl border border-[var(--border-color)] bg-[var(--bg-secondary)] p-4">
            {editingId === p.id ? (
              <EditProviderForm
                provider={p}
                onSave={(updated) => {
                  setProviders(providers.map((x) => (x.id === p.id ? updated : x)));
                  setEditingId(null);
                }}
                onCancel={() => setEditingId(null)}
              />
            ) : (
              <div className="flex items-center justify-between">
                <div>
                  <div className="flex items-center gap-2">
                    <h4 className="font-medium text-sm">{p.name}</h4>
                    <span className="text-xs px-2 py-0.5 rounded-full bg-green-100 dark:bg-green-900 text-green-700 dark:text-green-300">
                      {t("已连接")}
                    </span>
                  </div>
                  <p className="text-xs text-[var(--text-tertiary)] mt-1">{p.base_url}</p>
                  <div className="flex items-center gap-4 mt-2 text-xs text-[var(--text-tertiary)]">
                    <span>{t("默认模型：")}{p.default_model}</span>
                    <span>{t("思考模型：")}{p.thinking_model}</span>
                  </div>
                </div>
                <div className="flex items-center gap-1">
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
            <Input placeholder={t("Base URL（如 http://localhost:11434/v1）")} value={form.base_url} onChange={(e) => setForm({ ...form, base_url: e.target.value })} />
            <Input placeholder={t("API Key（可选）")} type="password" value={form.api_key} onChange={(e) => setForm({ ...form, api_key: e.target.value })} />
            <Input placeholder={t("默认模型")} value={form.default_model} onChange={(e) => setForm({ ...form, default_model: e.target.value })} />
            <Input placeholder={t("思考模型")} value={form.thinking_model} onChange={(e) => setForm({ ...form, thinking_model: e.target.value })} />
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
        <Input placeholder="Base URL" value={form.base_url} onChange={(e) => setForm({ ...form, base_url: e.target.value })} />
        <Input placeholder="API Key" type="password" value={form.api_key} onChange={(e) => setForm({ ...form, api_key: e.target.value })} />
        <Input placeholder={t("默认模型")} value={form.default_model} onChange={(e) => setForm({ ...form, default_model: e.target.value })} />
        <Input placeholder={t("思考模型")} value={form.thinking_model} onChange={(e) => setForm({ ...form, thinking_model: e.target.value })} />
      </div>
      <div className="flex justify-end gap-2 mt-3">
        <Button variant="ghost" size="sm" onClick={onCancel}>{t("取消")}</Button>
        <Button size="sm" onClick={() => onSave(form)}>{t("保存")}</Button>
      </div>
    </div>
  );
}

function AgentSettings() {
  const [config, setConfig] = useState<AgentConfig>({
    targetLanguage: "zh-CN",
    detailLevel: "standard",
    concurrencyDegree: 3,
    primaryModelId: "",
  });

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
            value={config.targetLanguage}
            onChange={(v) => setConfig({ ...config, targetLanguage: v })}
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">{t("摘要详细程度")}</label>
          <Dropdown
            items={detailLevels}
            value={config.detailLevel}
            onChange={(v) => setConfig({ ...config, detailLevel: v as AgentConfig["detailLevel"] })}
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

        <div>
          <label className="block text-sm font-medium mb-2">{t("默认 AI 模型")}</label>
          <Dropdown
            items={[
              { label: t("跟随 Provider 默认"), value: "" },
              { label: "qwen2.5:7b (Ollama)", value: "qwen2.5:7b" },
              { label: "deepseek-chat (DeepSeek)", value: "deepseek-chat" },
            ]}
            value={config.primaryModelId}
            onChange={(v) => setConfig({ ...config, primaryModelId: v })}
          />
        </div>
      </div>
    </div>
  );
}

function AppearanceSettings({ theme, onToggleTheme }: { theme: string; onToggleTheme: () => void }) {
  const fonts = [
    { label: t("系统默认"), value: "system" },
    { label: "Inter", value: "Inter" },
    { label: t("思源黑体"), value: "Noto Sans SC" },
    { label: t("霞鹜文楷"), value: "LXGW WenKai" },
    { label: "JetBrains Mono", value: "JetBrains Mono" },
  ];

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
            items={fonts}
            value="system"
            onChange={() => {}}
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">{t("代码字体")}</label>
          <Dropdown
            items={[
              { label: "JetBrains Mono", value: "JetBrains Mono" },
              { label: "Fira Code", value: "Fira Code" },
              { label: "Cascadia Code", value: "Cascadia Code" },
            ]}
            value="JetBrains Mono"
            onChange={() => {}}
          />
        </div>
      </div>
    </div>
  );
}

function SyncSettings() {
  const handleOpmlExport = async () => {
    try {
      const home = (window as any).__TAURI__
        ? await import("@tauri-apps/api/path").then(m => m.homeDir())
        : "";
      const path = `${home}subscriptions.opml`;
      await exportOpml(path);
      toast(t("已导出到 ") + path, "success");
    } catch {
      toast(t("导出失败"), "error");
    }
  };

  const handleOpmlImport = async () => {
    try {
      const home = (window as any).__TAURI__
        ? await import("@tauri-apps/api/path").then(m => m.homeDir())
        : "";
      const path = `${home}subscriptions.opml`;
      const results = await importOpml(path);
      const ok = results.filter((r: any) => r.success).length;
      toast(t(`导入完成: ${ok}/${results.length} 个订阅源`), ok > 0 ? "success" : "error");
    } catch {
      toast(t("导入失败"), "error");
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
          <Button variant="secondary" size="sm" onClick={() => handleOpmlImport()}>
            {t("导入订阅源")}
          </Button>
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