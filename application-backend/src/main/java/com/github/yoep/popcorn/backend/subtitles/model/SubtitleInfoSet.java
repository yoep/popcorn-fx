package com.github.yoep.popcorn.backend.subtitles.model;

import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.ToString;

import java.io.Closeable;
import java.util.List;

import static java.util.Arrays.asList;

@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"subtitles", "len"})
public class SubtitleInfoSet extends Structure implements Closeable {
    public SubtitleInfo.ByReference subtitles;
    public int len;

    public List<SubtitleInfo> getSubtitles() {
        return asList((SubtitleInfo[]) subtitles.toArray(len));
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
