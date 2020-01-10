package com.github.yoep.popcorn.trakt.models;

import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.NoArgsConstructor;

@Data
@NoArgsConstructor
@AllArgsConstructor
public class TraktShow {
    private String title;
    private int year;
    private TraktShowIds ids;
}
