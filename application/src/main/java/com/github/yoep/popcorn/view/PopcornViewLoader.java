package com.github.yoep.popcorn.view;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewManager;
import com.github.spring.boot.javafx.view.impl.ViewLoaderImpl;
import com.github.yoep.popcorn.settings.OptionsService;
import javafx.fxml.FXMLLoader;
import lombok.Builder;
import org.springframework.context.ApplicationContext;

import java.io.File;

public class PopcornViewLoader extends ViewLoaderImpl {
    private static final String COMMON_DIRECTORY = "common";

    private final OptionsService optionsService;

    @Builder
    public PopcornViewLoader(ApplicationContext applicationContext, ViewManager viewManager, LocaleText localeText, OptionsService optionsService) {
        super(applicationContext, viewManager, localeText);
        this.optionsService = optionsService;
    }

    //region Functions

    @Override
    protected FXMLLoader loadResource(String view) {
        // check if the view is located in the common directory
        // if so, don't add a path prefix to the view
        // otherwise, add a path prefix to the view for the current mode
        if (isLocatedInCommonDirectory(view)) {
            return super.loadResource(view);
        } else {
            var base = isTvMode() ? "tv" : "desktop";

            return super.loadResource(base + File.separator + view);
        }
    }

    private boolean isLocatedInCommonDirectory(String view) {
        return view.startsWith(COMMON_DIRECTORY);
    }

    private boolean isTvMode() {
        return optionsService.options().isTvMode();
    }

    //endregion
}
