package com.github.yoep.popcorn.ui.messages;

import com.github.spring.boot.javafx.text.Message;
import lombok.Getter;

@Getter
public enum TraktMessage implements Message {
    TOKEN_EXPIRED("trakt_token_expired"),
    AUTHENTICATION_FAILED("trakt_authentication_failed"),
    AUTHENTICATION_SUCCESS("trakt_authentication_successful"),
    SYNCHRONIZATION_FAILED("trakt_synchronization_failed");

    private final String key;

    TraktMessage(String key) {
        this.key = key;
    }
}
