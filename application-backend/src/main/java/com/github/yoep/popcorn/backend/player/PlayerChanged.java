package com.github.yoep.popcorn.backend.player;

public record PlayerChanged(String oldPlayerId, String newPlayerId, String newPlayerName) {
}
