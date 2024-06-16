package com.github.yoep.popcorn.backend.subtitles.ffi;

import com.sun.jna.Structure;
import com.sun.jna.ptr.IntByReference;
import lombok.*;

import java.io.Closeable;
import java.util.Optional;

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

        @Builder
        public ByReference(int fileId, String name, String url, Integer score, Integer downloads, IntByReference quality) {
            super(fileId, name, url, score, downloads, quality);
        }

        public static SubtitleFile.ByReference from(com.github.yoep.popcorn.backend.subtitles.model.SubtitleFile file) {
            return SubtitleFile.ByReference.builder()
                    .fileId(file.fileId())
                    .name(file.name())
                    .url(file.url())
                    .score(file.score())
                    .downloads(file.downloads())
                    .quality(Optional.ofNullable(file.quality())
                            .map(IntByReference::new)
                            .orElse(null))
                    .build();
        }
    }

    public int fileId;
    public String name;
    public String url;
    public Integer score;
    public Integer downloads;
    public IntByReference quality;

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
