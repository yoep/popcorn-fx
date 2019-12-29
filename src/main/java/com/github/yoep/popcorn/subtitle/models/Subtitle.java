package com.github.yoep.popcorn.subtitle.models;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;

import java.util.List;

@Data
@Builder
@AllArgsConstructor
public class Subtitle implements Comparable<Subtitle> {
    private final long index;
    private final long startTime;
    private final long endTime;
    private List<SubtitleLine> lines;

    @Override
    public int compareTo(Subtitle o) {
        return Long.compare(getIndex(), o.getIndex());
    }
}
