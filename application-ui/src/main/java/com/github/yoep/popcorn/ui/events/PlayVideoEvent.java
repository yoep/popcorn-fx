package com.github.yoep.popcorn.ui.events;

import lombok.EqualsAndHashCode;
import lombok.Getter;
import org.springframework.context.ApplicationEvent;
import org.springframework.util.Assert;

@Getter
@EqualsAndHashCode(callSuper = false)
public class PlayVideoEvent extends ApplicationEvent {
    /**
     * The url of the video to play.
     */
    private final String url;
    /**
     * The title of the video.
     */
    private final String title;
    /**
     * Check if the subtitles should be enabled for this video playback.
     * If true, the player will show the subtitle UI section, otherwise it will be hidden.
     */
    private final boolean subtitlesEnabled;

    public PlayVideoEvent(Object source, String url, String title, boolean subtitlesEnabled) {
        super(source);
        Assert.notNull(url, "url cannot be null");
        Assert.notNull(title, "title cannot be null");
        this.url = url;
        this.title = title;
        this.subtitlesEnabled = subtitlesEnabled;
    }
}
