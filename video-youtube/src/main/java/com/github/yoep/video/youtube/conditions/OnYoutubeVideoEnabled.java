package com.github.yoep.video.youtube.conditions;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;

@Slf4j
@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class OnYoutubeVideoEnabled {
    public static boolean matches(FxLib fxLib, PopcornFx instance) {
        return fxLib.is_youtube_video_player_disabled(instance) == 0;
    }
}
