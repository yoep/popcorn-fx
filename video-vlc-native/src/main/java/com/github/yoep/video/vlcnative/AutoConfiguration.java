package com.github.yoep.video.vlcnative;

import com.github.yoep.video.vlcnative.config.VlcNativeConfig;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Import;

@Configuration
@Import({
        VlcNativeConfig.class
})
public class AutoConfiguration {
}
