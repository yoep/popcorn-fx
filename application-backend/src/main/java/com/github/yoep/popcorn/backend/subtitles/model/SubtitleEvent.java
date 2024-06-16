package com.github.yoep.popcorn.backend.subtitles.model;

import lombok.Builder;

import java.util.Objects;
import java.util.Optional;

@Builder
public record SubtitleEvent(SubtitleEventTag tag, SubtitleInfo subtitleInfo) {
    public Optional<SubtitleInfo> getSubtitleInfo() {
        return Optional.ofNullable(subtitleInfo);
    }

    public static SubtitleEvent from(com.github.yoep.popcorn.backend.subtitles.ffi.SubtitleEvent event) {
        Objects.requireNonNull(event, "event cannot be null");
        var tag = event.getTag();
        SubtitleInfo info = null;

        if (tag == SubtitleEventTag.SubtitleInfoChanged) {
            info = SubtitleInfo.from(event.getUnion().getSubtitle_info_changed().getSubtitleInfo());
        }

        return new SubtitleEvent(
                tag,
                info
        );
    }
}
