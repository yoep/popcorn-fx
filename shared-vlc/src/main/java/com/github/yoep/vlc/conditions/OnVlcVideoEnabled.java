package com.github.yoep.vlc.conditions;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;

@Slf4j
@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class OnVlcVideoEnabled {
    public static boolean matches(FxLib fxLib, PopcornFx instance) {
        return fxLib.is_vlc_video_player_disabled(instance) == 0;
    }
}
