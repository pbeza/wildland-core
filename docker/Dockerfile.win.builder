# escape=`

FROM mcr.microsoft.com/windows/servercore:ltsc2022

SHELL ["powershell", "-Command", "$ErrorActionPreference = 'Stop'; $ProgressPreference = 'SilentlyContinue';"]

# Download Visual Studio 2019 online installer
ADD https://aka.ms/vs/17/release/channel C:\TEMP\VisualStudio2019.chman
ADD https://aka.ms/vs/17/release/vs_buildtools.exe C:\TEMP\vs_buildtools2019.exe

SHELL ["cmd", "/S", "/C"]

# Install VS2019 build tools and resources
RUN C:\TEMP\vs_buildtools2019.exe --quiet --wait --norestart --nocache `
    --installPath C:\BuildTools `
    --includeRecommended `
    --channelUri C:\Temp\VisualStudio2019.chman `
    --installChannelUri C:\Temp\VisualStudio2019.chman `
    --add Microsoft.VisualStudio.Workload.VCTools `
    --add Microsoft.VisualStudio.Workload.AzureBuildTools `
    --add Microsoft.VisualStudio.Workload.UniversalBuildTools `
    --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
    --remove Microsoft.VisualStudio.Component.Windows10SDK.10240 `
    --remove Microsoft.VisualStudio.Component.Windows10SDK.10586 `
    --remove Microsoft.VisualStudio.Component.Windows10SDK.14393 `
    --remove Microsoft.VisualStudio.Component.Windows81SDK `
|| IF "%ERRORLEVEL%"=="3010" EXIT 0

SHELL ["powershell", "-Command", "$ErrorActionPreference = 'Stop'; $ProgressPreference = 'SilentlyContinue';"]

# Install chocolatey
RUN Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

RUN choco install --yes swig
RUN choco install --yes rustup.install
RUN choco install --yes sed
RUN choco install --yes nano

ENTRYPOINT ["C:\\BuildTools\\Common7\\Tools\\VsDevCmd.bat", "-host_arch=amd64", "-arch=amd64", "&&", "powershell.exe", "-NoLogo", "-ExecutionPolicy", "Bypass"]
