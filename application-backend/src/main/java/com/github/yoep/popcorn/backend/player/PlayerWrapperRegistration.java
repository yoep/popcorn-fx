package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.sun.jna.Structure;
import javafx.application.Platform;
import lombok.Getter;
import lombok.ToString;

@Getter
@ToString(callSuper = true)
@Structure.FieldOrder({"playerPlayCallback", "playerPauseCallback", "playerResumeCallback", "playerSeekCallback", "playerStopCallback"})
public class PlayerWrapperRegistration extends PlayerWrapper {
    public PlayerPlayCallback playerPlayCallback;
    public PlayerPauseCallback playerPauseCallback;
    public PlayerResumeCallback playerResumeCallback;
    public PlayerSeekCallback playerSeekCallback;
    public PlayerStopCallback playerStopCallback;

    public PlayerWrapperRegistration() {
    }

    public PlayerWrapperRegistration(Player player) {
        super(player);
        this.playerPlayCallback = onPlay(player);
        this.playerPauseCallback = onPause(player);
        this.playerResumeCallback = onResume(player);
        this.playerSeekCallback = onSeek(player);
        this.playerStopCallback = onStop(player);
    }

    private PlayerPlayCallback onPlay(Player player) {
        return request -> Platform.runLater(() -> player.play(request));
    }

    private PlayerPauseCallback onPause(Player player) {
        return player::pause;
    }

    private PlayerResumeCallback onResume(Player player) {
        return player::resume;
    }

    private PlayerSeekCallback onSeek(Player player) {
        return player::seek;
    }

    private PlayerStopCallback onStop(Player player) {
        return () -> Platform.runLater(player::stop);
    }
}
