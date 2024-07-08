package com.github.yoep.popcorn.backend.subtitles.model;

import com.sun.jna.ptr.IntByReference;
import lombok.Builder;

import java.util.Optional;

@Builder
public record SubtitleFile(int fileId, String name, String url, int score, int downloads, Integer quality) {
    public static SubtitleFile from(com.github.yoep.popcorn.backend.subtitles.ffi.SubtitleFile file) {
        return SubtitleFile.builder()
                .fileId(file.fileId)
                .name(file.name)
                .url(file.url)
                .score(file.score)
                .downloads(file.downloads)
                .quality(Optional.ofNullable(file.quality)
                        .map(IntByReference::getValue)
                        .orElse(null))
                .build();
    }
}
