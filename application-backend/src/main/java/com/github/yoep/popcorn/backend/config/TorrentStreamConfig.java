package com.github.yoep.popcorn.backend.config;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamService;
import com.github.yoep.popcorn.backend.torrent.TorrentStreamServiceImpl;
import org.springframework.boot.autoconfigure.condition.ConditionalOnMissingBean;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class TorrentStreamConfig {
    @Bean
    @ConditionalOnMissingBean
    public TorrentStreamService streamService(FxLib fxLib, TorrentService torrentService) {
        return new TorrentStreamServiceImpl(fxLib, PopcornFxInstance.INSTANCE, torrentService);
    }
}
