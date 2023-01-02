package com.github.yoep.popcorn.backend.config;

import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.config.properties.SubtitleProperties;
import de.timroes.axmlrpc.XMLRPCClient;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.Configuration;

import java.net.MalformedURLException;

@Slf4j
@Configuration
@ComponentScan("com.github.yoep.popcorn.backend.subtitles")
public class SubtitlesConfig {
    @Bean
    public XMLRPCClient xmlrpcClient(PopcornProperties properties) {
        SubtitleProperties subtitleProperties = properties.getSubtitle();

        try {
            return new XMLRPCClient(subtitleProperties.getUrl().toURL(), subtitleProperties.getUserAgent());
        } catch (MalformedURLException ex) {
            log.error(ex.getMessage(), ex);
            return null;
        }
    }
}
