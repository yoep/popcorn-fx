package com.github.yoep.popcorn.ui.messages;

import com.github.yoep.popcorn.backend.utils.Message;
import lombok.Getter;

@Getter
public enum SettingsMessage implements Message {
    AUTHORIZE("settings_trakt_connect"),
    DISCONNECT("settings_trakt_disconnect"),
    LAST_SYNC_STATE("settings_trakt_last_sync_state"),
    LAST_SYNC_TIME("settings_trakt_last_sync_time"),
    SETTINGS_FAILED_TO_LOAD("settings_failed_to_load"),
    SETTINGS_SAVED("settings_saved"),
    SYNC_FAILED("settings_trakt_sync_failed"),
    SYNC_SUCCESS("settings_trakt_sync_success"),
    TRAKT_LOGIN_TITLE("settings_trakt_login_title");

    private final String key;

    SettingsMessage(String key) {
        this.key = key;
    }
}
