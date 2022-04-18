package com.github.yoep.player.vlc.mappers;

import com.github.yoep.player.vlc.model.VlcState;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class StateMapper {
    public static PlayerState map(VlcState state) {
        switch (state) {
            case PLAYING -> {
                return PlayerState.PLAYING;
            }
            case PAUSED -> {
                return PlayerState.PAUSED;
            }
            case STOPPED -> {
                return PlayerState.STOPPED;
            }
            default -> {
                return PlayerState.UNKNOWN;
            }
        }
    }
}
