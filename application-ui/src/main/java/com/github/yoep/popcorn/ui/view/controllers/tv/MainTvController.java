package com.github.yoep.popcorn.ui.view.controllers.tv;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.ui.events.*;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.view.controllers.MainController;
import com.github.yoep.popcorn.ui.view.controllers.common.AbstractMainController;
import com.github.yoep.popcorn.ui.view.services.UrlService;
import javafx.application.Platform;
import javafx.scene.layout.Pane;
import lombok.Builder;
import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.ApplicationArguments;
import org.springframework.core.task.TaskExecutor;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
public class MainTvController extends AbstractMainController implements MainController {
    private static final String TV_STYLESHEET = "/styles/tv.css";

    //region Constructors

    @Builder
    public MainTvController(ActivityManager activityManager,
                            ViewLoader viewLoader,
                            ApplicationArguments arguments,
                            UrlService urlService,
                            SettingsService settingsService,
                            TaskExecutor taskExecutor) {
        super(activityManager, viewLoader, arguments, urlService, settingsService, taskExecutor);
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        super.initialize(url, resourceBundle);
        initializeStylesheets();

        if (!processApplicationArguments())
            switchSection(SectionType.CONTENT);
    }

    private void initializeStylesheets() {
        rootPane.getStylesheets().add(TV_STYLESHEET);
    }

    //endregion

    //region PostConstruct

    @Override
    protected void initializeListeners() {
        activityManager.register(ShowDetailsEvent.class, activity -> switchSection(SectionType.CONTENT));
        activityManager.register(PlayVideoEvent.class, activity -> switchSection(SectionType.PLAYER));
        activityManager.register(LoadEvent.class, activity -> switchSection(SectionType.LOADER));

        activityManager.register(ClosePlayerEvent.class, activity -> switchSection(SectionType.CONTENT));
        activityManager.register(CloseLoadEvent.class, activity -> switchSection(SectionType.CONTENT));
    }

    //endregion

    private void switchSection(SectionType sectionType) {
        AtomicReference<Pane> content = new AtomicReference<>();

        switch (sectionType) {
            case CONTENT:
                content.set(contentPane);
                break;
            case PLAYER:
                content.set(playerPane);
                break;
            case LOADER:
                content.set(loaderPane);
                break;
        }

        Platform.runLater(() -> {
            rootPane.getChildren().removeIf(e -> e != notificationPane);
            rootPane.getChildren().add(0, content.get());
        });
    }

    private enum SectionType {
        CONTENT,
        PLAYER,
        LOADER
    }
}
