package com.github.yoep.popcorn.backend.subtitles.model;

import com.github.yoep.popcorn.backend.FxLib;
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
    public static class ByReference extends  SubtitleInfoSet implements Structure.ByReference {
        @Override
        public void close() {
            super.close();
            FxLib.INSTANCE.get().dispose_subtitle_info_set(this);
        }
    }

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
