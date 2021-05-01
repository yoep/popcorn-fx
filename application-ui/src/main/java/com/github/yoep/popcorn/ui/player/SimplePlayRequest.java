package com.github.yoep.popcorn.ui.player;

import com.github.yoep.player.adapter.PlayRequest;
import com.github.yoep.player.adapter.subtitles.Subtitle;
import lombok.*;

import java.util.Collection;
import java.util.List;
import java.util.Optional;

@Getter
@Builder
@AllArgsConstructor
@ToString
@EqualsAndHashCode
public class SimplePlayRequest implements PlayRequest {
    private final String url;
    private final String title;
    private final Subtitle subtitle;
    private final String quality;
    private final List<Subtitle> subtitles;

    @Override
    public Optional<String> getTitle() {
        return Optional.ofNullable(title);
    }

    @Override
    public Optional<Subtitle> getSubtitle() {
        return Optional.ofNullable(subtitle);
    }

    @Override
    public Optional<String> getQuality() {
        return Optional.ofNullable(quality);
    }

    @Override
    public Collection<Subtitle> subtitles() {
        return subtitles;
    }
}
