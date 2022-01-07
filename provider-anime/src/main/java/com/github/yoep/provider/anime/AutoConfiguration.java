package com.github.yoep.provider.anime;

import com.github.yoep.provider.anime.config.AnimeConfig;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Import;

@Configuration
@Import({
        AnimeConfig.class
})
public class AutoConfiguration {
}
