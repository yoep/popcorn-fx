package com.github.yoep.popcorn.messages;

import com.github.spring.boot.javafx.text.Message;
import lombok.Getter;

@Getter
public enum DetailsMessage implements Message {
    MAGNET_LINK("details_magnet_link"),
    HEALTH_UNKNOWN("health_unknown");

    private final String key;

    DetailsMessage(String key) {
        this.key = key;
    }
}
