#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""使用reportlab生成产品原型PDF"""

from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle
from reportlab.lib.units import cm
from reportlab.lib import colors
from reportlab.platypus import (
    SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle,
    HRFlowable, PageBreak, KeepTogether
)
from reportlab.lib.enums import TA_LEFT, TA_CENTER, TA_JUSTIFY
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
import os

# 注册中文字体
pdfmetrics.registerFont(TTFont('SimHei', 'C:\\Windows\\Fonts\\simhei.ttf'))
pdfmetrics.registerFont(TTFont('SimSun', 'C:\\Windows\\Fonts\\simsun.ttc'))
pdfmetrics.registerFont(TTFont('SimKai', 'C:\\Windows\\Fonts\\simkai.ttf'))
pdfmetrics.registerFont(TTFont('SimFang', 'C:\\Windows\\Fonts\\simfang.ttf'))
pdfmetrics.registerFont(TTFont('NotoSans', 'C:\\Windows\\Fonts\\NotoSansSC-VF.ttf'))

# ── 颜色 ──
C_PRIMARY = colors.HexColor('#667EEA')
C_SECONDARY = colors.HexColor('#764BA2')
C_ACCENT = colors.HexColor('#FF6B6B')
C_GREEN = colors.HexColor('#4CAF50')
C_DARK = colors.HexColor('#2D2D2D')
C_GRAY = colors.HexColor('#666666')
C_LIGHT_GRAY = colors.HexColor('#F8F9FA')
C_BORDER = colors.HexColor('#E0E0E0')
C_CODE_BG = colors.HexColor('#1E1E1E')
C_CODE_TEXT = colors.HexColor('#D4D4D4')

PAGE_W, PAGE_H = A4
MARGIN = 2.2 * cm

doc = SimpleDocTemplate(
    '宠寻寻_产品原型文档_v2.pdf',
    pagesize=A4,
    leftMargin=MARGIN, rightMargin=MARGIN,
    topMargin=2.5*cm, bottomMargin=2.5*cm,
    title='宠寻寻 — 产品原型文档',
    author='宠寻寻团队'
)

# ── 样式 ──
def make_styles():
    s = {}

    s['title'] = ParagraphStyle('title',
        fontName='SimHei', fontSize=22, textColor=C_PRIMARY,
        spaceAfter=6, leading=28, alignment=TA_CENTER, bold=True)

    s['subtitle'] = ParagraphStyle('subtitle',
        fontName='SimSun', fontSize=11, textColor=C_GRAY,
        spaceAfter=4, leading=16, alignment=TA_CENTER)

    s['h1'] = ParagraphStyle('h1',
        fontName='SimHei', fontSize=16, textColor=C_PRIMARY,
        spaceBefore=20, spaceAfter=8, leading=22, bold=True,
        borderPad=(0,0,4,0))

    s['h2'] = ParagraphStyle('h2',
        fontName='SimHei', fontSize=13, textColor=C_SECONDARY,
        spaceBefore=16, spaceAfter=6, leading=18, bold=True)

    s['h3'] = ParagraphStyle('h3',
        fontName='SimHei', fontSize=11, textColor=C_DARK,
        spaceBefore=12, spaceAfter=4, leading=15, bold=True)

    s['body'] = ParagraphStyle('body',
        fontName='SimSun', fontSize=10, textColor=C_DARK,
        spaceBefore=4, spaceAfter=4, leading=17, alignment=TA_JUSTIFY)

    s['bullet'] = ParagraphStyle('bullet',
        fontName='SimSun', fontSize=10, textColor=C_DARK,
        spaceBefore=2, spaceAfter=2, leading=16,
        leftIndent=14, firstLineIndent=-14)

    s['code'] = ParagraphStyle('code',
        fontName='Courier', fontSize=8, textColor=C_CODE_TEXT,
        spaceBefore=4, spaceAfter=4, leading=13,
        backColor=C_CODE_BG, borderPad=8)

    s['quote'] = ParagraphStyle('quote',
        fontName='SimSun', fontSize=10, textColor=C_GRAY,
        spaceBefore=6, spaceAfter=6, leading=16,
        leftIndent=12, borderPad=6, italic=True)

    s['table_header'] = ParagraphStyle('table_header',
        fontName='SimHei', fontSize=9, textColor=colors.white,
        alignment=TA_CENTER)

    s['table_cell'] = ParagraphStyle('table_cell',
        fontName='SimSun', fontSize=9, textColor=C_DARK,
        leading=13, alignment=TA_LEFT)

    s['table_center'] = ParagraphStyle('table_center',
        fontName='SimSun', fontSize=9, textColor=C_DARK,
        leading=13, alignment=TA_CENTER)

    return s

