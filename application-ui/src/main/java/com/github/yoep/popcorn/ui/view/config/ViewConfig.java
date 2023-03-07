package com.github.yoep.popcorn.ui.view.config;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.spring.boot.javafx.view.ViewManager;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.FxLibInstance;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.OptionsService;
import com.github.yoep.popcorn.ui.view.PopcornViewLoader;
import com.github.yoep.popcorn.ui.view.controllers.MainController;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.DesktopFilterComponent;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.WindowComponent;
import com.github.yoep.popcorn.ui.view.controllers.tv.components.SystemTimeComponent;
import com.github.yoep.popcorn.ui.view.controllers.tv.components.TvFilterComponent;
import com.github.yoep.popcorn.ui.view.services.MaximizeService;
import com.github.yoep.popcorn.ui.view.services.UrlService;
import org.springframework.boot.ApplicationArguments;
import org.springframework.context.ApplicationContext;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.task.TaskExecutor;
import org.springframework.scheduling.annotation.EnableScheduling;

@Configuration
@EnableScheduling
public class ViewConfig {
    @Bean
    public MainController mainController(ApplicationEventPublisher eventPublisher,
                                         ViewLoader viewLoader,
                                         ApplicationArguments arguments,
                                         UrlService urlService,
                                         ApplicationConfig settingsService,
                                         OptionsService optionsService,
                                         TaskExecutor taskExecutor) {
        return new MainController(eventPublisher, viewLoader, arguments, urlService, settingsService, optionsService, taskExecutor);
    }

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
    public DesktopFilterComponent desktopFilterComponent(LocaleText localeText,
                                                         EventPublisher eventPublisher,
                                                         FxLib fxLib,
                                                         PopcornFx instance) {
        if (!isTvModeEnabled()) {
            return new DesktopFilterComponent(localeText, eventPublisher, fxLib, instance);
        }

        return null;
    }

    @Bean
    public TvFilterComponent tvFilterComponent() {
        if (isTvModeEnabled()) {
            return new TvFilterComponent();
        }

        return null;
    }

    @Bean
    public WindowComponent windowComponent(MaximizeService maximizeService,
                                           PlatformProvider platformProvider) {
        if (!isTvModeEnabled()) {
            return new WindowComponent(maximizeService, platformProvider);
        }

        return null;
    }

    @Bean
    public SystemTimeComponent systemTimeComponent() {
        if (isTvModeEnabled()) {
            return new SystemTimeComponent();
        }

        return null;
    }

    private static boolean isTvModeEnabled() {
        return FxLibInstance.INSTANCE.get().is_tv_mode(PopcornFxInstance.INSTANCE.get()) == 1;
    }
}
