package com.github.yoep.vlc.config;

import com.github.yoep.vlc.conditions.ConditionalOnVlcVideoEnabled;
import com.github.yoep.vlc.discovery.LinuxNativeDiscoveryStrategy;
import com.github.yoep.vlc.discovery.OsxNativeDiscoveryStrategy;
import com.github.yoep.vlc.discovery.WindowsNativeDiscoveryStrategy;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import uk.co.caprica.vlcj.factory.discovery.NativeDiscovery;
import uk.co.caprica.vlcj.factory.discovery.strategy.NativeDiscoveryStrategy;

import java.util.List;

@Slf4j
@Configuration
@ConditionalOnVlcVideoEnabled
public class VlcConfig {
    @Bean
    public NativeDiscovery nativeDiscovery(List<NativeDiscoveryStrategy> discoveryStrategies) {
        return new NativeDiscovery(discoveryStrategies.toArray(NativeDiscoveryStrategy[]::new));
    }

    @Bean
    public NativeDiscoveryStrategy linuxNativeDiscoveryStrategy() {
        return new LinuxNativeDiscoveryStrategy();
    }

    @Bean
    public NativeDiscoveryStrategy osxNativeDiscoveryStrategy() {
        return new OsxNativeDiscoveryStrategy();
    }

    @Bean
    public NativeDiscoveryStrategy windowsNativeDiscoveryStrategy() {
        return new WindowsNativeDiscoveryStrategy();
    }
}
