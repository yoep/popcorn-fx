package com.github.yoep.popcorn.backend.config;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class SettingsConfig {
    @Bean
    public ApplicationConfig applicationConfig(LocaleText localeText,
                                               FxLib fxLib,
                                               PopcornFx instance) {
        return new ApplicationConfig(localeText, fxLib, instance);
    }
}
