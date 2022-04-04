package com.github.yoep.popcorn.backend.subtitles.model;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;

import java.nio.charset.Charset;

@Data
@Builder
@AllArgsConstructor
public class SubtitleFile implements Comparable<SubtitleFile> {
    private final Integer quality;
    private final String name;
    private final String url;
    private final int score;
    private final int downloads;
    private final Charset encoding;

    @Override
    public int compareTo(SubtitleFile compareTo) {
        if (score > compareTo.getScore() || (score == compareTo.getScore() && downloads > compareTo.getDownloads())) {
            return -1;
        } else if (score == compareTo.getScore() && downloads == compareTo.getDownloads()) {
            return 0;
        }

        return 1;
    }
}
