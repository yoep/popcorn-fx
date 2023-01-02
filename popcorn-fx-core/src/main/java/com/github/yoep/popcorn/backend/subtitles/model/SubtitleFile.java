package com.github.yoep.popcorn.backend.subtitles.model;

import lombok.*;

import java.nio.charset.Charset;

@Getter
@Builder
@ToString
@AllArgsConstructor
@EqualsAndHashCode(callSuper = false)
public class SubtitleFile implements Comparable<SubtitleFile> {
    private int fileId;
    private String name;
    private String url;
    private Integer quality;
    private int score;
    private int downloads;
    private Charset encoding;

    public SubtitleFile() {
    }

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