ST = make_styles()

story = []

# ══════════════════════════════════════
# 封面
# ══════════════════════════════════════
story.append(Spacer(1, 3*cm))
story.append(Paragraph('宠寻寻', ST['title']))
story.append(Paragraph('产品原型文档', ParagraphStyle('cover_sub',
    fontName='SimHei', fontSize=28, textColor=C_SECONDARY,
    alignment=TA_CENTER, spaceAfter=16)))
story.append(HRFlowable(width='60%', thickness=3, color=C_PRIMARY,
    spaceAfter=20, hAlign='CENTER'))
story.append(Paragraph('基于AI图像识别的智能寻宠平台', ST['subtitle']))
story.append(Spacer(1, 0.5*cm))
story.append(Paragraph('版本：v1.0  |  更新日期：2026年6月19日', ST['subtitle']))
story.append(Paragraph('团队成员：产品经理 · 技术开发 · UI设计', ST['subtitle']))
story.append(PageBreak())

# ══════════════════════════════════════
# 通用函数
# ══════════════════════════════════════
def h1(text):
    story.append(HRFlowable(width='100%', thickness=2, color=C_PRIMARY,
        spaceBefore=8, spaceAfter=2))
    story.append(Paragraph(text, ST['h1']))

def h2(text):
    story.append(Paragraph(text, ST['h2']))

def h3(text):
    story.append(Paragraph(text, ST['h3']))

def body(text):
    story.append(Paragraph(text, ST['body']))

def bullet(text, bold=False):
    style = ParagraphStyle('b', parent=ST['bullet'])
    if bold:
        style.textColor = C_PRIMARY
        style.fontName = 'SimHei'
    story.append(Paragraph(f'• {text}', style))

def sp(h=0.3):
    story.append(Spacer(1, h*cm))

def br():
    story.append(HRFlowable(width='100%', thickness=1, color=C_BORDER,
        spaceBefore=8, spaceAfter=8))

def make_table(headers, rows, col_widths=None):
    """创建带样式的表格"""
    from reportlab.platypus import Paragraph as P
    from reportlab.lib.styles import ParagraphStyle

    def ph(text, center=False):
        s = ParagraphStyle('th', fontName='SimHei', fontSize=9,
            textColor=colors.white, leading=13, alignment=TA_CENTER if center else TA_LEFT)
        return P(text, s)

    def pc(text, center=False):
        s = ParagraphStyle('td', fontName='SimSun', fontSize=9,
            textColor=C_DARK, leading=13, alignment=TA_CENTER if center else TA_LEFT)
        return P(str(text), s)

    header_row = [ph(h) for h in headers]
    data = [header_row]
    for row in rows:
        data.append([pc(str(c), i > 0) for i, c in enumerate(row)])

    style = [
        ('BACKGROUND', (0,0), (-1,0), C_PRIMARY),
        ('TEXTCOLOR', (0,0), (-1,0), colors.white),
        ('FONTNAME', (0,0), (-1,0), 'SimHei'),
        ('FONTSIZE', (0,0), (-1,0), 9),
        ('ALIGN', (0,0), (-1,-1), 'CENTER'),
        ('VALIGN', (0,0), (-1,-1), 'MIDDLE'),
        ('GRID', (0,0), (-1,-1), 0.5, C_BORDER),
        ('ROWBACKGROUNDS', (0,1), (-1,-1), [colors.white, C_LIGHT_GRAY]),
        ('TOPPADDING', (0,0), (-1,-1), 6),
        ('BOTTOMPADDING', (0,0), (-1,-1), 6),
        ('LEFTPADDING', (0,0), (-1,-1), 8),
        ('RIGHTPADDING', (0,0), (-1,-1), 8),
    ]

    t = Table(data, colWidths=col_widths)
    t.setStyle(TableStyle(style))
    return t

