package com.github.yoep.torrent.stream;

import com.github.yoep.torrent.stream.config.TorrentStreamConfig;
import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Import;

@Configuration
@Import({
        TorrentStreamConfig.class
})
@ComponentScan({
        "com.github.yoep.torrent.stream.controllers",
        "com.github.yoep.torrent.stream.services"
})
public class AutoConfiguration {
}
