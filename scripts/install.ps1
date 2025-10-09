# Uveddi Windows Installation Script
# PowerShell 5.1+ compatible
#
# Usage:
#   irm https://uveddi.io/install.ps1 | iex
#   Invoke-WebRequest -Uri https://uveddi.io/install.ps1 -UseBasicParsing | Invoke-Expression
#
# Environment Variables:
#   $env:UVEDDI_VERSION - Specific version to install (default: latest)
#   $env:UVEDDI_INSTALL_DIR - Installation directory (default: %USERPROFILE%\.local\bin)
#   $env:UVEDDI_VERIFY_CHECKSUM - Verify SHA256 checksum (default: true)

param(
    [string]$Version = $env:UVEDDI_VERSION,
    [string]$InstallDir = $env:UVEDDI_INSTALL_DIR,
    [switch]$Uninstall,
    [switch]$Force
)

# ============================================================================
# Configuration
# ============================================================================

$ErrorActionPreference = "Stop"

if (-not $Version) {
    $Version = "1.0.0"
}

if (-not $InstallDir) {
    $InstallDir = Join-Path $env:USERPROFILE ".local\bin"
}

$ProjectName = "uveddi"
$BaseUrl = "https://github.com/botzrDev/uveddi/releases/download"
$TempDir = Join-Path $env:TEMP "uveddi-install-$(Get-Random)"

# ============================================================================
# Helper Functions
# ============================================================================

function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Level = "Info"
    )
    
    $color = switch ($Level) {
        "Success" { "Green" }
        "Warning" { "Yellow" }
        "Error" { "Red" }
        "Info" { "Cyan" }
        default { "White" }
    }
    
    $prefix = switch ($Level) {
        "Success" { "[SUCCESS]" }
        "Warning" { "[WARNING]" }
        "Error" { "[ERROR]" }
        "Info" { "[INFO]" }
        default { "" }
    }
    
    Write-Host "$prefix $Message" -ForegroundColor $color
}

function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# ============================================================================
# Cleanup
# ============================================================================

function Remove-TempDirectory {
    if (Test-Path $TempDir) {
        try {
            Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue
        } catch {
            # Ignore cleanup errors
        }
    }
}

# ============================================================================
# Platform Detection
# ============================================================================

function Get-Platform {
    $arch = $env:PROCESSOR_ARCHITECTURE
    
    switch ($arch) {
        "AMD64" { return "x86_64-pc-windows-gnu" }
        "x86" { 
            Write-ColorOutput "32-bit Windows is not supported" "Error"
            exit 1
        }
        "ARM64" {
            Write-ColorOutput "ARM64 Windows support is experimental" "Warning"
            return "aarch64-pc-windows-gnu"
        }
        default {
            Write-ColorOutput "Unsupported architecture: $arch" "Error"
            exit 1
        }
    }
}

# ============================================================================
# Download Functions
# ============================================================================

function Download-File {
    param(
        [string]$Url,
        [string]$OutputPath
    )
    
    Write-ColorOutput "Downloading: $([System.IO.Path]::GetFileName($Url))" "Info"
    
    try {
        # Use .NET WebClient for better compatibility
        $webClient = New-Object System.Net.WebClient
        $webClient.DownloadFile($Url, $OutputPath)
        $webClient.Dispose()
    } catch {
        # Fallback to Invoke-WebRequest
        try {
            Invoke-WebRequest -Uri $Url -OutFile $OutputPath -UseBasicParsing
        } catch {
            Write-ColorOutput "Failed to download: $Url" "Error"
            Write-ColorOutput $_.Exception.Message "Error"
            throw
        }
    }
}

# ============================================================================
# Verification Functions
# ============================================================================

function Test-Checksum {
    param(
        [string]$FilePath,
        [string]$ChecksumFile
    )
    
    Write-ColorOutput "Verifying checksum" "Info"
    
    $fileName = [System.IO.Path]::GetFileName($FilePath)
    $checksumContent = Get-Content $ChecksumFile
    
    # Find the line with our file
    $checksumLine = $checksumContent | Where-Object { $_ -match [regex]::Escape($fileName) }
    
    if (-not $checksumLine) {
        Write-ColorOutput "Checksum not found for $fileName" "Warning"
        return $true
    }
    
    # Extract expected checksum (first part before spaces)
    $expectedChecksum = ($checksumLine -split '\s+')[0]
    
    # Calculate actual checksum
    $actualChecksum = (Get-FileHash -Path $FilePath -Algorithm SHA256).Hash.ToLower()
    
    if ($expectedChecksum -ne $actualChecksum) {
        Write-ColorOutput "Checksum verification failed!" "Error"
        Write-ColorOutput "Expected: $expectedChecksum" "Error"
        Write-ColorOutput "Got: $actualChecksum" "Error"
        return $false
    }
    
    Write-ColorOutput "Checksum verified" "Success"
    return $true
}

# ============================================================================
# Installation Functions
# ============================================================================

