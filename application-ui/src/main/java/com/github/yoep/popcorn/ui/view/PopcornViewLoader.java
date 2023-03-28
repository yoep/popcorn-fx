package com.github.yoep.popcorn.ui.view;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoaderImpl;
import com.github.spring.boot.javafx.view.ViewManager;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import javafx.fxml.FXMLLoader;
import org.springframework.context.ApplicationContext;

import java.io.File;

public class PopcornViewLoader extends ViewLoaderImpl {
    private static final String COMMON_DIRECTORY = "common";

    private final ApplicationConfig applicationConfig;

    public PopcornViewLoader(ApplicationContext applicationContext,
                             ViewManager viewManager,
                             LocaleText localeText,
                             ApplicationConfig applicationConfig) {
        super(applicationContext, viewManager, localeText);
        this.applicationConfig = applicationConfig;
        init();
    }

    float getScale() {
        return scale;
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

    private void init() {
        applicationConfig.setOnUiScaleChanged(this::setScale);
    }

    private boolean isLocatedInCommonDirectory(String view) {
        return view.startsWith(COMMON_DIRECTORY);
    }

    private boolean isTvMode() {
        return applicationConfig.isTvMode();
    }

    //endregion
}
