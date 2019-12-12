package com.github.yoep.popcorn.models;

import lombok.Getter;

@Getter
public class Genre implements Comparable<Genre> {
    private static final String ALL_KEYWORD = "all";

    private final String key;
    private final String text;

    public Genre(String key, String text) {
        this.key = key;
        this.text = text;
    }

    @Override
    public String toString() {
        return text;
    }

    @Override
    public int compareTo(Genre o) {
        // make sure that the "all" key is always on top
        if (this.key.equalsIgnoreCase(ALL_KEYWORD))
            return -1;
        if (o.getKey().equalsIgnoreCase(ALL_KEYWORD))
            return 1;

        return this.getText().compareTo(o.getText());
    }
}
