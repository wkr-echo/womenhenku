#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""将产品原型文档MD转换为PDF"""

import markdown
from weasyprint import HTML, CSS
import os

md_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), '宠寻寻_产品原型文档.md')
pdf_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), '宠寻寻_产品原型文档.pdf')

# 读取MD文件
with open(md_path, 'r', encoding='utf-8') as f:
    md_content = f.read()

# Markdown扩展
md = markdown.Markdown(
    extensions=[
        'tables',
        'fenced_code',
        'codehilite',
        'toc',
        'nl2br',
        'sane_lists',
    ]
)

# 转换为HTML
html_body = md.convert(md_content)

# HTML模板（带样式）
html_template = f'''<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>宠寻寻 — 产品原型文档</title>
<style>
    @page {{
        size: A4;
        margin: 2.5cm 2cm 2.5cm 2cm;
        @top-center {{
            content: "宠寻寻 — 产品原型文档";
            font-size: 9pt;
            color: #999;
        }}
        @bottom-center {{
            content: counter(page);
            font-size: 9pt;
            color: #999;
        }}
    }}

    body {{
        font-family: "Microsoft YaHei", "SimHei", "WenQuanYi Micro Hei", "Noto Sans CJK SC", sans-serif;
        font-size: 10.5pt;
        line-height: 1.8;
        color: #2D2D2D;
        max-width: 100%;
    }}

    h1 {{
        font-size: 22pt;
        color: #667EEA;
        border-bottom: 3px solid #667EEA;
        padding-bottom: 8px;
        page-break-after: avoid;
        margin-top: 0;
    }}

    h2 {{
        font-size: 16pt;
        color: #764BA2;
        border-left: 4px solid #764BA2;
        padding-left: 10px;
        page-break-after: avoid;
        margin-top: 28px;
    }}

    h3 {{
        font-size: 13pt;
        color: #2D2D2D;
        border-bottom: 1px solid #E0E0E0;
        padding-bottom: 4px;
        page-break-after: avoid;
        margin-top: 20px;
    }}

    h4 {{
        font-size: 11pt;
        color: #555;
        page-break-after: avoid;
        margin-top: 14px;
    }}

    p {{
        margin: 8px 0;
        text-align: justify;
    }}

    strong {{
        color: #667EEA;
    }}

    em {{
        color: #764BA2;
    }}

    code {{
        font-family: "Consolas", "Courier New", monospace;
        background: #F5F5F5;
        padding: 2px 6px;
        border-radius: 4px;
        font-size: 9pt;
        color: #D63384;
    }}

    pre {{
        background: #1E1E1E;
        color: #D4D4D4;
        padding: 14px;
        border-radius: 8px;
        overflow-x: auto;
        font-size: 8.5pt;
        line-height: 1.6;
        page-break-inside: avoid;
    }}

    pre code {{
        background: transparent;
        padding: 0;
        color: #D4D4D4;
    }}

    blockquote {{
        border-left: 4px solid #667EEA;
        background: #EEF1FF;
        margin: 14px 0;
        padding: 10px 16px;
        color: #444;
        border-radius: 0 6px 6px 0;
    }}

    blockquote p {{
        margin: 0;
    }}

    table {{
        width: 100%;
        border-collapse: collapse;
        margin: 14px 0;
        font-size: 9.5pt;
        page-break-inside: avoid;
    }}

    thead th {{
        background: #667EEA;
        color: white;
        padding: 8px 10px;
        text-align: left;
        font-weight: bold;
    }}

    tbody tr:nth-child(even) {{
        background: #F8F9FF;
    }}

    tbody tr:nth-child(odd) {{
        background: #FFFFFF;
    }}

    td {{
        padding: 7px 10px;
        border: 1px solid #E0E0E0;
        vertical-align: top;
    }}

    ul, ol {{
        margin: 8px 0;
        padding-left: 24px;
    }}

    li {{
        margin: 4px 0;
    }}

    li ul, li ol {{
        margin: 2px 0;
    }}

    hr {{
        border: none;
        border-top: 2px solid #E0E0E0;
        margin: 20px 0;
    }}

    a {{
        color: #667EEA;
        text-decoration: none;
    }}

    /* ASCII art / code block preservation */
    pre:has(code) {{
        white-space: pre;
        overflow-x: visible;
    }}
</style>
</head>
<body>
{html_body}
</body>
</html>'''

# 生成PDF
HTML(string=html_template).write_pdf(pdf_path)
print(f'PDF generated: {pdf_path}')
