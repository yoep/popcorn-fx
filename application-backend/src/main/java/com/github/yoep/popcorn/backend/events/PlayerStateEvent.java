package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

/**
 * Invoked when the player state has changed.
 */
@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
public class PlayerStateEvent extends ApplicationEvent {
    private final PlayerState newState;
    
    public PlayerStateEvent(Object source, PlayerState newState) {
        super(source);
        this.newState = newState;
    }
}
