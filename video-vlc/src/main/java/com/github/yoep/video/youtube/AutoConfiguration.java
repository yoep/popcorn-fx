package com.github.yoep.video.youtube;

import com.github.yoep.video.youtube.config.VideoConfig;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Import;

@Configuration
@Import(VideoConfig.class)
public class AutoConfiguration {

}
