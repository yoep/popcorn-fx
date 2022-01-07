package com.github.yoep.popcorn.backend.media.filters.models;

import lombok.Getter;

@Getter
public enum Category {
    MOVIES("movies"),
    SERIES("series"),
    ANIME("animes"),
    FAVORITES("favorites");

    /**
     * The provider name for this category.
     */
    private final String providerName;

    Category(String providerName) {
        this.providerName = providerName;
    }
}
