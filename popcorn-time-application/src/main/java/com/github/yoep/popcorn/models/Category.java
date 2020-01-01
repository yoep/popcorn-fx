package com.github.yoep.popcorn.models;

import lombok.Getter;

@Getter
public enum Category {
    MOVIES("movies"),
    SERIES("series"),
    FAVORITES("favorites");

    /**
     * The provider name for this category.
     */
    private final String providerName;

    Category(String providerName) {
        this.providerName = providerName;
    }
}
