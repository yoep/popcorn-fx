package com.github.yoep.popcorn.backend.adapters.player;

import lombok.Getter;

import java.text.MessageFormat;

/**
 * Defines that the player which is being registered already exists.
 */
@Getter
public class PlayerAlreadyExistsException extends PlayerException {
    private final String id;

    public PlayerAlreadyExistsException(String id) {
        super(MessageFormat.format("The player with ID \"{0}\" already exists", id));
        this.id = id;
    }
}
