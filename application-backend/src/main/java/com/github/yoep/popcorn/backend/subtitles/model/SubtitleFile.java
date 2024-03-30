package com.github.yoep.popcorn.backend.subtitles.model;

import com.sun.jna.Structure;
import com.sun.jna.ptr.IntByReference;
import lombok.*;

import java.io.Closeable;

@Getter
@ToString
@NoArgsConstructor
@AllArgsConstructor
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"fileId", "name", "url", "score", "downloads", "quality"})
public class SubtitleFile extends Structure implements Closeable {
    public static class ByReference extends SubtitleFile implements Structure.ByReference {
        public ByReference() {
        }

        public ByReference(int fileId, String name, String url, int score, int downloads, IntByReference quality) {
            super(fileId, name, url, score, downloads, quality);
        }
    }

    public int fileId;
    public String name;
    public String url;
    public int score;
    public int downloads;
    public IntByReference quality;

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