def code_block(text):
    """代码块"""
    from reportlab.platypus import Paragraph as P
    s = ParagraphStyle('code', fontName='Courier', fontSize=8,
        textColor=C_CODE_TEXT, leading=13, backColor=C_CODE_BG,
        borderPad=10, leftIndent=0)
    for line in text.strip().split('\n'):
        story.append(P(line, s))
    sp(0.2)

# ══════════════════════════════════════
# 一、产品概述
# ══════════════════════════════════════
h1('一、产品概述')

h2('1.1 产品定位')
body('宠寻寻是一款面向宠物主人的AI图像识别寻宠平台，通过"AI智能匹配 + 社区互助"模式，帮助宠物主人快速找回走失宠物。')
sp(0.3)

h2('1.2 核心价值')
story.append(make_table(
    ['维度', '描述'],
    [
        ['目标用户', '养宠家庭（宠物主人）、爱心人士、宠物机构'],
        ['核心功能', '走失发布、线索上报、AI图像匹配、社区互助'],
        ['关键差异', 'AI双重图像识别、实时推送、悬赏机制'],
        ['用户价值', '提升寻宠效率，从传统20%提升至60%+'],
    ],
    col_widths=[3.5*cm, 13*cm]
))

sp(0.3)
h2('1.3 业务流程总图')
body('核心业务流程如下：')
sp(0.2)
code_block('''┌──────────────────────────────────────────────────────┐
│              宠寻寻 — 核心业务闭环流程                    │
└──────────────────────────────────────────────────────┘

【宠物主人侧】                              【爱心人士侧】
     │                                           │
     ▼                                           │
┌─────────────┐                                  │
│  发现宠物走失 │                                 │
└──────┬──────┘                                  │
       │                                          │
       ▼                                          │
┌─────────────┐   推送通知   ┌─────────────┐     │
│ 发布走失信息 │ ──────────→ │ 附近用户收到 │     │
└──────┬──────┘             │ 走失推送     │     │
       │                    └──────┬──────┘     │
       │                           │            │
       │                           ▼            │
       │                    ┌─────────────┐      │
       │                    │ 发现疑似走失 │      │
       │                    │ 宠物         │      │
       │                    └──────┬──────┘      │
       │                           │            │
       │                           ▼            │
       │                    ┌─────────────┐      │
       │                    │ 上报发现线索 │ ─────┘
       │                    └──────┬──────┘
       │                           │
       │                           ▼
       │                    ┌─────────────┐
       │                    │ AI图像匹配   │
       │                    │ 自动比对     │
       │                    └──────┬──────┘
       │                           │
       │         ┌─────────────────┼─────────────────┐
       │         │                 │                 │
       │         ▼                 │                 │
       │  ┌─────────────┐          │                 │
       │  │ 匹配失败     │          │                 │
       │  │ 继续寻找     │          │                 │
       │  └─────────────┘          │                 │
       │                           ▼                 │
       │                    ┌─────────────┐          │
       │                    │ 匹配成功     │──────────┘
       │                    │ 通知发布者   │
       │                    └──────┬──────┘
       │                           │
       ▼                           │
┌─────────────┐                   │
│ 收到匹配通知  │◄──────────────────┘
│ 确认匹配     │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ 联系发现者   │
│ 确认找回     │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ 宠物安全回家 │
│ 悬赏金结算  │
└─────────────┘

图例：【宠物主人】 ← ─ ─ → 【爱心人士/平台】
      关键节点：发布 → 推送 → 发现 → 上报 → AI匹配 → 通知 → 确认 → 回家''')
