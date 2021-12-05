package com.github.yoep.player.chromecast;

import com.github.yoep.player.chromecast.config.ChromecastConfig;
import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Import;

@Configuration
@Import({
        ChromecastConfig.class
})
@ComponentScan("com.github.yoep.player.chromecast.services")
public class AutoConfigure {
}
