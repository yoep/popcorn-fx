package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.sun.jna.Structure;
import lombok.Getter;
import lombok.ToString;

@Getter
@ToString (callSuper = true)
@Structure.FieldOrder({"playCallback"})
public class PlayerWrapperRegistration extends PlayerWrapper {
    public PlayCallback playCallback;

    public PlayerWrapperRegistration() {
    }

    public PlayerWrapperRegistration(Player player) {
        super(player);
        this.playCallback = request -> player.play(request.toPlayRequest());
    }
}
