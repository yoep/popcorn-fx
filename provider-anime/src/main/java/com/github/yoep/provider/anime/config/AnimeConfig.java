package com.github.yoep.provider.anime.config;

import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.Configuration;

@Configuration
@ComponentScan({
        "com.github.yoep.provider.anime.media",
        "com.github.yoep.provider.anime.imdb",
})
public class AnimeConfig {
}
