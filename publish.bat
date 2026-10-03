@echo off
setlocal enabledelayedexpansion

echo =======================================================
echo        VELOTYPE - ONE-STEP GITHUB PUBLISHER (WINDOWS)
echo =======================================================
echo.

set REPO_OWNER=Saboor-Hamedi
set REPO_NAME=typingforge
set VERSION=v0.1.0

git init
git config user.name "%REPO_OWNER%"
git config user.email "saboorhamedi49@gmail.com"
git branch -M main

git add -A
git commit -m "Release Velotype: Kinetic Typing Game with Real-Time Velocity Graph"

git remote remove origin >nul 2>&1
git remote add origin "https://github.com/%REPO_OWNER%/%REPO_NAME%.git"

where gh >nul 2>&1
if %ERRORLEVEL% equ 0 (
    echo [INFO] GitHub CLI detected. Verifying public repository...
    gh repo create "%REPO_OWNER%/%REPO_NAME%" --public --source=. --remote=origin
) else (
    echo [INFO] GitHub CLI not detected; pushing via Git credentials.
)

git tag -d %VERSION% >nul 2>&1
git tag -a %VERSION% -m "Release %VERSION%"
echo [INFO] Pushing main branch and release tags to GitHub...
git push -u origin main --tags --force

echo.
echo =======================================================
echo [SUCCESS] Pushed to https://github.com/%REPO_OWNER%/%REPO_NAME%
echo GitHub Actions will now automatically build:
echo   1. Windows Installer (velotype-windows-setup.exe)
echo   2. macOS Universal DMG
echo   3. Linux tar.gz
echo =======================================================
pause
