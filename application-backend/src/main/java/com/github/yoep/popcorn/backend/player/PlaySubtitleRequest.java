package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.sun.jna.Structure;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.Optional;

@Getter
@ToString
@Structure.FieldOrder({"enabled", "subtitleInfo"})
public class PlaySubtitleRequest extends Structure implements Closeable {
    public static class ByValue extends PlaySubtitleRequest implements Structure.ByValue {
    }

    public byte enabled;
    public SubtitleInfo.ByReference subtitleInfo;

    @Override
    public void close() {
        setAutoSynch(false);
        Optional.ofNullable(subtitleInfo)
                .ifPresent(SubtitleInfo.ByReference::close);
    }
}
