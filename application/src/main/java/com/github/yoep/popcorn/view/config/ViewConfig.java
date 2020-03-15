package com.github.yoep.popcorn.view.config;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.spring.boot.javafx.view.ViewManager;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.settings.OptionsService;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.view.PopcornViewLoader;
import com.github.yoep.popcorn.view.conditions.ConditionalOnTvMode;
import com.github.yoep.popcorn.view.controllers.MainController;
import com.github.yoep.popcorn.view.controllers.desktop.MainDesktopController;
import com.github.yoep.popcorn.view.controllers.tv.MainTvController;
import com.github.yoep.popcorn.view.services.UrlService;
import org.springframework.boot.ApplicationArguments;
import org.springframework.boot.autoconfigure.condition.ConditionalOnMissingBean;
import org.springframework.context.ApplicationContext;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import org.springframework.core.task.TaskExecutor;

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

    @Bean
    @Order(Ordered.HIGHEST_PRECEDENCE + 10)
    @ConditionalOnTvMode
    public MainController tvController(ActivityManager activityManager, ViewLoader viewLoader, ViewManager viewManager, ApplicationArguments arguments,
                                       SettingsService settingsService, UrlService urlService, TaskExecutor taskExecutor) {
        return MainTvController.builder()
                .activityManager(activityManager)
                .viewLoader(viewLoader)
                .viewManager(viewManager)
                .arguments(arguments)
                .settingsService(settingsService)
                .urlService(urlService)
                .taskExecutor(taskExecutor)
                .build();
    }

    @Bean
    @Order(Ordered.HIGHEST_PRECEDENCE + 11)
    @ConditionalOnMissingBean(MainController.class)
    public MainController desktopController(ActivityManager activityManager, ViewLoader viewLoader, ViewManager viewManager, TaskExecutor taskExecutor,
                                            SettingsService settingsService, ApplicationArguments arguments, UrlService urlService) {
        return MainDesktopController.builder()
                .activityManager(activityManager)
                .arguments(arguments)
                .settingsService(settingsService)
                .taskExecutor(taskExecutor)
                .viewLoader(viewLoader)
                .viewManager(viewManager)
                .urlService(urlService)
                .build();
    }
}