sp(0.2)

# ══════════════════════════════════════
# 二、页面原型与交互说明
# ══════════════════════════════════════
h1('二、页面原型与交互说明')

h2('2.1 页面结构总览')
body('产品共包含以下核心页面：')
sp(0.2)
story.append(make_table(
    ['页面名称', '功能描述', '优先级'],
    [
        ['注册/登录页', '手机号注册、密码登录、验证码登录', 'P0'],
        ['首页（走失信息流）', '展示附近走失宠物列表，支持地图/列表双视图切换', 'P0'],
        ['发布走失信息', '填写宠物走失信息，上传照片，设置悬赏', 'P0'],
        ['走失详情页', '展示走失宠物详细信息，提供线索上报入口', 'P0'],
        ['线索上报', '爱心人士发现疑似走失宠物时上报线索', 'P0'],
        ['AI匹配结果页', '展示AI图像匹配结果，按相似度排序', 'P0'],
        ['AI智能助手', '集成AI大模型的智能对话助手，辅助寻宠', 'P1'],
        ['个人中心', '用户个人信息管理与宠物档案管理', 'P0'],
    ],
    col_widths=[4*cm, 8*cm, 1.5*cm]
))
sp(0.4)

h2('2.2 页面详细设计')

h3('页面1：注册/登录页')
story.append(make_table(
    ['元素', '类型', '说明'],
    [
        ['Logo', '图片', '宠寻寻品牌Logo'],
        ['手机号输入框', '输入', '11位手机号，格式校验'],
        ['密码输入框', '输入', '6-20位密码登录'],
        ['验证码登录切换', '切换按钮', '切换至短信验证码登录模式'],
        ['注册/登录按钮', '按钮', '提交表单，自动识别注册/登录状态'],
        ['协议勾选', '复选框', '用户协议与隐私政策勾选确认'],
    ],
    col_widths=[3.5*cm, 2.5*cm, 11*cm]
))
sp(0.2)
body('交互流程：用户输入手机号 -> 点击获取验证码/输入密码 -> 系统校验格式 -> 验证成功自动跳转首页 -> 失败则提示错误信息。')
sp(0.4)

h3('页面2：首页（走失信息流）')
story.append(make_table(
    ['元素', '类型', '说明'],
    [
        ['顶部搜索栏', '搜索框', '按品种、地点、特征搜索走失信息'],
        ['视图切换', '切换按钮', '列表视图 / 地图视图 切换'],
        ['走失卡片', '卡片列表', '宠物照片、名称、走失时间、距离、悬赏金额'],
        ['悬浮发布按钮', 'FAB按钮', '快速发布走失信息（右下角悬浮）'],
        ['底部导航', 'Tab栏', '首页 / AI助手 / 消息 / 我的'],
    ],
    col_widths=[3.5*cm, 2.5*cm, 11*cm]
))
sp(0.2)
body('交互流程：默认加载附近走失信息列表（按时间倒序） -> 上滑加载更多（分页，每页20条） -> 点击卡片进入详情页 -> 点击地图切换显示标注 -> 下拉刷新重新获取。')
sp(0.4)

