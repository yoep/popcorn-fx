package com.github.yoep.vlc;

import com.github.yoep.vlc.config.VlcConfig;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Import;

@Configuration
@Import({
        VlcConfig.class
})
public class VlcAutoConfiguration {
}
