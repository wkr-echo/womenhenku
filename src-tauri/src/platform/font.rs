/// Returns a curated list of font-family stacks for reading.
/// Each entry includes fallback fonts so Chinese text renders correctly.
pub fn list_fonts() -> Vec<String> {
    let mut fonts = vec![
        // System default
        "system-ui, sans-serif".to_string(),
        // Serif
        "Georgia, Noto Serif SC, serif".to_string(),
    ];

    #[cfg(target_os = "windows")]
    {
        fonts.extend(vec![
            "Microsoft YaHei, sans-serif".into(),
            "Segoe UI, Microsoft YaHei, sans-serif".into(),
            "SimSun, serif".into(),
            "KaiTi, serif".into(),
        ]);
    }

    #[cfg(target_os = "macos")]
    {
        fonts.extend(vec![
            "PingFang SC, sans-serif".into(),
            "Hiragino Sans GB, sans-serif".into(),
            "SF Pro, PingFang SC, sans-serif".into(),
        ]);
    }

    #[cfg(target_os = "linux")]
    {
        fonts.extend(vec![
            "Noto Sans SC, sans-serif".into(),
            "Noto Serif SC, serif".into(),
            "WenQuanYi Micro Hei, sans-serif".into(),
            "WenQuanYi Zen Hei, sans-serif".into(),
        ]);
    }

    // Cross-platform: popular reading fonts with Chinese fallback
    fonts.extend(vec![
        "Inter, Noto Sans SC, sans-serif".into(),
        "Source Han Sans SC, sans-serif".into(),
        "LXGW WenKai, serif".into(),
        "Noto Serif SC, serif".into(),
    ]);

    fonts
}
