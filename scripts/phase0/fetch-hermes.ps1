[CmdletBinding()]
param(
  [string]$RepoUrl = "https://github.com/NousResearch/hermes-agent.git",
  [string]$Destination = "",
  [string]$Ref = "main",
  [switch]$Force,
  [switch]$PlanOnly
)

$ErrorActionPreference = "Stop"

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..\.."))
$externalRoot = [System.IO.Path]::GetFullPath((Join-Path $repoRoot ".external")).TrimEnd(
  [System.IO.Path]::DirectorySeparatorChar,
  [System.IO.Path]::AltDirectorySeparatorChar
)
$externalRootWithSeparator = "$externalRoot$([System.IO.Path]::DirectorySeparatorChar)"
if ([string]::IsNullOrWhiteSpace($Destination)) {
  $Destination = Join-Path $externalRoot "hermes-agent"
}
$destinationFull = [System.IO.Path]::GetFullPath($Destination)

$sparsePaths = @(
  "README.md",
  "LICENSE",
  "pyproject.toml",
  "uv.lock",
  "run_agent.py",
  "model_tools.py",
  "toolsets.py",
  "mcp_serve.py",
  "hermes_state.py",
  "hermes_constants.py",
  "agent",
  "tools",
  "skills",
  "optional-skills",
  "plugins",
  "providers",
  "hermes_cli",
  "gateway",
  "website/docs",
  "apps/desktop/package.json",
  "apps/desktop/src",
  "apps/desktop/electron",
  "apps/desktop/DESIGN.md",
  "apps/desktop/README.md"
)

if ($PlanOnly) {
  [PSCustomObject]@{
    repo_url = $RepoUrl
    destination = $destinationFull
    ref = $Ref
    sparse_paths = $sparsePaths
  } | ConvertTo-Json -Depth 4
  exit 0
}

if (-not $destinationFull.StartsWith($externalRootWithSeparator, [System.StringComparison]::OrdinalIgnoreCase)) {
  throw "Destination must stay inside $externalRoot. Received: $destinationFull"
}

New-Item -ItemType Directory -Force -Path $externalRoot | Out-Null

if (Test-Path $destinationFull) {
  if (-not $Force) {
    throw "Destination already exists: $destinationFull. Re-run with -Force to replace it."
  }
  Remove-Item -LiteralPath $destinationFull -Recurse -Force
}

git --version | Out-Host
git clone --depth 1 --filter=blob:none --sparse --branch $Ref $RepoUrl $destinationFull
git -C $destinationFull sparse-checkout set --no-cone @sparsePaths
git -C $destinationFull status --short

[PSCustomObject]@{
  status = "fetched"
  destination = $destinationFull
  ref = $Ref
} | ConvertTo-Json -Depth 3
