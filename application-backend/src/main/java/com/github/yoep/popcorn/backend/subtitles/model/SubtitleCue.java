package com.github.yoep.popcorn.backend.subtitles.model;

import lombok.*;

import java.util.List;
import java.util.Objects;

@Getter
@Builder
@ToString
@EqualsAndHashCode
@AllArgsConstructor
public class SubtitleCue implements Comparable<SubtitleCue> {
    /**
     * The unique ID of the subtitle cue.
     */
    private final String id;
    /**
     * The start time of the cue in millis.
     */
    private final long startTime;
    /**
     * The end time of the cue in millis.
     */
    private final long endTime;
    /**
     * The lines of the cue.
     */
    private final List<SubtitleLine> lines;

    @Override
    public int compareTo(SubtitleCue o) {
        return Objects.compare(getStartTime(), o.getStartTime(), Long::compareTo);
    }
}
