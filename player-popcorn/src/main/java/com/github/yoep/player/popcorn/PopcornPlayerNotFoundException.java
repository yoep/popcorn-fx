package com.github.yoep.player.popcorn;

public class PopcornPlayerNotFoundException extends RuntimeException {
    public PopcornPlayerNotFoundException() {
        super("The popcorn player couldn't be found back within the registered players");
    }
}
