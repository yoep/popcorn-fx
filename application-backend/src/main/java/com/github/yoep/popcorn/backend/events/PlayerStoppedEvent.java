package com.github.yoep.popcorn.backend.events;

import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

/**
 * Event indicating that the video playback has been stopped.
 * This can either be caused by the user closing the player,
 * or the video has reached the end of it's playback.
 */
@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
public class PlayerStoppedEvent extends ApplicationEvent {
    public PlayerStoppedEvent(Object source) {
        super(source);
    }
}
