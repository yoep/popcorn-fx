package com.github.yoep.popcorn.ui.config;

import com.github.yoep.popcorn.Application;
import com.github.yoep.popcorn.PopcornFxManager;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.platform.NativePlatform;
import com.github.yoep.popcorn.platform.PlatformFX;
import com.github.yoep.popcorn.platform.PlatformInfo;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

import javax.annotation.PreDestroy;

@Configuration
public class PlatformConfig {
    @Bean
    public PlatformProvider platformProvider() {
        return new PlatformFX(new NativePlatform() {
            @Override
            public PlatformInfo platformInfo() {
                return Application.INSTANCE.platform_info(PopcornFxManager.INSTANCE.fxInstance());
            }

            @Override
            public void enableScreensaver() {
                Application.INSTANCE.enable_screensaver(PopcornFxManager.INSTANCE.fxInstance());
            }

            @Override
            public void disableScreensaver() {
                Application.INSTANCE.disable_screensaver(PopcornFxManager.INSTANCE.fxInstance());
            }
        });
    }

    @PreDestroy
    public void onDestroy() {
        PopcornFxManager.INSTANCE.dispose();
    }
}
