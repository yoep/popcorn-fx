package com.github.yoep.popcorn.backend.settings.models.subtitles;

import lombok.Getter;

@Getter
public enum SubtitleFamily {
    ARIAL("Arial"),
    COMIC_SANS("Comic Sans MS"),
    GEORGIA("Georgia"),
    TAHOMA("Tahoma"),
    TREBUCHET_MS("Trebuchet MS"),
    VERDANA("Verdana");

    private final String family;

    SubtitleFamily(String family) {
        this.family = family;
    }

    @Override
    public String toString() {
        return family;
    }
}
