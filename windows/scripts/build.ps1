<#
.SYNOPSIS
    打包 Windows 便携版（self-contained zip，解压即用）。
.DESCRIPTION
    dotnet publish -r win-x64 --self-contained，输出到 publish/ 后压缩为
    mdreader-windows-x64.zip。产物可解压即用，无需用户预装 .NET 运行时。
#>
$ErrorActionPreference = 'Stop'

# windows/scripts/ -> windows/
$root = Split-Path -Parent $PSScriptRoot
$proj = Join-Path $root 'MDreader\MDreader.csproj'
$out = Join-Path $root 'publish'
$zip = Join-Path $root 'mdreader-windows-x64.zip'

Remove-Item -Recurse -Force $out -ErrorAction SilentlyContinue
Remove-Item -Force $zip -ErrorAction SilentlyContinue

dotnet publish $proj -c Release -r win-x64 --self-contained true -o $out
if ($LASTEXITCODE -ne 0) { throw "dotnet publish failed" }

Compress-Archive -Path (Join-Path $out '*') -DestinationPath $zip -Force
Write-Host "Published: $zip"
