package com.github.yoep.popcorn.backend.subtitles.model;

import com.sun.jna.Structure;
import lombok.AllArgsConstructor;
import lombok.NoArgsConstructor;
import lombok.ToString;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

@NoArgsConstructor
@AllArgsConstructor
@ToString(exclude = {"cached"})
@Structure.FieldOrder({"textRef", "len", "cap"})
public class SubtitleLine extends Structure implements Closeable {
    public static class ByReference extends SubtitleLine implements Structure.ByReference {
    }

    public SubtitleText.ByReference textRef;
    public int len;
    public int cap;

    private List<SubtitleText> cached;

    public List<SubtitleText> texts() {
        if (cached == null) {
            cached = Optional.ofNullable(textRef)
                    .map(e -> e.toArray(len))
                    .map(e -> (SubtitleText[]) e)
                    .map(Arrays::asList)
                    .orElse(Collections.emptyList());
        }

        return cached;
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
