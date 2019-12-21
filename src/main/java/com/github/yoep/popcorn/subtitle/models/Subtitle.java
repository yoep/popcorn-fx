package com.github.yoep.popcorn.subtitle.models;

import lombok.Data;

@Data
public class Subtitle {
    private final String language;
    private String url;
    private int score;
    private int downloads;

    public Subtitle(String language, String url) {
        this.language = language;
        this.url = url;
    }

    public Subtitle(String language, String url, int score, int downloads) {
        this.language = language;
        this.url = url;
        this.score = score;
        this.downloads = downloads;
    }
}
