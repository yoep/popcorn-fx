package com.github.yoep.torrent.frostwire;

import com.github.yoep.torrent.frostwire.config.TorrentConfig;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Import;

@Configuration
@Import({
        TorrentConfig.class
})
public class AutoConfiguration {
}
