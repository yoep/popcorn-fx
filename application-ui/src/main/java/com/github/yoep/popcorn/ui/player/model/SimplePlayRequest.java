package com.github.yoep.popcorn.ui.player.model;

import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.player.subtitles.Subtitle;
import lombok.*;

import java.util.Collection;
import java.util.Collections;
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

    @Override
    public Optional<String> getTitle() {
        return Optional.ofNullable(title);
    }

    @Override
    public Optional<Subtitle> getSubtitle() {
        return Optional.empty();
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
    public Collection<Subtitle> subtitles() {
        return Collections.emptyList();
    }
}
