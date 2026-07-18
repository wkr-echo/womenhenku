// Prompt 模板管理
//
// Prompt 模板以 YAML 格式存放在 resources/prompts/ 目录下。
// 内置模板只读，用户可覆盖。

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Prompt 模板结构
#[derive(Debug, Clone, Deserialize)]
pub struct PromptTemplate {
    /// 系统提示词
    pub system: String,
    /// 用户提示词（可含占位符）
    pub user: String,
    /// 默认参数
    #[serde(default)]
    pub defaults: HashMap<String, String>,
    /// 模板版本
    pub version: String,
}

/// Prompt 管理器
#[derive(Debug)]
pub struct PromptManager {
    templates: HashMap<String, PromptTemplate>,
}

impl PromptManager {
    /// 从 resources 目录加载所有 prompt 模板
    pub fn load(resources_dir: &Path) -> Result<Self, String> {
        let prompts_dir = resources_dir.join("prompts");
        if !prompts_dir.exists() {
            tracing::warn!("Prompts directory not found: {:?}", prompts_dir);
            return Ok(Self {
                templates: HashMap::new(),
            });
        }

        let mut templates = HashMap::new();
        let entries = std::fs::read_dir(&prompts_dir)
            .map_err(|e| format!("Failed to read prompts directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let content =
                    std::fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                let template: PromptTemplate =
                    serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;
                templates.insert(name, template);
            }
        }

        tracing::info!("Loaded {} prompt templates", templates.len());
        Ok(Self { templates })
    }

    /// 获取指定名称的模板
    pub fn get(&self, name: &str) -> Option<&PromptTemplate> {
        self.templates.get(name)
    }

    /// 渲染 prompt：替换占位符
    pub fn render(&self, name: &str, vars: &HashMap<String, String>) -> Result<(String, String), String> {
        let template = self
            .templates
            .get(name)
            .ok_or_else(|| format!("Prompt template '{}' not found", name))?;

        let system = Self::replace_placeholders(&template.system, vars);
        let user = Self::replace_placeholders(&template.user, vars);

        Ok((system, user))
    }

    fn replace_placeholders(template: &str, vars: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        for (key, value) in vars {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }
        result
    }

    /// 创建一个空的 PromptManager（用于测试或降级模式）
    pub fn empty() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }
}

/// 内置默认 Prompt（当 resources/prompts/ 不存在时作为后备）
pub fn builtin_summary_prompt() -> (String, String) {
    let system = r#"你是一个专业的文章摘要助手。请根据用户指定的语言和详细程度，对文章内容进行摘要总结。

要求：
1. 准确概括文章的核心观点和主要内容
2. 保持客观中立，不添加个人观点
3. 使用清晰简洁的语言
4. 保持原文的关键信息和逻辑结构"#.to_string();

    let user = r#"请用{{target_language}}为以下文章生成{{detail_level}}详细程度的摘要：

文章内容（Markdown 格式）：
{{content}}

请生成摘要："#.to_string();

    (system, user)
}

/// 内置默认翻译 Prompt
pub fn builtin_translation_prompt() -> (String, String) {
    let system = r#"你是一个专业翻译助手。请将用户提供的文本翻译成指定的目标语言。
要求：
1. 准确传达原文意思
2. 符合目标语言表达习惯
3. 保持原文的语气和风格
4. 专业术语翻译准确
5. 只输出翻译结果，不要添加解释"#.to_string();

    let user = r#"请将以下文本翻译成{{target_language}}：

{{content}}

翻译结果："#.to_string();

    (system, user)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_builtin_prompts() {
        let (sys, user) = builtin_summary_prompt();
        assert!(!sys.is_empty());
        assert!(!user.is_empty());
        assert!(user.contains("{{target_language}}"));
        assert!(user.contains("{{detail_level}}"));
        assert!(user.contains("{{content}}"));
    }

    #[test]
    fn test_builtin_translation_prompt() {
        let (sys, user) = builtin_translation_prompt();
        assert!(!sys.is_empty());
        assert!(user.contains("{{target_language}}"));
        assert!(user.contains("{{content}}"));
    }

    #[test]
    fn test_replace_placeholders() {
        let template = "Hello {{name}}, your score is {{score}}.".to_string();
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("score".to_string(), "95".to_string());

        let result = PromptManager::replace_placeholders(&template, &vars);
        assert_eq!(result, "Hello Alice, your score is 95.");
    }
}
