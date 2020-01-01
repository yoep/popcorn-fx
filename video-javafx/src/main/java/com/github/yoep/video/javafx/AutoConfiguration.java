package com.github.yoep.video.javafx;

import com.github.yoep.video.javafx.config.VideoConfig;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Import;

@Configuration
@Import(VideoConfig.class)
public class AutoConfiguration {

}
