package com.github.yoep.video.vlc;

import com.github.yoep.video.vlc.config.VideoConfig;
import com.github.yoep.video.vlc.config.VlcConfig;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Import;

@Configuration
@Import({
        VlcConfig.class,
        VideoConfig.class
})
public class AutoConfiguration {
}
