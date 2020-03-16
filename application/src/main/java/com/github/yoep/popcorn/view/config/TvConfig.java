package com.github.yoep.popcorn.view.config;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.view.conditions.ConditionalOnTvMode;
import com.github.yoep.popcorn.view.controllers.MainController;
import com.github.yoep.popcorn.view.controllers.tv.MainTvController;
import com.github.yoep.popcorn.view.controllers.tv.sections.*;
import com.github.yoep.popcorn.view.services.UrlService;
import org.springframework.boot.ApplicationArguments;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.task.TaskExecutor;

@Configuration
@ConditionalOnTvMode
public class TvConfig {
    @Bean
    public MainController mainController(ActivityManager activityManager, ViewLoader viewLoader, ApplicationArguments arguments, UrlService urlService,
                                         TaskExecutor taskExecutor) {
        return MainTvController.builder()
                .activityManager(activityManager)
                .viewLoader(viewLoader)
                .arguments(arguments)
                .urlService(urlService)
                .taskExecutor(taskExecutor)
                .build();
    }

    @Bean
    public ContentSectionController contentSectionTvController() {
        return new ContentSectionController();
    }

    @Bean
    public ListSectionController listSectionTvController() {
        return new ListSectionController();
    }

    @Bean
    public LoaderSectionController loaderSectionTvController() {
        return new LoaderSectionController();
    }

    @Bean
    public MenuSectionController menuSectionTvController() {
        return new MenuSectionController();
    }

    @Bean
    public PlayerSectionController playerSectionTvController() {
        return new PlayerSectionController();
    }

    @Bean
    public SettingsSectionController settingsSectionTvController() {
        return new SettingsSectionController();
    }
}
