package com.github.yoep.popcorn.ui.subtitles.config;

import com.github.yoep.popcorn.ui.config.properties.PopcornProperties;
import com.github.yoep.popcorn.ui.config.properties.SubtitleProperties;
import de.timroes.axmlrpc.XMLRPCClient;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

import java.net.MalformedURLException;

@Slf4j
@Configuration
public class XmlRpcConfig {
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
