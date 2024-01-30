package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.sun.jna.Structure;
import lombok.Getter;
import lombok.ToString;

@Getter
@ToString(callSuper = true)
@Structure.FieldOrder({"playerPlayCallback", "playerStopCallback"})
public class PlayerWrapperRegistration extends PlayerWrapper {
    public PlayerPlayCallback playerPlayCallback;
    public PlayerStopCallback playerStopCallback;

    public PlayerWrapperRegistration() {
    }

    public PlayerWrapperRegistration(Player player) {
        super(player);
        this.playerPlayCallback = request -> player.play(request.toPlayRequest());
        this.playerStopCallback = player::stop;
    }
}
