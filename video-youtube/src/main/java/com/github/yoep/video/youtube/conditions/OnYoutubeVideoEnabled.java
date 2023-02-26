package com.github.yoep.video.youtube.conditions;

import lombok.AccessLevel;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.ApplicationArguments;

@Slf4j
@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class OnYoutubeVideoEnabled {
    static final String DISABLE_OPTION = "disable-youtube-video-player";

    public static boolean matches(ApplicationArguments arguments) {
        log.trace("The application started with \"{}\" options", arguments.getOptionNames());
        return !arguments.containsOption(DISABLE_OPTION);
    }
}
