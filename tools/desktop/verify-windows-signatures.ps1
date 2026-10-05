param([Parameter(Mandatory = $true)][string]$Target)
$ErrorActionPreference = "Stop"
$root = Join-Path $PSScriptRoot "../.."
$release = Join-Path $root "src-tauri/target/$Target/release"
$installers = @(Get-ChildItem (Join-Path $release "bundle/nsis") -Filter "*-setup.exe" -File)
if ($installers.Count -ne 1) { throw "Expected exactly one NSIS installer" }
foreach ($file in @((Join-Path $release "knovum.exe"), $installers[0].FullName)) {
    $signature = Get-AuthenticodeSignature -LiteralPath $file
    if ($signature.Status -ne "Valid") {
        throw "Invalid Authenticode signature on $file`: $($signature.Status)"
    }
    if ($null -eq $signature.TimeStamperCertificate) {
        throw "A timestamped code signature is required: $file"
    }
}
