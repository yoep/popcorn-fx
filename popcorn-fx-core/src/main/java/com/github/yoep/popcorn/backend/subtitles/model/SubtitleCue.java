package com.github.yoep.popcorn.backend.subtitles.model;

import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.*;

@Getter
@ToString(exclude = {"cached"})
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"id", "startTime", "endTime", "lineRef", "len"})
public class SubtitleCue extends Structure implements Comparable<SubtitleCue>, Closeable {
    public static class ByReference extends SubtitleCue implements Structure.ByReference {
    }

    /**
     * The unique ID of the subtitle cue.
     */
    public String id;
    /**
     * The start time of the cue in millis.
     */
    public long startTime;
    /**
     * The end time of the cue in millis.
     */
    public long endTime;

    public SubtitleLine.ByReference lineRef;
    public int len;

    private List<SubtitleLine> cached;

    public SubtitleCue() {
    }

    public long getStartTime() {
        return startTime;
    }

    public long getEndTime() {
        return endTime;
    }

    public List<SubtitleLine> getLines() {
        if (cached == null) {
            cached = Optional.ofNullable(lineRef)
                    .map(e -> e.toArray(len))
                    .map(e -> (SubtitleLine[]) e)
                    .map(Arrays::asList)
                    .orElse(Collections.emptyList());
        }

        return cached;
    }

    @Override
    public int compareTo(SubtitleCue o) {
        return Objects.compare(getStartTime(), o.getStartTime(), Long::compareTo);
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