function Install-Uveddi {
    param(
        [string]$Platform
    )
    
    $archiveName = "$ProjectName-$Version-$Platform.zip"
    $downloadUrl = "$BaseUrl/v$Version/$archiveName"
    $checksumUrl = "$BaseUrl/v$Version/SHA256SUMS"
    
    # Create temp directory
    New-Item -ItemType Directory -Path $TempDir -Force | Out-Null
    
    try {
        # Download archive
        $archivePath = Join-Path $TempDir $archiveName
        Download-File -Url $downloadUrl -OutputPath $archivePath
        
        # Download checksums
        $checksumPath = Join-Path $TempDir "SHA256SUMS"
        try {
            Download-File -Url $checksumUrl -OutputPath $checksumPath
            
            # Verify checksum
            if (-not (Test-Checksum -FilePath $archivePath -ChecksumFile $checksumPath)) {
                throw "Checksum verification failed"
            }
        } catch {
            Write-ColorOutput "Failed to download or verify checksums" "Warning"
            if (-not $Force) {
                $continue = Read-Host "Continue without checksum verification? (y/N)"
                if ($continue -ne "y" -and $continue -ne "Y") {
                    throw "Installation aborted"
                }
            }
        }
        
        # Extract archive
        Write-ColorOutput "Extracting archive" "Info"
        Expand-Archive -Path $archivePath -DestinationPath $TempDir -Force
        
        # Find binary
        $binaryPath = Get-ChildItem -Path $TempDir -Filter "$ProjectName.exe" -Recurse | Select-Object -First 1
        
        if (-not $binaryPath) {
            throw "Binary not found in archive"
        }
        
        # Create install directory
        if (-not (Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }
        
        # Install binary
        $targetPath = Join-Path $InstallDir "$ProjectName.exe"
        Write-ColorOutput "Installing to $targetPath" "Info"
        
        # Remove existing binary if present
        if (Test-Path $targetPath) {
            Remove-Item -Path $targetPath -Force
        }
        
        Copy-Item -Path $binaryPath.FullName -Destination $targetPath -Force
        
        Write-ColorOutput "Uveddi $Version installed successfully!" "Success"
        
    } finally {
        Remove-TempDirectory
    }
}

# ============================================================================
# Post-installation
# ============================================================================

function Test-PathConfiguration {
    $pathEntries = $env:PATH -split ';'
    
    if ($pathEntries -notcontains $InstallDir) {
        Write-ColorOutput "Installation directory not in PATH" "Warning"
        Write-Host ""
        Write-Host "To add Uveddi to your PATH, run:" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "  [Environment]::SetEnvironmentVariable('Path', `$env:Path + ';$InstallDir', 'User')" -ForegroundColor Cyan
        Write-Host ""
        Write-Host "Then restart your PowerShell session." -ForegroundColor Yellow
        Write-Host ""
        return $false
    }
    return $true
}

function Test-Installation {
    Write-ColorOutput "Verifying installation" "Info"
    
    $binaryPath = Join-Path $InstallDir "$ProjectName.exe"
    
    if (-not (Test-Path $binaryPath)) {
        Write-ColorOutput "Installation verification failed: Binary not found" "Error"
        return $false
    }
    
    try {
        $output = & $binaryPath --version 2>&1
        Write-ColorOutput "Installation verified" "Success"
        Write-Host ""
        Write-Host $output
        return $true
    } catch {
        Write-ColorOutput "Installation verification failed" "Error"
        Write-ColorOutput $_.Exception.Message "Error"
        return $false
    }
}

function Show-NextSteps {
    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "🎉 Uveddi is ready to use!" -ForegroundColor Green
    Write-Host ""
    
    if (-not (Test-PathConfiguration)) {
        Write-Host "After updating your PATH, try:" -ForegroundColor Yellow
    } else {
        Write-Host "Get started with:" -ForegroundColor Green
    }
    
    Write-Host ""
    Write-Host "  uveddi --help"
    Write-Host "  uveddi analyze C:\path\to\code"
    Write-Host ""
    Write-Host "Documentation: https://uveddi.io/docs"
    Write-Host "Support: https://github.com/botzrDev/uveddi/issues"
    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host ""
}

# ============================================================================
# Uninstallation
# ============================================================================

function Uninstall-Uveddi {
    Write-ColorOutput "Uninstalling Uveddi" "Info"
    
    $binaryPath = Join-Path $InstallDir "$ProjectName.exe"
    
    if (Test-Path $binaryPath) {
        Remove-Item -Path $binaryPath -Force
        Write-ColorOutput "Uveddi uninstalled from $InstallDir" "Success"
    } else {
        Write-ColorOutput "Uveddi not found at $binaryPath" "Warning"
    }
    
    # Remove config directory if desired
    $configDir = Join-Path $env:APPDATA "uveddi"
    if (Test-Path $configDir) {
        $remove = Read-Host "Remove configuration directory? (y/N)"
        if ($remove -eq "y" -or $remove -eq "Y") {
            Remove-Item -Path $configDir -Recurse -Force
            Write-ColorOutput "Configuration removed" "Success"
        }
    }
}

# ============================================================================
# Main
# ============================================================================

function Show-Banner {
    Write-Host ""
    Write-Host "╔═══════════════════════════════════════════╗" -ForegroundColor Cyan
    Write-Host "║     Uveddi Installer v$Version            ║" -ForegroundColor Cyan
    Write-Host "╚═══════════════════════════════════════════╝" -ForegroundColor Cyan
    Write-Host ""
}

function Main {
    Show-Banner
    
    # Handle uninstall
    if ($Uninstall) {
        Uninstall-Uveddi
        return
    }
    
    # Check if running as administrator (optional, for system-wide install)
    if ((Test-Administrator) -and ($InstallDir -notmatch "^$([regex]::Escape($env:USERPROFILE))")) {
        Write-ColorOutput "Running as Administrator - installing system-wide" "Info"
    }
    
    # Detect platform
    $platform = Get-Platform
    Write-ColorOutput "Detected platform: $platform" "Info"
    
    # Install
    try {
        Install-Uveddi -Platform $platform
    } catch {
        Write-ColorOutput "Installation failed: $($_.Exception.Message)" "Error"
        exit 1
    }
    
    # Verify
    if (-not (Test-Installation)) {
        Write-ColorOutput "Installation completed but verification failed" "Warning"
    }
    
    # Show next steps
    Show-NextSteps
}

# Run main function
try {
    Main
} catch {
    Write-ColorOutput "Unexpected error: $($_.Exception.Message)" "Error"
    Remove-TempDirectory
    exit 1
}
