package com.github.yoep.player.qt;

import com.github.yoep.player.qt.config.QtPlayerConfig;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Import;

@Configuration
@Import({
        QtPlayerConfig.class
})
public class AutoConfiguration {
}