h3('页面3：发布走失信息')
story.append(make_table(
    ['元素', '类型', '说明'],
    [
        ['宠物选择', '下拉选择', '从已有宠物档案中选择，或新建宠物'],
        ['照片上传', '多图上传', '最多6张，支持相机/相册选择'],
        ['走失时间', '日期时间选择器', '默认当前时间，可手动调整'],
        ['走失地点', '地图选择/文本输入', '支持地图标记和手动输入文字'],
        ['详细描述', '多行文本', '宠物特征、走失时情况、习惯、注意事项'],
        ['悬赏金额', '数字输入框', '可选，设置悬赏可吸引更多关注'],
        ['紧急程度', '开关', '加急推送开关（付费增值服务）'],
        ['发布按钮', '按钮', '确认发布，提交前需完整填写必填项'],
    ],
    col_widths=[3.5*cm, 2.5*cm, 11*cm]
))
sp(0.2)
body('交互流程：选择或新建宠物档案 -> 上传走失照片（至少1张） -> 标记走失地点 -> 填写走失时间、描述 -> 可选设置悬赏 -> 可选开启加急 -> 点击发布 -> 确认弹窗 -> 提交成功 -> 自动推送至附近用户。')
sp(0.4)

h3('页面4：走失详情页')
story.append(make_table(
    ['元素', '类型', '说明'],
    [
        ['宠物照片', '轮播图', '多角度展示走失宠物照片，支持左右滑动'],
        ['基本信息', '文本', '名称、品种、毛色、年龄、体重'],
        ['走失信息', '文本', '走失时间、走失地点、详细描述'],
        ['悬赏金额', '高亮标签', '醒目的悬赏金额展示，吸引关注'],
        ['线索上报按钮', '醒目按钮', '"我有线索！"全宽按钮，鼓励上报'],
        ['评论区', '留言列表', '社区互动，其他用户提供线索或鼓励'],
        ['分享按钮', '按钮', '分享至微信/朋友圈，扩大传播'],
    ],
    col_widths=[3.5*cm, 2.5*cm, 11*cm]
))
sp(0.4)

h3('页面5：线索上报')
story.append(make_table(
    ['元素', '类型', '说明'],
    [
        ['关联走失信息', '自动关联', '从详情页进入时自动关联对应走失信息'],
        ['发现照片', '单图/多图', '拍摄或上传发现的宠物照片'],
        ['发现地点', '地图选择', '当前定位自动获取，支持手动调整'],
        ['发现时间', '时间选择器', '默认当前时间'],
        ['情况描述', '多行文本', '宠物状态、周围环境、与走失信息的差异'],
        ['联系方式', '自动填充', '默认使用注册手机号，可修改'],
        ['提交按钮', '按钮', '确认上报，提交后自动触发AI匹配'],
    ],
    col_widths=[3.5*cm, 2.5*cm, 11*cm]
))
sp(0.4)

h3('页面6：AI匹配结果页')
story.append(make_table(
    ['元素', '类型', '说明'],
    [
        ['匹配进度', '动画进度条', 'AI分析中动画，增强感知'],
        ['匹配结果列表', '卡片列表', '按相似度降序排列，高匹配优先展示'],
        ['相似度标签', '彩色标识', '高(>80%绿)/中(60-80%黄)/低(<60%灰)三级'],
        ['特征对比', '对照视图', '发现照片与走失照片左右对比展示'],
        ['匹配详情', '展开列表', '各维度匹配得分：品种、毛色、体型、特殊标记'],
        ['联系发布者', '按钮', '确认匹配后展示联系方式，支持一键拨号/发消息'],
    ],
    col_widths=[3.5*cm, 2.5*cm, 11*cm]
))
sp(0.2)
body('匹配结果卡片内容：相似度百分比 -> 高/中/低匹配标识 -> 发现照片与走失照片对照 -> 品种/毛色/体型/特征各维度评分 -> 确认匹配按钮 / 查看更多按钮。')
sp(0.4)

h3('页面7：AI智能助手')
story.append(make_table(
    ['元素', '类型', '说明'],
    [
        ['对话列表', '消息列表', '用户与AI的对话记录，支持多轮对话'],
        ['快捷操作', '按钮组', '"帮我找宠物"、"养宠建议"、"常见问题"等快捷入口'],
        ['输入框', '文本输入', '用户自由输入问题或描述'],
        ['发送按钮', '按钮', '发送消息触发AI回复'],
        ['宠物搜索结果', '卡片列表', 'AI自动搜索数据库后展示匹配的宠物卡片'],
    ],
    col_widths=[3.5*cm, 2.5*cm, 11*cm]
))
sp(0.4)

