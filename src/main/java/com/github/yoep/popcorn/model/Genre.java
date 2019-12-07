package com.github.yoep.popcorn.model;

import lombok.Data;

@Data
public class Genre {
    private String key;
    private String text;

    public Genre(String key, String text) {
        this.key = key;
        this.text = text;
    }

    @Override
    public String toString() {
        return text;
    }
}
