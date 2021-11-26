package com.github.yoep.player.popcorn.config;

import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.Configuration;

@Configuration
@ComponentScan({
        "com.github.yoep.player.popcorn.controllers",
        "com.github.yoep.player.popcorn.services",
        "com.github.yoep.player.popcorn.player"
})
public class PopcornConfig {
}
