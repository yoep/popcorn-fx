package com.github.yoep.player.vlc;

import com.github.yoep.popcorn.backend.utils.HostUtils;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;

import java.util.Random;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class VlcPlayerConstants {
    public static final String HOST = "localhost";
    public static final String PORT = String.valueOf(HostUtils.availablePort());
    public static final String PASSWORD = "popcorn-" + new Random().nextInt();
}
