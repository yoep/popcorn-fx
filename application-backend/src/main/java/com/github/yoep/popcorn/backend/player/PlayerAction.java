package com.github.yoep.popcorn.backend.player;

import javafx.scene.input.KeyCode;
import lombok.Getter;

import java.util.Arrays;
import java.util.Objects;
import java.util.Optional;

@Getter
public enum PlayerAction {
    TOGGLE_PLAYBACK_STATE(KeyCode.P, KeyCode.SPACE),
    REVERSE(KeyCode.LEFT, KeyCode.KP_LEFT),
    FORWARD(KeyCode.RIGHT, KeyCode.KP_RIGHT),
    TOGGLE_FULLSCREEN(KeyCode.F11),
    INCREASE_SUBTITLE_OFFSET(KeyCode.G),
    DECREASE_SUBTITLE_OFFSET(KeyCode.H);

    private final KeyCode[] keys;

    PlayerAction(KeyCode... keys) {
        this.keys = keys;
    }

    public static Optional<PlayerAction> FromKey(KeyCode code) {
        Objects.requireNonNull(code, "code cannot be null");
        return Arrays.stream(PlayerAction.values())
                .filter(e -> containsKeyCode(e, code))
                .findFirst();
    }

    private static boolean containsKeyCode(PlayerAction action, KeyCode code) {
        return Arrays.stream(action.getKeys())
                .anyMatch(key -> key.getCode() == code.getCode());
    }
}
