package com.github.yoep.popcorn.ui.config;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.vlc.conditions.OnVlcVideoEnabled;
import com.github.yoep.vlc.discovery.LinuxNativeDiscoveryStrategy;
import com.github.yoep.vlc.discovery.OsxNativeDiscoveryStrategy;
import com.github.yoep.vlc.discovery.WindowsNativeDiscoveryStrategy;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
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