h3('页面8：个人中心')
story.append(make_table(
    ['区域', '功能', '说明'],
    [
        ['用户信息', '头像、昵称、手机号', '展示与编辑用户个人信息'],
        ['我的宠物', '宠物列表', '管理宠物档案，支持添加/编辑/删除'],
        ['发布记录', '走失记录列表', '查看/编辑已发布的走失信息，支持追加悬赏'],
        ['线索管理', '上报线索列表', '查看自己上报的线索及反馈状态'],
        ['账户设置', '密码/通知/隐私', '修改密码、通知偏好、隐私设置'],
        ['统计数据', '发布数、找回数', '展示寻宠成就，增强用户粘性'],
    ],
    col_widths=[2.5*cm, 4*cm, 10.5*cm]
))
sp(0.4)

# ══════════════════════════════════════
# 三、核心交互流程
# ══════════════════════════════════════
h1('三、核心交互流程详解')

h2('流程1：宠物走失完整闭环（发布 -> 匹配 -> 找回）')
code_block('''Step 1: 宠物走失
  -> 用户发现宠物走失

Step 2: 拍照发布
  -> 选择宠物档案
  -> 上传走失照片
  -> 填写走失时间/地点/描述
  -> 设置悬赏金额（可选）
  -> 提交发布

Step 3: AI推送
  -> 系统根据位置智能推送
  -> 消息推送给附近用户
  -> 爱心人士收到推送通知

Step 4: 爱心人士上报线索
  -> 发现疑似走失宠物
  -> 拍照并上报线索
  -> 填写发现地点/时间/描述

Step 5: AI图像匹配
  -> 自动提取特征
  -> 与数据库走失宠物比对
  -> 按相似度排序返回结果
  -> 匹配成功则通知发布者

Step 6: 确认找回
  -> 发布者查看匹配结果
  -> 确认匹配并联系发现者
  -> 双方沟通确认
  -> 宠物安全回家''')
sp(0.4)

h2('流程2：悬赏与加急推广流程')
code_block('''Step 1: 发布走失信息（基础发布）
  -> 免费推送给周边5km用户

Step 2: 设置悬赏
  -> 用户设置悬赏金额
  -> 赏金由平台托管
  -> 悬赏信息获得更高曝光

Step 3: 加急推广（付费增值）
  -> 用户选择加急推广
  -> 支付推广费用
  -> 推送范围扩大至10-20km
  -> 推送频率加倍

Step 4: 寻回结算
  -> 宠物成功寻回
  -> 平台收取10%服务费
  -> 剩余悬赏金支付给发现者
  -> 平台获得悬赏分成收入''')
sp(0.4)

h2('流程3：AI智能对话寻宠流程')
code_block('''Step 1: 用户输入描述
  -> 用户通过文字描述看到的宠物
  -> 或选择快捷操作（帮我找猫/狗）

Step 2: AI理解意图
  -> 提取关键特征：品种、毛色、地点
  -> 解析搜索条件
  -> 转化为数据库查询条件

Step 3: 数据库搜索匹配
  -> 关键词匹配走失宠物
  -> 相似度计算排序
  -> 返回top结果

Step 4: 返回搜索结果
  -> 展示匹配的宠物卡片列表
  -> 显示相似度评分
  -> 可点击查看详情

Step 5: 继续多轮对话
  -> 用户补充更多特征
  -> AI进一步精确搜索
  -> 直到找到目标''')
sp(0.4)

# ══════════════════════════════════════
# 四、数据模型
# ══════════════════════════════════════
h1('四、数据模型')

