package com.github.yoep.popcorn.messages;

import com.github.spring.boot.javafx.text.Message;
import lombok.Getter;

@Getter
public enum WatchlistMessage implements Message {
    FAILED_TO_PARSE_MOVIE("watchlist_failed_to_parse_movie");

    private final String key;

    WatchlistMessage(String key) {
        this.key = key;
    }
}
