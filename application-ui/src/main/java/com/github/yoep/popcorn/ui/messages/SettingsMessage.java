package com.github.yoep.popcorn.ui.messages;

import com.github.spring.boot.javafx.text.Message;
import lombok.Getter;

@Getter
public enum SettingsMessage implements Message {
    TRAKT_LOGIN_TITLE("settings_trakt_login_title"),
    SETTINGS_SAVED("settings_saved"),
    AUTHORIZE("settings_trakt_connect"),
    DISCONNECT("settings_trakt_disconnect"),
    SYNC_SUCCESS("settings_trakt_sync_success"),
    SYNC_FAILED("settings_trakt_sync_failed"),
    LAST_SYNC_STATE("settings_trakt_last_sync_state"),
    LAST_SYNC_TIME("settings_trakt_last_sync_time");

    private final String key;

    SettingsMessage(String key) {
        this.key = key;
    }
}