h2('4.1 用户表（users）')
story.append(make_table(
    ['字段名', '数据类型', '约束', '说明'],
    [
        ['id', 'INTEGER', 'PK, AUTO', '用户唯一标识'],
        ['phone', 'TEXT', 'UNIQUE, NOT NULL', '手机号（登录账号）'],
        ['password', 'TEXT', 'NOT NULL', '加密后的密码'],
        ['name', 'TEXT', '-', '用户昵称'],
        ['emergency_contact', 'TEXT', '-', '紧急联系人'],
        ['address', 'TEXT', '-', '用户地址'],
        ['created_at', 'TEXT', '-', '注册时间'],
    ],
    col_widths=[3.5*cm, 2.5*cm, 2.5*cm, 8*cm]
))
sp(0.3)

h2('4.2 宠物表（pets）')
story.append(make_table(
    ['字段名', '数据类型', '约束', '说明'],
    [
        ['id', 'INTEGER', 'PK, AUTO', '宠物唯一标识'],
        ['user_id', 'INTEGER', 'FK -> users.id', '所属用户ID'],
        ['name', 'TEXT', 'NOT NULL', '宠物名称'],
        ['species', 'TEXT', '-', '物种（cat/dog/other）'],
        ['breed', 'TEXT', '-', '品种（如：金毛、橘猫）'],
        ['color', 'TEXT', '-', '毛色'],
        ['age', 'INTEGER', '-', '年龄（岁）'],
        ['weight', 'REAL', '-', '体重（kg）'],
        ['features', 'TEXT', '-', '特殊特征描述'],
        ['photo', 'BLOB', '-', '宠物照片二进制数据'],
        ['created_at', 'TEXT', '-', '创建时间'],
    ],
    col_widths=[3.5*cm, 2.5*cm, 2.5*cm, 8*cm]
))
sp(0.3)

h2('4.3 走失信息表（lost_pets）')
story.append(make_table(
    ['字段名', '数据类型', '约束', '说明'],
    [
        ['id', 'INTEGER', 'PK, AUTO', '走失信息唯一标识'],
        ['pet_id', 'INTEGER', 'FK -> pets.id', '关联宠物ID'],
        ['lost_time', 'TEXT', 'NOT NULL', '走失时间'],
        ['lost_location', 'TEXT', 'NOT NULL', '走失地点'],
        ['description', 'TEXT', '-', '走失详细描述'],
        ['reward', 'REAL', '-', '悬赏金额（元）'],
        ['status', 'TEXT', "DEFAULT 'lost'", '状态（lost/found/closed）'],
        ['urgent', 'INTEGER', 'DEFAULT 0', '是否加急（0否/1是）'],
        ['total_push_count', 'INTEGER', 'DEFAULT 0', '已推送人数'],
        ['created_at', 'TEXT', '-', '发布时间'],
    ],
    col_widths=[3.5*cm, 2.5*cm, 2.5*cm, 8*cm]
))
sp(0.3)

h2('4.4 发现线索表（found_pets）')
story.append(make_table(
    ['字段名', '数据类型', '约束', '说明'],
    [
        ['id', 'INTEGER', 'PK, AUTO', '线索唯一标识'],
        ['finder_id', 'INTEGER', 'FK -> users.id', '发现者用户ID'],
        ['photo', 'BLOB', '-', '发现的宠物照片'],
        ['found_time', 'TEXT', 'NOT NULL', '发现时间'],
        ['found_location', 'TEXT', 'NOT NULL', '发现地点'],
        ['description', 'TEXT', '-', '发现情况描述'],
        ['status', 'TEXT', "DEFAULT 'pending'", '状态（pending/matched）'],
        ['matched_lost_id', 'INTEGER', '-', '匹配到的走失信息ID'],
        ['created_at', 'TEXT', '-', '上报时间'],
    ],
    col_widths=[3.5*cm, 2.5*cm, 2.5*cm, 8*cm]
))
sp(0.3)

# ══════════════════════════════════════
# 五、技术方案
# ══════════════════════════════════════
h1('五、技术实现方案')

