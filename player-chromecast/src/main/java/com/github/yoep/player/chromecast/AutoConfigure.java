package com.github.yoep.player.chromecast;

import com.github.yoep.player.chromecast.config.ChromecastConfig;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Import;

@Configuration
@Import({
        ChromecastConfig.class
})
public class AutoConfigure {
}
