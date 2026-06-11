[CmdletBinding()]
param(
  [string]$RepoUrl = "https://github.com/NousResearch/hermes-agent.git",
  [string]$Destination = "",
  [string]$Ref = "main",
  [switch]$Force,
  [switch]$PlanOnly
)

$ErrorActionPreference = "Stop"

function Normalize-FullPathForBoundary([string]$Path) {
  return [System.IO.Path]::GetFullPath($Path).TrimEnd(
    [System.IO.Path]::DirectorySeparatorChar,
    [System.IO.Path]::AltDirectorySeparatorChar
  )
}

function Test-IsReparsePoint([string]$Path) {
  $item = Get-Item -LiteralPath $Path -Force
  return (($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -eq [System.IO.FileAttributes]::ReparsePoint)
}

function Assert-NoReparsePointInDestinationPath([string]$ExternalRoot, [string]$DestinationFull) {
  $externalRootFull = Normalize-FullPathForBoundary $ExternalRoot
  $destinationFullNormalized = Normalize-FullPathForBoundary $DestinationFull
  $destinationParent = Normalize-FullPathForBoundary ([System.IO.Path]::GetDirectoryName($destinationFullNormalized))
  $externalRootWithSeparator = $externalRootFull + [System.IO.Path]::DirectorySeparatorChar

  if (-not $destinationFullNormalized.StartsWith($externalRootWithSeparator, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "Destination must stay inside $externalRootFull. Received: $destinationFullNormalized"
  }

  $pathsToCheck = New-Object System.Collections.Generic.List[string]
  $current = $destinationParent
  while ($true) {
    $pathsToCheck.Add($current)

    if ($current.Equals($externalRootFull, [System.StringComparison]::OrdinalIgnoreCase)) {
      break
    }

    $parent = [System.IO.Path]::GetDirectoryName($current)
    if ([string]::IsNullOrWhiteSpace($parent)) {
      throw "Destination must stay inside $externalRootFull. Received: $destinationFullNormalized"
    }

    $current = Normalize-FullPathForBoundary $parent
  }

  foreach ($pathToCheck in $pathsToCheck) {
    if ((Test-Path -LiteralPath $pathToCheck) -and (Test-IsReparsePoint $pathToCheck)) {
      throw "Destination path crosses a reparse point: $pathToCheck"
    }
  }

  if ((Test-Path -LiteralPath $destinationFullNormalized) -and (Test-IsReparsePoint $destinationFullNormalized)) {
    throw "Destination is a reparse point: $destinationFullNormalized"
  }
}

function Invoke-Git {
  param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Arguments)

  & git @Arguments
  if ($LASTEXITCODE -ne 0) {
    throw "git $($Arguments -join ' ') failed with exit code $LASTEXITCODE"
  }
}

$repoRoot = Normalize-FullPathForBoundary (Join-Path $PSScriptRoot "..\..")
$externalRoot = Normalize-FullPathForBoundary (Join-Path $repoRoot ".external")
if ([string]::IsNullOrWhiteSpace($Destination)) {
  $Destination = Join-Path $externalRoot "hermes-agent"
}
$destinationFull = Normalize-FullPathForBoundary $Destination
$externalRootWithSeparator = $externalRoot + [System.IO.Path]::DirectorySeparatorChar

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

if (-not $destinationFull.StartsWith($externalRootWithSeparator, [System.StringComparison]::OrdinalIgnoreCase)) {
  throw "Destination must stay inside $externalRoot. Received: $destinationFull"
}

Assert-NoReparsePointInDestinationPath $externalRoot $destinationFull

if ($PlanOnly) {
  [PSCustomObject]@{
    repo_url = $RepoUrl
    destination = $destinationFull
    ref = $Ref
    sparse_paths = $sparsePaths
  } | ConvertTo-Json -Depth 4
  exit 0
}

New-Item -ItemType Directory -Force -Path $externalRoot | Out-Null
Assert-NoReparsePointInDestinationPath $externalRoot $destinationFull

if (Test-Path $destinationFull) {
  if (-not $Force) {
    throw "Destination already exists: $destinationFull. Re-run with -Force to replace it."
  }
  Remove-Item -LiteralPath $destinationFull -Recurse -Force
}

Assert-NoReparsePointInDestinationPath $externalRoot $destinationFull

Invoke-Git --version
Invoke-Git clone --depth 1 --filter=blob:none --sparse --branch $Ref '--' $RepoUrl $destinationFull
Invoke-Git -C $destinationFull sparse-checkout set --no-cone @sparsePaths
Invoke-Git -C $destinationFull status --short

[PSCustomObject]@{
  status = "fetched"
  destination = $destinationFull
  ref = $Ref
} | ConvertTo-Json -Depth 3
