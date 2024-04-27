package com.github.yoep.popcorn.backend.events;

import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.util.Optional;

@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
public class PlayerChangedEvent extends ApplicationEvent {
    private final String oldPlayerId;
    private final String newPlayerId;
    private final String newPlayerName;

    @Builder
    public PlayerChangedEvent(Object source, String oldPlayerId, String newPlayerId, String newPlayerName) {
        super(source);
        this.oldPlayerId = oldPlayerId;
        this.newPlayerId = newPlayerId;
        this.newPlayerName = newPlayerName;
    }

    public Optional<String> getOldPlayerId() {
        return Optional.ofNullable(oldPlayerId);
    }
}
