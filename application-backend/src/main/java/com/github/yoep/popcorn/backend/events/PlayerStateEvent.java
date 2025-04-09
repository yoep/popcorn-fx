package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
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
    private final Player.State newState;
    
    public PlayerStateEvent(Object source, Player.State newState) {
        super(source);
        this.newState = newState;
    }
}
