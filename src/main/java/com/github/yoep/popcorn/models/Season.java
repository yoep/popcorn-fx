package com.github.yoep.popcorn.models;

import lombok.AllArgsConstructor;
import lombok.Data;

@Data
@AllArgsConstructor
public class Season {
    private final int season;
    private final String text;

    @Override
    public String toString() {
        return text;
    }
}
