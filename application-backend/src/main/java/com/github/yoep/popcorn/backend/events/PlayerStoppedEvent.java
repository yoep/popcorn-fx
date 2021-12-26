package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.media.providers.models.Media;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;
import org.springframework.context.ApplicationEvent;
import org.springframework.lang.Nullable;

import java.util.Optional;

/**
 * Event indicating that the video playback has been stopped.
 * This can either be caused by the user closing the player,
 * or the video has reached the end of it's playback.
 */
@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
public class PlayerStoppedEvent extends ApplicationEvent {
    /**
     * The unknown value for the {@link #getTime()} and {@link #getDuration()}.
     */
    public static final long UNKNOWN = -1;

    /**
     * The url that was being played in the player.
     */
    private final String url;
    /**
     * The media that was being played in the player.
     */
    @Nullable
    private final Media media;
    /**
     * The quality of the video that was being played in the player.
     */
    @Nullable
    private final String quality;
    /**
     * The last known time of the video player.
     */
    private final long time;
    /**
     * The duration of the playback in the video player.
     */
    private final long duration;

    @Builder
    public PlayerStoppedEvent(Object source, String url, @Nullable Media media, @Nullable String quality, long time, long duration) {
        super(source);
        this.url = url;
        this.media = media;
        this.quality = quality;
        this.time = time;
        this.duration = duration;
    }

    public Optional<Media> getMedia() {
        return Optional.ofNullable(media);
    }

    public Optional<String> getQuality() {
        return Optional.ofNullable(quality);
    }
}
