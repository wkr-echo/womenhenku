<#
.SYNOPSIS
安装 Womenhenku (Mercury 跨平台复刻) 的 Windows 开发环境

.DESCRIPTION
此脚本会自动安装以下依赖：
1. Rust 1.80+ (stable)
2. Node.js 20 LTS
3. Tauri CLI 2.x
4. 前端 npm 依赖

使用方法：
powershell -ExecutionPolicy Bypass -File scripts/install-windows.ps1

需要以管理员身份运行。
#>

$ErrorActionPreference = "Stop"
$script:ProgressPreference = "SilentlyContinue"

function Test-CommandExists {
    param($command)
    $exists = $null -ne (Get-Command $command -ErrorAction SilentlyContinue)
    return $exists
}

function Write-Status {
    param([string]$message)
    Write-Host "`n=== $message ===" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$message)
    Write-Host "[OK] $message" -ForegroundColor Green
}

function Write-Error {
    param([string]$message)
    Write-Host "[ERROR] $message" -ForegroundColor Red
}

function Write-Warning {
    param([string]$message)
    Write-Host "[WARN] $message" -ForegroundColor Yellow
}

# ============================================================
# 检查管理员权限
# ============================================================
Write-Status "检查管理员权限"
$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
$isAdmin = $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Write-Error "需要以管理员身份运行此脚本"
    Write-Warning "请右键点击 PowerShell -> 以管理员身份运行"
    exit 1
}
Write-Success "已获取管理员权限"

# ============================================================
# 检查并安装 Rust
# ============================================================
Write-Status "检查 Rust 环境"

if (Test-CommandExists "cargo") {
    $rustVersion = cargo --version 2>&1
    Write-Host "已安装: $rustVersion"
    
    if ($rustVersion -match "1\.(8[0-9]|[9-9][0-9])") {
        Write-Success "Rust 版本符合要求 (1.80+)"
    } else {
        Write-Warning "Rust 版本较旧，将更新..."
        rustup update stable
        Write-Success "Rust 更新完成"
    }
} else {
    Write-Host "正在安装 Rust..."
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
    Start-Process -FilePath "$env:TEMP\rustup-init.exe" -ArgumentList "-y" -Wait -NoNewWindow
    
    if (-not $?) {
        Write-Error "Rust 安装失败"
        exit 1
    }
    Write-Success "Rust 安装完成"
    
    # 添加 Rust 到 PATH
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
    [Environment]::SetEnvironmentVariable("PATH", $env:PATH, "User")
}

# ============================================================
# 检查并安装 Node.js 20 LTS
# ============================================================
Write-Status "检查 Node.js 环境"

if (Test-CommandExists "node") {
    $nodeVersion = node --version 2>&1
    Write-Host "已安装: $nodeVersion"
    
    if ($nodeVersion -match "v20\.") {
        Write-Success "Node.js 版本符合要求 (20.x)"
    } else {
        Write-Warning "Node.js 版本不符合，将安装 20 LTS..."
        $installNode = $true
    }
} else {
    $installNode = $true
}

if ($installNode) {
    Write-Host "正在下载 Node.js 20 LTS..."
    $nodeUrl = "https://nodejs.org/dist/v20.15.1/node-v20.15.1-x64.msi"
    Invoke-WebRequest -Uri $nodeUrl -OutFile "$env:TEMP\node-installer.msi"
    
    Write-Host "正在安装 Node.js..."
    Start-Process -FilePath "msiexec.exe" -ArgumentList "/i `"$env:TEMP\node-installer.msi`" /qn /norestart" -Wait -NoNewWindow
    
    if (-not $?) {
        Write-Error "Node.js 安装失败"
        exit 1
    }
    Write-Success "Node.js 20 LTS 安装完成"
    
    # 刷新 PATH
    $env:PATH = "$env:ProgramFiles\nodejs;$env:USERPROFILE\AppData\Roaming\npm;$env:PATH"
    [Environment]::SetEnvironmentVariable("PATH", $env:PATH, "Machine")
}

# ============================================================
# 安装 Tauri CLI
# ============================================================
Write-Status "安装 Tauri CLI"

if (Test-CommandExists "tauri") {
    $tauriVersion = tauri --version 2>&1
    Write-Host "已安装: $tauriVersion"
    
    if ($tauriVersion -match "2\.\d+") {
        Write-Success "Tauri CLI 版本符合要求 (2.x)"
    } else {
        Write-Warning "Tauri CLI 版本较旧，将更新..."
        cargo install tauri-cli --version "^2"
    }
} else {
    Write-Host "正在安装 Tauri CLI..."
    cargo install tauri-cli --version "^2"
    
    if (-not $?) {
        Write-Error "Tauri CLI 安装失败"
        exit 1
    }
    Write-Success "Tauri CLI 安装完成"
}

# ============================================================
# 安装前端依赖
# ============================================================
Write-Status "安装前端 npm 依赖"

if (-not (Test-Path "src-ui/node_modules")) {
    Write-Host "正在安装 npm 依赖..."
    Set-Location src-ui
    npm install
    
    if (-not $?) {
        Write-Error "npm 依赖安装失败"
        exit 1
    }
    Set-Location ..
    Write-Success "npm 依赖安装完成"
} else {
    Write-Success "npm 依赖已存在，跳过安装"
}

# ============================================================
# 验证环境
# ============================================================
Write-Status "验证环境配置"

$allOk = $true

if (Test-CommandExists "cargo") {
    $v = cargo --version 2>&1
    Write-Host "Rust: $v"
} else {
    Write-Error "cargo 命令未找到"
    $allOk = $false
}

if (Test-CommandExists "node") {
    $v = node --version 2>&1
    Write-Host "Node.js: $v"
} else {
    Write-Error "node 命令未找到"
    $allOk = $false
}

if (Test-CommandExists "tauri") {
    $v = tauri --version 2>&1
    Write-Host "Tauri CLI: $v"
} else {
    Write-Error "tauri 命令未找到"
    $allOk = $false
}

if ($allOk) {
    Write-Success "所有环境检查通过！"
} else {
    Write-Warning "部分环境检查未通过，请重启 PowerShell 重试"
}

# ============================================================
# 启动开发服务器
# ============================================================
Write-Status "启动开发服务器"
Write-Warning "开发服务器启动后会弹出窗口，请等待应用加载..."

Start-Process -FilePath "powershell" -ArgumentList "-Command", "cd '$PWD'; cargo tauri dev" -Wait

Write-Status "安装脚本执行完成"
