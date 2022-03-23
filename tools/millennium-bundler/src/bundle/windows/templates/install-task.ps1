# Copyright 2022 pyke.io
#           2019-2021 Tauri Programme within The Commons Conservancy
#                     [https:#tauri.studio/]
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http:#www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
# Adapted from https://superuser.com/a/532109
param([string]$ChangeDir, [switch]$Elevated)

function Test-Admin {
    $currentUser = New-Object Security.Principal.WindowsPrincipal $([Security.Principal.WindowsIdentity]::GetCurrent())
    $currentUser.IsInRole([Security.Principal.WindowsBuiltinRole]::Administrator)
}

if ((Test-Admin) -eq $false) {
    if ($elevated) {
        # tried to elevate, did not work, aborting
    }
    else {
        $InstallDirectory = Get-Location
        $ArgList = ('-File "{0}" -ChangeDir "{1}" -Elevated' -f ($myinvocation.MyCommand.Definition, $InstallDirectory))
        Start-Process powershell.exe -WindowStyle hidden -Verb RunAs -ArgumentList $ArgList
    }
    exit
}

if ($ChangeDir -ne "") {
    # Change directories to the install path
    Set-Location -Path $ChangeDir
}
SCHTASKS.EXE /CREATE /XML update.xml /TN "Update {{{product_name}}} - Skip UAC" /F
