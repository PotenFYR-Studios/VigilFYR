param(
  [switch]$DryRun
)

$ErrorActionPreference = "Stop"
$Repo = "PotenFYR-Studios/VigilFYR"
$InstallDir = if ($env:VIGIL_INSTALL_DIR) { $env:VIGIL_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA "Programs/Vigil" }
$Architecture = switch ([Runtime.InteropServices.RuntimeInformation].GetProperty("OSArchitecture").GetValue($null).ToString()) {
  "X64" { "x86_64" }
  "Arm64" { "arm64" }
  default { throw "unsupported architecture" }
}
$Target = if ($Architecture -eq "arm64") { "aarch64-pc-windows-msvc" } else { "x86_64-pc-windows-msvc" }
$Asset = if ($env:VIGIL_RELEASE_URL) { $env:VIGIL_RELEASE_URL } else { "https://github.com/$Repo/releases/latest/download" }
$Asset = "$Asset/vigil-windows-$Target.tar.gz"

Write-Host "install plan: vigil-windows-$Architecture ($Target).tar.gz -> $InstallDir"
if ($DryRun) { exit 0 }

$Temp = New-Item -ItemType Directory -Path (Join-Path $env:TEMP ([Guid]::NewGuid().ToString()))
try {
  Invoke-WebRequest -Uri "$Asset.sha256" -OutFile (Join-Path $Temp "vigil.tar.gz.sha256")
  Invoke-WebRequest -Uri $Asset -OutFile (Join-Path $Temp "vigil.tar.gz")
  $Expected = (Get-Content (Join-Path $Temp "vigil.tar.gz.sha256")).Split(" ")[0].Trim()
  $Actual = (Get-FileHash -Algorithm SHA256 (Join-Path $Temp "vigil.tar.gz")).Hash.ToLower()
  if ($Expected -ne $Actual) { throw "checksum mismatch" }
  tar -xzf (Join-Path $Temp "vigil.tar.gz") -C $Temp.FullName
  New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
  $Binary = Get-ChildItem -Path $Temp.FullName -Filter vigil.exe -Recurse | Select-Object -First 1
  Copy-Item $Binary.FullName (Join-Path $InstallDir "vigil.exe") -Force
  Write-Host "installed. run: vigil setup"
}
finally {
  Remove-Item $Temp.FullName -Recurse -Force -ErrorAction SilentlyContinue
}
