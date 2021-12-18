package com.github.yoep.popcorn.ui.player.model;

import com.github.yoep.popcorn.backend.adapters.player.subtitles.Subtitle;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.util.Collection;
import java.util.List;
import java.util.Optional;

@Getter
@ToString(callSuper = true)
@EqualsAndHashCode(callSuper = true)
public class MediaPlayRequest extends SimplePlayRequest {
    private final Subtitle subtitle;
    private final String quality;
    private final List<Subtitle> subtitles;

    @Builder(builderMethodName = "mediaBuilder")
    public MediaPlayRequest(String url, String title, String thumb, Subtitle subtitle, String quality, List<Subtitle> subtitles) {
        super(url, title, thumb);
        this.subtitle = subtitle;
        this.quality = quality;
        this.subtitles = subtitles;
    }

    @Override
    public Optional<Subtitle> getSubtitle() {
        return Optional.ofNullable(subtitle);
    }

    @Override
    public Collection<Subtitle> subtitles() {
        return subtitles;
    }

    @Override
    public Optional<String> getQuality() {
        return Optional.ofNullable(quality);
    }
}
