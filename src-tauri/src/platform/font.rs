/// Returns a curated list of common system fonts for the current platform.
/// Falls back to a default cross-platform font list if detection fails.
pub fn list_fonts() -> Vec<String> {
    let mut fonts = vec![
        // Always-available generic families
        "system-ui".to_string(),
        "sans-serif".to_string(),
        "serif".to_string(),
        "monospace".to_string(),
    ];

    #[cfg(target_os = "windows")]
    {
        fonts.extend(vec![
            "Segoe UI".into(),
            "Microsoft YaHei".into(),
            "SimSun".into(),
            "KaiTi".into(),
            "Consolas".into(),
            "Courier New".into(),
        ]);
    }

    #[cfg(target_os = "macos")]
    {
        fonts.extend(vec![
            "SF Pro".into(),
            "Helvetica Neue".into(),
            "PingFang SC".into(),
            "Hiragino Sans GB".into(),
            "Menlo".into(),
            "Monaco".into(),
        ]);
    }

    #[cfg(target_os = "linux")]
    {
        fonts.extend(vec![
            "Noto Sans".into(),
            "Noto Sans SC".into(),
            "DejaVu Sans".into(),
            "DejaVu Serif".into(),
            "WenQuanYi Micro Hei".into(),
            "Ubuntu".into(),
        ]);
    }

    // Cross-platform developer/reading fonts
    fonts.extend(vec![
        "Inter".into(),
        "JetBrains Mono".into(),
        "Fira Code".into(),
        "Cascadia Code".into(),
        "Source Han Sans SC".into(),
        "LXGW WenKai".into(),
        "Noto Serif SC".into(),
    ]);

    fonts
}
