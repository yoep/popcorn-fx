package com.github.yoep.video.vlc;

import com.github.yoep.video.vlc.config.VideoConfig;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Import;

@Configuration
@Import(VideoConfig.class)
public class AutoConfiguration {

}
