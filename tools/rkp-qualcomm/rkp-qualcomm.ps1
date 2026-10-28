<#
RKP keybox 签发（高通 / Duck ToolBox 路线）。

一条命令：检查 SoC → 把引擎送上设备 → 让 TA 对我们的公钥签 CSR → 引擎去 Google 换链 →
拉回 keybox.xml + 私钥 → 本地验链/验配对 → 清设备痕迹。

用法：
  .\rkp-qualcomm.ps1 -Device <序列号> -EngineUrl http://host/rkp-engine.gz -ProxyUrl http://u:p@host:port
  .\rkp-qualcomm.ps1 -Device <序列号> -EnginePath <本地 duckd> -ProxyUrl ...
  .\rkp-qualcomm.ps1 -Device <序列号> -EnginePath <本地 duckd> -NoProxy

细节和踩过的坑见同目录 README.md。
#>
[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)][string]$Device,
  [string]$EnginePath,
  [string]$EngineUrl,
  [string]$ProxyUrl,
  [switch]$NoProxy,
  [string]$OutDir = 'D:\keys\rkp-keyboxes',
  [string]$Adbr = 'D:\platform-tools\adbr.exe',
  [int]$TimeoutSec = 240,
  [string]$Model,
  [switch]$KeepDevice
)

$ErrorActionPreference = 'Stop'
$edge = '/data/local/tmp'
$engineRemote = "$edge/rkp-engine"
$outRemote = "$edge/keybox.xml"
$logRemote = "$edge/rkp.out"
$rootRemote = "$edge/duckroot"
$dataRemote = "$edge/duckdata"

function Adbr {
  param([string[]]$A)
  & $Adbr -s $Device @A 2>&1
}

function AdbrText {
  param([string]$Cmd)
  ((Adbr @('shell', $Cmd)) -join "`n")
}

# 设备侧执行一段脚本（避免 PowerShell / sh 的引号互相踩）
function RunScriptOnDevice {
  param([string]$Body)
  $b64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($Body))
  Adbr @('shell', "printf '%s' '$b64' | base64 -d > $edge/_x.sh; sh $edge/_x.sh; rm -f $edge/_x.sh") | Out-Null
  AdbrText "echo ok"
}

function Fail($msg) { Write-Host "失败：$msg" -ForegroundColor Red; exit 1 }

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

# ---- 1) 设备基本检查 ----
$props = AdbrText "getprop ro.soc.manufacturer; getprop ro.hardware; getprop ro.soc.model; getprop ro.product.model; getprop ro.build.version.release"
$p = @($props) -join "`n"
$lines = @($props) | ForEach-Object { ($_ -replace "`r", '').Trim() } | Where-Object { $_ -ne '' }
if ($lines.Count -lt 5) { Fail "读不到设备属性（型号/SoC），检查 adbr 是否在线：$p" }
$socMan, $hw, $socModel, $model, $android = $lines[0..4]
if (-not $Model) { $Model = $model }
Write-Host "设备 $Model（serial=$Device）Android $android  SoC=$socMan/$hw/$socModel"
if ($socMan -notmatch 'QTI|Qualcomm' -or $hw -notmatch 'qcom') { Fail "不是高通设备（$socMan/$hw），这条路只支持高通 TA" }

# ---- 2) 引擎上设备 ----
if ($EngineUrl) {
  $url = $EngineUrl
  $isGz = $url -match '\.gz$'
  $tmpRemote = if ($isGz) { "$edge/e.gz" } else { $engineRemote }
  Write-Host "设备侧下载引擎：$url"
  $dl = AdbrText "curl -s -w '%{http_code}' --max-time 240 -o $tmpRemote '$url'"
  if ($dl -notmatch '200') { Fail "下载失败（$dl）" }
  if ($isGz) { AdbrText "cd $edge; gunzip -f e.gz; mv -f e rkp-engine" | Out-Null }
} elseif ($EnginePath) {
  if (-not (Test-Path $EnginePath)) { Fail "引擎不存在：$EnginePath" }
  Write-Host "adbr push 引擎（数据通道不通时会失败，此时改用 -EngineUrl）"
  $push = Adbr @('push', $EnginePath, $engineRemote)
  if ($push -match '10054|失败|failed') { Fail "push 失败，改用 -EngineUrl 让设备自己下载：$push" }
} else {
  Fail "要么 -EnginePath，要么 -EngineUrl"
}
AdbrText "chmod 755 $engineRemote; $engineRemote --version" | ForEach-Object { Write-Host "  引擎：$_" }

# ---- 3) 假模块根（引擎要求） ----
AdbrText "mkdir -p $rootRemote/bin $dataRemote; echo id=rkp > $rootRemote/module.prop; ls $rootRemote" | Out-Null

# ---- 4) 签发 ----
$proxy = ''
if (-not $NoProxy) {
  if (-not $ProxyUrl) { Fail "没给 -ProxyUrl（国行机连不上 Google）；出厂网络可直连就加 -NoProxy" }
  $proxy = "HTTPS_PROXY=$ProxyUrl HTTP_PROXY=$ProxyUrl ALL_PROXY=$ProxyUrl"
}
AdbrText "su -c 'rm -f $outRemote $logRemote; mkdir -p $dataRemote; nohup env PATH=/system/bin:/system/xbin:/vendor/bin DUCK_TOOLBOX_ROOT=$rootRemote DUCK_TOOLBOX_DATA_ROOT=$dataRemote $proxy $engineRemote rkp-device provision --acknowledge-limitations --output $outRemote > $logRemote 2>&1 &'" | Out-Null

