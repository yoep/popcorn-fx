package com.github.yoep.player.popcorn;

import com.github.yoep.player.popcorn.config.PopcornConfig;
import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Import;

@Configuration
@Import({
        PopcornConfig.class
})
@ComponentScan("com.github.yoep.player.popcorn.services")
public class AutoConfigure {
}
