#!/usr/bin/env bash

create-dmg \
    --volname "Telegram Air installer" \
    --volicon "./tauri/icons/icon.icns" \
    --background "./tauri/images/background-dmg.tiff" \
    --window-size 540 380 \
    --icon-size 100 \
    --icon "Telegram Air.app" 138 225 \
    --hide-extension "Telegram Air.app" \
    --app-drop-link 402 225 \
    --no-internet-enable \
    "$1" \
    "$2"
