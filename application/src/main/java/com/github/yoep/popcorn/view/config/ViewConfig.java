package com.github.yoep.popcorn.view.config;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.spring.boot.javafx.view.ViewManager;
import com.github.yoep.popcorn.settings.OptionsService;
import com.github.yoep.popcorn.view.PopcornViewLoader;
import org.springframework.context.ApplicationContext;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class ViewConfig {
    @Bean
    public ViewLoader viewLoader(ApplicationContext applicationContext, ViewManager viewManager, LocaleText localeText, OptionsService optionsService) {
        return PopcornViewLoader.builder()
                .applicationContext(applicationContext)
                .viewManager(viewManager)
                .localeText(localeText)
                .optionsService(optionsService)
                .build();
    }
}
