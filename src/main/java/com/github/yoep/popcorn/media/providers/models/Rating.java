package com.github.yoep.popcorn.media.providers.models;

import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.NoArgsConstructor;

@Data
@NoArgsConstructor
@AllArgsConstructor
public class Rating {
    private int percentage;
    private int watching;
    private int votes;
    private int loved;
    private int hated;
}
