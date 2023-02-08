package com.github.yoep.torrent.stream.config;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamService;
import com.github.yoep.torrent.stream.services.TorrentStreamServiceImpl;
import org.springframework.boot.autoconfigure.condition.ConditionalOnMissingBean;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class TorrentStreamConfig {
    @Bean
    @ConditionalOnMissingBean
    public TorrentStreamService streamService(TorrentService torrentService) {
        return new TorrentStreamServiceImpl(torrentService);
    }
}
