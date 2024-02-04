package com.github.yoep.popcorn.backend.events;

import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;
import org.springframework.context.ApplicationEvent;

@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
public class PlayerStartedEvent extends ApplicationEvent {
    private final String url;
    private final String title;
    private final String thumbnail;
    private final String quality;
    private final Long autoResumeTimestamp;
    private final boolean subtitleEnabled;

    @Builder
    public PlayerStartedEvent(Object source, String url, String title, String thumbnail, String quality, Long autoResumeTimestamp, boolean subtitleEnabled) {
        super(source);
        this.url = url;
        this.title = title;
        this.thumbnail = thumbnail;
        this.quality = quality;
        this.autoResumeTimestamp = autoResumeTimestamp;
        this.subtitleEnabled = subtitleEnabled;
    }
}
