package com.github.yoep.player.chromecast;

import com.github.yoep.player.chromecast.config.ChromecastConfig;
import com.github.yoep.player.chromecast.config.TranscodeConfig;
import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Import;

@Configuration
@Import({
        ChromecastConfig.class,
        TranscodeConfig.class
})
@ComponentScan({
        "com.github.yoep.player.chromecast.controllers",
        "com.github.yoep.player.chromecast.services"
})
public class ChromecastAutoConfigure {
}
