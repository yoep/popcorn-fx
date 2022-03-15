package com.github.yoep.popcorn.ui.messages;

import com.github.spring.boot.javafx.text.Message;
import lombok.Getter;

@Getter
public enum ListMessage implements Message {
    GENERIC("list_failed_generic"),
    RETRIEVAL_FAILED("list_failed_retrieving_data"),
    API_UNAVAILABLE("list_failed_api_unavailable");

    private final String key;

    ListMessage(String key) {
        this.key = key;
    }
}
