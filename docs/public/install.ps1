#!/usr/bin/env pwsh
# Uzumaki installer for Windows (PowerShell)
# Usage: irm https://uzumaki.run/install.ps1 | iex

$ErrorActionPreference = 'Stop'

if ($v)
{
  $Version = "v${v}"
}
if ($Args.Length -eq 1)
{
  $Version = $Args.Get(0)
}

$Repo = "golok727/uzumaki"
$UzumakiInstall = $env:UZUMAKI_INSTALL
$BinDir = if ($UzumakiInstall)
{
  "${UzumakiInstall}\bin"
} else
{
  "${Home}\.uzumaki\bin"
}

$UzumakiZip = "$BinDir\uzumaki.zip"
$UzumakiExe = "$BinDir\uzumaki.exe"
$Target = 'windows-x64'

$Version = if (!$Version)
{
  (curl.exe -s "https://api.github.com/repos/$Repo/releases/latest" | ConvertFrom-Json).tag_name
} else
{
  $Version
}

$DownloadUrl = "https://github.com/$Repo/releases/download/${Version}/uzumaki-${Target}.zip"

if (!(Test-Path $BinDir))
{
  New-Item $BinDir -ItemType Directory | Out-Null
}

curl.exe -Lo $UzumakiZip $DownloadUrl

tar.exe xf $UzumakiZip -C $BinDir

Remove-Item $UzumakiZip

$User = [System.EnvironmentVariableTarget]::User
$Path = [System.Environment]::GetEnvironmentVariable('Path', $User)
if (!(";${Path};".ToLower() -like "*;${BinDir};*".ToLower()))
{
  [System.Environment]::SetEnvironmentVariable('Path', "${Path};${BinDir}", $User)
  $Env:Path += ";${BinDir}"
}

Write-Output "Uzumaki was installed successfully to ${UzumakiExe}"
Write-Output "Run 'uzumaki --help' to get started. You may need to restart your terminal."
