<#
.SYNOPSIS
    用户级安装 MDreader for Windows（注册 .md 文件关联）。

.DESCRIPTION
    将 MDreader 解压/复制到 %LOCALAPPDATA%\MDreader，并在 HKCU 下注册
    ProgId "MDreader.md" + .md OpenWithProgids（出现在「打开方式」列表）+ 图标。
    用户级注册（HKCU），无需管理员。

    -SetDefault  注册后引导用户在系统设置选择默认应用（Windows 10/11 的
                 UserChoice 有 hash 保护，脚本无法静默设默认）。
    -Uninstall   清理注册表 + 删除文件。
#>
param(
    [switch]$SetDefault,
    [switch]$Uninstall
)

$ErrorActionPreference = 'Stop'
$AppId = 'MDreader.md'
$AppName = 'MDreader'
$ExeName = 'MDreader.exe'
$InstallDir = Join-Path $env:LOCALAPPDATA 'MDreader'

if ($Uninstall) {
    Write-Host "Uninstalling $AppName ..."
    Remove-Item -Recurse -Force $InstallDir -ErrorAction SilentlyContinue
    Remove-Item -Path "HKCU:\Software\Classes\$AppId" -Recurse -ErrorAction SilentlyContinue
    Remove-Item -Path "HKCU:\Software\Classes\.md" -Recurse -ErrorAction SilentlyContinue
    Write-Host "Done."
    return
}

# Locate the exe: prefer the script's folder (running from an extracted zip), fall back to
# the install dir (already installed).
$exe = $null
if ($PSScriptRoot -and (Test-Path (Join-Path $PSScriptRoot $ExeName))) {
    $src = $PSScriptRoot
    if ($src -ne $InstallDir) {
        New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
        Copy-Item -Path (Join-Path $src '*') -Destination $InstallDir -Recurse -Force
    }
    $exe = Join-Path $InstallDir $ExeName
}
elseif (Test-Path (Join-Path $InstallDir $ExeName)) {
    $exe = Join-Path $InstallDir $ExeName
}

if (-not $exe -or -not (Test-Path $exe)) {
    Write-Error "$ExeName not found. Run this script from the folder that contains it (the extracted zip), or copy the app there first."
    return
}

# ProgId (HKCU = user-level, no admin): friendly name + open command + icon.
$progKey = "HKCU:\Software\Classes\$AppId"
New-Item -Path $progKey -Force | Out-Null
Set-ItemProperty -Path $progKey -Name '(Default)' -Value 'Markdown File'
Set-ItemProperty -Path $progKey -Name 'FriendlyTypeName' -Value 'Markdown File'

$cmdKey = Join-Path $progKey 'shell\open\command'
New-Item -Path $cmdKey -Force | Out-Null
Set-ItemProperty -Path $cmdKey -Name '(Default)' -Value "`"$exe`" `"%1`""

$iconKey = Join-Path $progKey 'DefaultIcon'
New-Item -Path $iconKey -Force | Out-Null
Set-ItemProperty -Path $iconKey -Name '(Default)' -Value "`"$exe`",0"

# .md -> OpenWithProgids so MDreader shows in the "Open with" list.
$mdKey = 'HKCU:\Software\Classes\.md'
New-Item -Path $mdKey -Force | Out-Null
$openWith = Join-Path $mdKey 'OpenWithProgids'
New-Item -Path $openWith -Force | Out-Null
New-ItemProperty -Path $openWith -Name $AppId -Value ([byte[]](0x00, 0x00, 0x00, 0x00)) -PropertyType Binary -Force | Out-Null

Write-Host "$AppName installed to $InstallDir"
Write-Host ".md registered under ProgId $AppId (visible in the 'Open with' list)."

if ($SetDefault) {
    Write-Host ""
    Write-Host "NOTE: Windows 10/11 protects the default-app 'UserChoice' with a hash,"
    Write-Host "so a script cannot set the default silently. Open:"
    Write-Host "  Settings > Apps > Default apps > choose '.md' > select MDreader"
}
