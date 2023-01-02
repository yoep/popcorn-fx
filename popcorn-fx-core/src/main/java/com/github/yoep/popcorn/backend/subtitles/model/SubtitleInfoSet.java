package com.github.yoep.popcorn.backend.subtitles.model;

import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.ToString;

import java.io.Closeable;
import java.util.List;

import static java.util.Arrays.asList;

@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"subtitles", "numberOfSubtitles", "capacity"})
public class SubtitleInfoSet extends Structure implements Closeable {
    public static class ByReference extends SubtitleInfoSet implements Structure.ByReference {
        public ByReference() {
        }
    }

    public static class ByValue extends SubtitleInfoSet implements Structure.ByValue {
    }

    public SubtitleInfo.ByReference subtitles;
    public int numberOfSubtitles;
    public int capacity;

    public SubtitleInfoSet() {
    }

    public List<SubtitleInfo> getSubtitles() {
        return asList((SubtitleInfo[]) subtitles.toArray(numberOfSubtitles));
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
