package com.github.yoep.torrent.frostwire.config;

import com.github.yoep.torrent.adapter.TorrentService;
import com.github.yoep.torrent.adapter.TorrentSettingsService;
import com.github.yoep.torrent.frostwire.*;
import org.apache.http.client.HttpClient;
import org.apache.http.impl.client.DefaultRedirectStrategy;
import org.apache.http.impl.client.HttpClientBuilder;
import org.springframework.boot.autoconfigure.condition.ConditionalOnMissingBean;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.task.TaskExecutor;

@Configuration
public class TorrentConfig {
    @Bean
    public TorrentSessionManager sessionManager(TaskExecutor taskExecutor) {
        return new TorrentSessionManagerImpl(taskExecutor);
    }

    @Bean
    public HttpClient httpClient() {
        return HttpClientBuilder.create()
                .setRedirectStrategy(new DefaultRedirectStrategy())
                .build();
    }

    @Bean
    public TorrentResolverService torrentResolverService(TorrentSessionManager sessionManager,
                                                         HttpClient httpClient) {
        return new TorrentResolverService(sessionManager, httpClient);
    }

    @Bean
    @ConditionalOnMissingBean(TorrentService.class)
    public TorrentService torrentService(TorrentSessionManager sessionManager,
                                         TorrentResolverService torrentResolverService) {
        return new TorrentServiceImpl(sessionManager, torrentResolverService);
    }

    @Bean
    @ConditionalOnMissingBean(TorrentSettingsService.class)
    public TorrentSettingsService torrentSettingsService(TorrentSessionManager sessionManager) {
        return new TorrentSettingsServiceImpl(sessionManager);
    }
}
