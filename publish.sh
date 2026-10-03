#!/usr/bin/env bash
set -e

echo -e "\033[1;36m=======================================================\033[0m"
echo -e "\033[1;36m       VELOTYPE — ONE-STEP GITHUB PUBLISHER            \033[0m"
echo -e "\033[1;36m=======================================================\033[0m"
echo ""

REPO_OWNER="Saboor-Hamedi"
REPO_NAME="typingforge"
VERSION="v0.1.0"

# 1. Initialize git and configure
git init
git config user.name "$REPO_OWNER"
git config user.email "saboorhamedi49@gmail.com"
git branch -M main

# 2. Stage and commit
git add -A
git commit -m "Release Velotype: Kinetic Typing Game with Real-Time Velocity Graph" || true

# 3. Configure remote
git remote remove origin 2>/dev/null || true
git remote add origin "https://github.com/$REPO_OWNER/$REPO_NAME.git"

# 4. Use GitHub CLI (gh) if authenticated to create repository if needed
if command -v gh &> /dev/null; then
    echo -e "\033[32m✔ GitHub CLI detected. Verifying repository...\033[0m"
    gh repo create "$REPO_OWNER/$REPO_NAME" --public --source=. --remote=origin || true
else
    echo -e "\033[33mℹ GitHub CLI (gh) not found in PATH; pushing via Git credentials.\033[0m"
fi

# 5. Tag release and push
git tag -d "$VERSION" 2>/dev/null || true
git tag -a "$VERSION" -m "Release $VERSION"
echo -e "\033[1;33mPushing to GitHub (main + tags)...\033[0m"
git push -u origin main --tags --force

echo ""
echo -e "\033[1;32m=======================================================\033[0m"
echo -e "\033[1;32m✔ Successfully pushed to https://github.com/$REPO_OWNER/$REPO_NAME\033[0m"
echo -e "\033[1;32m  GitHub Actions is now compiling the Windows Installer,\033[0m"
echo -e "\033[1;32m  macOS package, and Linux packages automatically!\033[0m"
echo -e "\033[1;32m=======================================================\033[0m"
