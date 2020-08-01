package com.github.yoep.torrent.stream;

import com.github.yoep.torrent.stream.config.TorrentStreamConfig;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Import;

@Configuration
@Import({
        TorrentStreamConfig.class
})
public class AutoConfiguration {
}
