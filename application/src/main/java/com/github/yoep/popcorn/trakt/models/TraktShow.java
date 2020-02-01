package com.github.yoep.popcorn.trakt.models;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;

@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class TraktShow {
    private String title;
    private int year;
    private TraktShowIds ids;
}