Write-Host "已下发，等结果（最多 $TimeoutSec 秒）…"
$deadline = (Get-Date).AddSeconds($TimeoutSec)
$res = ''
while ((Get-Date) -lt $deadline) {
  Start-Sleep -Seconds 5
  $txt = AdbrText "tail -c 4000 $logRemote 2>/dev/null"
  if ($txt -match '"command":"rkp-device\.provision"') { $res = $txt; break }
}
if (-not $res) { Fail "超时，没拿到 provision 结果；设备日志：$txt" }

$ok = $res -match '"ok":true'
if (-not $ok) {
  $err = [regex]::Match($res, '"error":\{[^}]*\}').Value
  Write-Host "设备侧失败：$err" -ForegroundColor Yellow
}

$kbLocal = $null; $keyLocal = $null
if ($ok) {
  # ---- 5) 拉回本地 ----
  $kbB64 = (AdbrText "su -c 'base64 -w0 $outRemote'") -replace '[^A-Za-z0-9+/=]', ''
  $run = [regex]::Match((AdbrText "su -c 'ls -d $dataRemote/var/outputs/rkp-device-* 2>/dev/null | tail -1'"), 'rkp-device-\S+').Value
  if (-not $run) { Fail "找不到运行目录，拿不到私钥" }
  $keyB64 = (AdbrText "su -c 'base64 -w0 $run/private_key.pem'") -replace '[^A-Za-z0-9+/=]', ''
  if ($kbB64.Length -lt 100 -or $keyB64.Length -lt 100) { Fail "拉取产物为空" }

  $base = Join-Path $OutDir ("{0}-{1}-rkp" -f $Model, $Device)
  if (Test-Path "$base.xml") { $base = "{0}-{1}" -f $base, (Get-Date -Format 'yyyyMMdd-HHmmss') }
  $kbLocal = "$base.xml"; $keyLocal = "$base.key.pem"
  [IO.File]::WriteAllBytes($kbLocal, [Convert]::FromBase64String($kbB64))
  [IO.File]::WriteAllBytes($keyLocal, [Convert]::FromBase64String($keyB64))
  Write-Host "已保存：$kbLocal`n        $keyLocal"
}

# ---- 6) 清痕迹（失败也执行） ----
function Cleanup {
  if ($KeepDevice) { Write-Host '(-KeepDevice：跳过清理)'; return }
  AdbrText "su -c 'rm -rf $engineRemote $edge/rkp-sign.sh $edge/rkp.out $edge/keybox.xml $edge/e.gz $edge/eng $rootRemote $dataRemote /data/adb/duck-toolbox; if [ -f $edge/adb_keys.orig ]; then cp -f $edge/adb_keys.orig /data/misc/adb/adb_keys; rm -f $edge/adb_keys.orig; fi; setprop service.adb.tcp.port -1; echo cleaned'" | Out-Null
  $left = AdbrText "ls /data/local/tmp | grep -iE 'rkp|duck' || echo clean"
  Write-Host "设备清理：$left"
}
try { Cleanup } finally { }

if (-not $ok) { exit 1 }

# ---- 7) 本地验链 + 验配对 ----
$xml = Get-Content $kbLocal -Raw
$pems = [regex]::Matches($xml, '-----BEGIN CERTIFICATE-----.*?-----END CERTIFICATE-----', 'Singleline') | ForEach-Object { $_.Value }
Write-Host ("证书链：{0} 段" -f $pems.Count)
$paths = @()
for ($i = 0; $i -lt $pems.Count; $i++) {
  $f = Join-Path $env:TEMP ("rkp-c{0}.pem" -f ($i + 1))
  ($pems[$i] -split "`n" | ForEach-Object { $_.Trim() }) -join "`n" | Set-Content -Encoding ascii $f
  $paths += $f
  $s = (& openssl x509 -in $f -noout -subject -issuer -enddate 2>&1) -join ' | '
  Write-Host ("  cert{0}: {1}" -f ($i + 1), ($s -replace '\s+', ' '))
}
$chainOk = $true
for ($i = 0; $i -lt $paths.Count - 1; $i++) {
  $r = & openssl verify -CAfile $paths[$i + 1] $paths[$i] 2>&1
  if ($LASTEXITCODE -ne 0) { $chainOk = $false; Write-Host ("  验签 cert{0}/cert{1} 失败: {2}" -f ($i + 1), ($i + 2), $r) -ForegroundColor Yellow }
}
$kp = (& openssl ec -in $keyLocal -pubout 2>&1) -join "`n"
$cp = (& openssl x509 -in $paths[0] -pubkey -noout 2>&1) -join "`n"
$keyOk = ($kp -replace '\s', '') -eq ($cp -replace '\s', '')
Write-Host ("链验签：{0}   私钥配对：{1}" -f $(if ($chainOk) { 'OK' } else { '有问题' }), $(if ($keyOk) { 'OK' } else { '不匹配' }))

# keybox 的 DeviceID 也打出来，方便核对
$dev = [regex]::Match($xml, 'DeviceID="([^"]+)"').Groups[1].Value
Write-Host "DeviceID：$dev"
if (-not ($chainOk -and $keyOk)) { exit 2 }
Write-Host '完成。' -ForegroundColor Green
