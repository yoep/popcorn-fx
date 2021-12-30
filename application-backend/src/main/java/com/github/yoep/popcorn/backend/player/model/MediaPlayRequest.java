package com.github.yoep.popcorn.backend.player.model;

import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.subtitles.models.SubtitleInfo;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.util.Optional;

@Getter
@ToString(callSuper = true)
@EqualsAndHashCode(callSuper = true)
public class MediaPlayRequest extends SimplePlayRequest {
    private final SubtitleInfo subtitle;
    private final String quality;
    private final Media media;

    @Builder(builderMethodName = "mediaBuilder")
    public MediaPlayRequest(String url, String title, String thumb, Long autoResumeTimestamp,
                            SubtitleInfo subtitle, String quality, Media media) {
        super(url, title, thumb, autoResumeTimestamp);
        this.subtitle = subtitle;
        this.quality = quality;
        this.media = media;
    }

    @Override
    public Optional<SubtitleInfo> getSubtitle() {
        return Optional.ofNullable(subtitle);
    }

    @Override
    public Optional<String> getQuality() {
        return Optional.ofNullable(quality);
    }

    @Override
    public boolean isSubtitlesEnabled() {
        return true;
    }
}
