package com.github.yoep.popcorn.ui.subtitles.models;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;

import java.util.List;

@Data
@Builder
@AllArgsConstructor
public class SubtitleIndex implements Comparable<SubtitleIndex> {
    private final long index;
    private final long startTime;
    private final long endTime;
    private List<SubtitleLine> lines;

    @Override
    public int compareTo(SubtitleIndex o) {
        return Long.compare(getIndex(), o.getIndex());
    }
}
