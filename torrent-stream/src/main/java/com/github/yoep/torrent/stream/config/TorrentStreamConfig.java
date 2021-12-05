package com.github.yoep.torrent.stream.config;

import com.github.yoep.torrent.adapter.TorrentService;
import com.github.yoep.torrent.adapter.TorrentStreamService;
import com.github.yoep.torrent.stream.services.TorrentStreamServiceImpl;
import com.github.yoep.torrent.stream.web.ControllerHandler;
import org.springframework.boot.autoconfigure.condition.ConditionalOnMissingBean;
import org.springframework.boot.autoconfigure.web.ServerProperties;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class TorrentStreamConfig {
    @Bean
    @ConditionalOnMissingBean
    public TorrentStreamService streamService(TorrentService torrentService,
                                              ServerProperties serverProperties) {
        return new TorrentStreamServiceImpl(torrentService, serverProperties);
    }

    @Bean
    public ControllerHandler controllerHandler() {
        return new ControllerHandler();
    }
}
