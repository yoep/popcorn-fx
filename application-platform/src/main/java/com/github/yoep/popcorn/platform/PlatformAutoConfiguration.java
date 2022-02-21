package com.github.yoep.popcorn.platform;

import com.github.yoep.popcorn.platform.config.PlatformConfig;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Import;

@Configuration
@Import({
        PlatformConfig.class
})
public class PlatformAutoConfiguration {
}
