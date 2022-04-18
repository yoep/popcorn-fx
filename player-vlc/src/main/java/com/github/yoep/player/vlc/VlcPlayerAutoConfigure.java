package com.github.yoep.player.vlc;

import com.github.yoep.player.vlc.config.VlcDiscoveryConfig;
import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Import;

@Configuration
@Import({
        VlcDiscoveryConfig.class
})
@ComponentScan("com.github.yoep.player.vlc.services")
public class VlcPlayerAutoConfigure {
}
