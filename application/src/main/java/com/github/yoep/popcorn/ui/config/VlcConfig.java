package com.github.yoep.popcorn.ui.config;

import com.github.yoep.player.vlc.VlcPlayerConstants;
import com.github.yoep.player.vlc.discovery.VlcDiscovery;
import com.github.yoep.player.vlc.services.VlcPlayerService;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.vlc.conditions.OnVlcVideoEnabled;
import com.github.yoep.vlc.discovery.LinuxNativeDiscoveryStrategy;
import com.github.yoep.vlc.discovery.OsxNativeDiscoveryStrategy;
import com.github.yoep.vlc.discovery.WindowsNativeDiscoveryStrategy;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.web.reactive.function.client.WebClient;
import uk.co.caprica.vlcj.factory.discovery.NativeDiscovery;
import uk.co.caprica.vlcj.factory.discovery.strategy.NativeDiscoveryStrategy;

import static java.util.Arrays.asList;

@Slf4j
@Configuration
public class VlcConfig {
    @Bean
    public NativeDiscovery nativeDiscovery(FxLib fxLib, PopcornFx instance) {
        if (OnVlcVideoEnabled.matches(fxLib, instance)) {
            var discoveryStrategies = asList(
                    linuxNativeDiscoveryStrategy(),
                    osxNativeDiscoveryStrategy(),
                    windowsNativeDiscoveryStrategy()
            );

            log.debug("Creating new NativeDiscovery for VLC");
            return new NativeDiscovery(discoveryStrategies.toArray(NativeDiscoveryStrategy[]::new));
        } else {
            return null;
        }
    }

    @Bean
    public VlcDiscovery vlcDiscovery(PlayerManagerService playerManagerService, VlcPlayerService vlcPlayerService) {
        return new VlcDiscovery(playerManagerService, vlcPlayerService);
    }

    @Bean
    public WebClient vlcWebClient() {
        return WebClient.builder()
                .defaultHeaders(header -> header.setBasicAuth("", VlcPlayerConstants.PASSWORD))
                .build();
    }

    @Bean
    public VlcPlayerService vlcPlayerService(PlatformProvider platformProvider,
                                             SubtitleService subtitleService,
                                             WebClient vlcWebClient) {
        return new VlcPlayerService(platformProvider, subtitleService, vlcWebClient);
    }

    private static NativeDiscoveryStrategy linuxNativeDiscoveryStrategy() {
        return new LinuxNativeDiscoveryStrategy();
    }

    private static NativeDiscoveryStrategy osxNativeDiscoveryStrategy() {
        return new OsxNativeDiscoveryStrategy();
    }

    private static NativeDiscoveryStrategy windowsNativeDiscoveryStrategy() {
        return new WindowsNativeDiscoveryStrategy();
    }
}
