$roSokuURL = "https://github.com/mirumirumi/ro-soku/releases/latest/download/ro-soku.exe"
$roSokuFolder = "$env:USERPROFILE\AppData\Local\ro-soku"
$roSokuPath = Join-Path -Path $roSokuFolder -ChildPath "ro-soku.exe"

if (!(Test-Path -Path $roSokuFolder)) {
    New-Item -ItemType Directory -Path $roSokuFolder
}

Invoke-WebRequest -Uri $roSokuURL -OutFile $roSokuPath

$oldPath = (Get-ItemProperty -Path 'Registry::HKEY_CURRENT_USER\Environment' -Name 'PATH').PATH
$newPath = "$roSokuFolder;$oldPath"

Set-ItemProperty -Path 'Registry::HKEY_CURRENT_USER\Environment' -Name 'PATH' -Value $newPath

Write-Host "ro-soku has been installed!"
Write-Host "(command will not respond unless a new tab or window is opened)"
