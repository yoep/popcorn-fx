package com.github.yoep.popcorn.backend.subtitles.models;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;

@Data
@Builder
@AllArgsConstructor
public class SubtitleText {
    private final String text;
    private final boolean italic;
    private final boolean bold;
    private final boolean underline;
}
