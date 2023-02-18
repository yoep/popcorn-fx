package com.github.yoep.popcorn.backend.events;

import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;
import org.springframework.context.ApplicationEvent;

/**
 * Event indicating that the player is being closed.
 */
@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
public class ClosePlayerEvent extends ApplicationEvent {
    public enum Reason {
        /**
         * The user has closed the player.
         */
        USER,
        /**
         * The end of the video has been reached.
         */
        END_OF_VIDEO
    }

    private final Reason reason;

    public ClosePlayerEvent(Object source, Reason reason) {
        super(source);
        this.reason = reason;
    }
}