h2('5.1 技术栈选型')
story.append(make_table(
    ['层级', '技术选型', '选择理由'],
    [
        ['前端框架', 'Vue.js 2.x', '轻量级、组件化、生态完善、学习成本低'],
        ['页面样式', 'HTML5 + CSS3', '标准Web技术，兼容性好'],
        ['后端框架', 'Python Flask', '轻量级、RESTful API开发高效、扩展丰富'],
        ['数据库', 'SQLite', '文件型数据库、无需安装、适合中小型项目'],
        ['跨域支持', 'Flask-CORS', '解决前后端分离部署的跨域问题'],
        ['AI识别-1', '百度AI动物识别', '免费额度充足、识别准确率>90%、接口简单'],
        ['AI识别-2', 'GPT-4 Vision / 通义千问', '深度特征分析、毛色/体型/特殊标记提取'],
        ['地图服务', '微信小程序地图组件', '自带LBS能力，无需额外配置'],
    ],
    col_widths=[3*cm, 5*cm, 9*cm]
))
sp(0.4)

h2('5.2 系统架构')
code_block('''┌───────────────────────────────────────┐
│           用户端（Web浏览器）           │
│   HTML5 + CSS3 + Vue.js 响应式设计     │
└──────────────────┬────────────────────┘
                   │ HTTP/HTTPS
                   ▼
┌───────────────────────────────────────┐
│        API Gateway（Flask）            │
│   RESTful API + Flask-CORS            │
└──────────────────┬────────────────────┘
                   │
         ┌─────────┼─────────┐
         ▼         ▼         ▼
┌──────────┐ ┌──────────┐ ┌──────────┐
│ 业务服务层 │ │ AI服务层  │ │ 数据服务层 │
│ 用户/宠物 │ │ 百度AI   │ │ SQLite   │
│ 走失/线索 │ │ 大模型   │ │ 数据库   │
└──────────┘ └──────────┘ └──────────┘''')
sp(0.4)

h2('5.3 AI匹配算法')
body('AI相似度综合评分公式如下：')
sp(0.2)
code_block('''综合相似度 = 基础图像相似度 + AI加成分

加成规则：
  • 百度AI物种识别匹配：+0.2 × 置信度（0~1之间）
  • 大模型品种匹配：+0.15 × 置信度
  • 毛色匹配：+0.1（匹配）+ 0（不匹配）
  • 特殊特征匹配：+0.05 × 匹配特征数量

最终得分归一化为0-100%的相似度百分比
按相似度从高到低排序返回top-N结果''')
sp(0.4)

# ══════════════════════════════════════
# 六、版本规划
# ══════════════════════════════════════
h1('六、产品版本规划')

story.append(make_table(
    ['版本', '功能范围', '计划时间', '状态'],
    [
        ['v1.0 MVP', '用户注册、宠物档案、走失发布、线索上报、AI图像匹配', '第1-2个月', '已完成'],
        ['v1.1', '地图视图、悬赏托管、加急推送、消息通知', '第3-4个月', '开发中'],
        ['v1.2', '社区论坛、积分系统、宠物知识库、宠物保险', '第5-6个月', '规划中'],
        ['v2.0', '微信小程序、LBS附近推送、语音搜索、AR寻宠', '第7-12个月', '远期规划'],
    ],
    col_widths=[2.5*cm, 8*cm, 3*cm, 2*cm]
))

sp(0.4)
body('注：v1.0为最小可行产品（MVP），聚焦核心寻宠闭环；后续版本逐步扩展功能边界与用户体验优化。')

# ── 页脚 ──

def add_page_number(canvas, doc):
    canvas.saveState()
    canvas.setFont('SimSun', 9)
    canvas.setFillColor(colors.HexColor('#999999'))
    page_num = canvas.getPageNumber()
    text = f'- {page_num} -'
    canvas.drawCentredString(A4[0] / 2.0, 1.5*cm, text)
    canvas.restoreState()

# ── 生成 ──
doc.build(story, onFirstPage=add_page_number, onLaterPages=add_page_number)
print('PDF generated: 宠寻寻_产品原型文档_v2.pdf')
