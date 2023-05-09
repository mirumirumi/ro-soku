Invoke-WebRequest https://github.com/mirumirumi/crock/releases/latest/download/crock.exe -OutFile $env:USERPROFILE\ro-soku.exe
$oldPath = (Get-ItemProperty -Path 'Registry::HKEY_CURRENT_USER\System\CurrentControlSet\Control\Session Manager\Environment' -Name PATH).Path
$newPath = $oldPath + ";$env:USERPROFILE"
Set-ItemProperty -Path 'Registry::HKEY_CURRENT_USER\System\CurrentControlSet\Control\Session Manager\Environment' -Name PATH -Value $newPath
