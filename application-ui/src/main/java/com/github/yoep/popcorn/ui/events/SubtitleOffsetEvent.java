package com.github.yoep.popcorn.ui.events;

import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;
import org.springframework.context.ApplicationEvent;

@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
public class SubtitleOffsetEvent extends ApplicationEvent {
    /**
     * The subtitle offset to apply in seconds.
     */
    private final double offset;

    public SubtitleOffsetEvent(Object source, double offset) {
        super(source);
        this.offset = offset;
    }
}
