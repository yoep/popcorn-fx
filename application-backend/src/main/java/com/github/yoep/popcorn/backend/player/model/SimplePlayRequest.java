package com.github.yoep.popcorn.backend.player.model;

import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import lombok.*;

import java.util.Optional;

@Getter
@Builder
@AllArgsConstructor
@ToString
@EqualsAndHashCode
public class SimplePlayRequest implements PlayRequest {
    private final String url;
    private final String title;
    private final String thumb;
    private final Long autoResumeTimestamp;
    private final boolean subtitlesEnabled;

    @Override
    public Optional<String> getTitle() {
        return Optional.ofNullable(title);
    }

    @Override
    public Optional<String> getThumbnail() {
        return Optional.ofNullable(thumb);
    }

    @Override
    public Optional<String> getQuality() {
        return Optional.empty();
    }

    @Override
    public Optional<Long> getAutoResumeTimestamp() {
        return Optional.ofNullable(autoResumeTimestamp);
    }

    @Override
    public boolean isSubtitlesEnabled() {
        return subtitlesEnabled;
    }
}
