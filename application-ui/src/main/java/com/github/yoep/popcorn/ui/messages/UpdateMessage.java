package com.github.yoep.popcorn.ui.messages;

import com.github.spring.boot.javafx.text.Message;
import lombok.Getter;

@Getter
public enum UpdateMessage implements Message {
    CHECK_FOR_NEW_UPDATES("update_check"),
    CHECKING_FOR_UPDATES("update_checking"),
    NO_UPDATE_AVAILABLE("no_update_available"),
    DOWNLOAD_UPDATE("download_update"),
    NEW_VERSION("new_version"),
    STARTING_DOWNLOAD("update_state_starting_download"),
    DOWNLOADING("update_state_downloading"),
    DOWNLOAD_FINISHED("update_state_download_finished"),
    INSTALLING("update_state_installing"),
    ERROR("update_state_error");

    private final String key;

    UpdateMessage(String key) {
        this.key = key;
    }
}
