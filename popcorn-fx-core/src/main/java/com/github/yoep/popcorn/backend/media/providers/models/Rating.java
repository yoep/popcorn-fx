package com.github.yoep.popcorn.backend.media.providers.models;

import lombok.*;

import java.io.Serializable;

@Getter
@Builder
@AllArgsConstructor
@ToString
@EqualsAndHashCode
public class Rating implements Serializable {
    private final int percentage;
    private final int watching;
    private final int votes;
    private final int loved;
    private final int hated;
}
