package com.github.yoep.popcorn.models;

import lombok.Getter;

@Getter
public class SortBy {
    private final String key;
    private final String text;

    public SortBy(String key, String text) {
        this.key = key;
        this.text = text;
    }

    @Override
    public String toString() {
        return text;
    }
}
