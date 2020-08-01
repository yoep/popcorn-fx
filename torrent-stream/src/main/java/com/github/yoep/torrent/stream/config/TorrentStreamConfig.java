package com.github.yoep.torrent.stream.config;

import com.github.yoep.torrent.adapter.TorrentService;
import com.github.yoep.torrent.adapter.TorrentStreamService;
import com.github.yoep.torrent.stream.TorrentStreamServiceImpl;
import com.github.yoep.torrent.stream.controllers.VideoController;
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

    @Bean
    public VideoController videoController(TorrentStreamService streamService) {
        return new VideoController(streamService);
    }
}
