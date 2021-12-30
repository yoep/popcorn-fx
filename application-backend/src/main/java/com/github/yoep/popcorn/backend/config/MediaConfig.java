package com.github.yoep.popcorn.backend.config;

import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.Configuration;

@Configuration
@ComponentScan({
        "com.github.yoep.popcorn.backend.media.favorites",
        "com.github.yoep.popcorn.backend.media.providers",
        "com.github.yoep.popcorn.backend.media.resume",
        "com.github.yoep.popcorn.backend.media.watched",
})
public class MediaConfig {
}
