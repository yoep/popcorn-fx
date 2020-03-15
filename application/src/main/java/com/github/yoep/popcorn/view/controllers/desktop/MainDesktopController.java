package com.github.yoep.popcorn.view.controllers.desktop;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.spring.boot.javafx.view.ViewManager;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.view.controllers.MainController;
import com.github.yoep.popcorn.view.controllers.common.AbstractMainController;
import com.github.yoep.popcorn.view.services.UrlService;
import javafx.application.Platform;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import lombok.Builder;
import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.ApplicationArguments;
import org.springframework.core.task.TaskExecutor;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
public class MainDesktopController extends AbstractMainController implements MainController {
    private Pane overlayPane;

    //region Constructors

    @Builder
    public MainDesktopController(ActivityManager activityManager,
                                 ViewLoader viewLoader,
                                 ViewManager viewManager,
                                 TaskExecutor taskExecutor,
                                 SettingsService settingsService,
                                 ApplicationArguments arguments,
                                 UrlService urlService) {
        super(activityManager, viewLoader, viewManager, arguments, settingsService, urlService, taskExecutor);
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        super.initialize(url, resourceBundle);

        if (!processApplicationArguments())
            switchSection(SectionType.CONTENT);
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializePanes();
        initializeListeners();
    }

    @Override
    protected void initializePanes() {
        // load the other panes on a different thread
        taskExecutor.execute(() -> {
            overlayPane = viewLoader.load("sections/overlay.section.fxml");
        });
    }

    @Override
    protected void initializeListeners() {
        activityManager.register(PlayVideoActivity.class, activity -> switchSection(SectionType.PLAYER));
        activityManager.register(ShowSettingsActivity.class, activity -> switchSection(SectionType.SETTINGS));
        activityManager.register(LoadActivity.class, activity -> switchSection(SectionType.LOADER));
        activityManager.register(OverlayActivity.class, activity -> switchSection(SectionType.OVERLAY));

        activityManager.register(CloseSettingsActivity.class, activity -> switchSection(SectionType.CONTENT));
        activityManager.register(ClosePlayerActivity.class, activity -> switchSection(SectionType.CONTENT));
        activityManager.register(CloseLoadActivity.class, activity -> switchSection(SectionType.CONTENT));
        activityManager.register(CloseOverlayActivity.class, activity -> switchSection(SectionType.CONTENT));
    }

    //endregion

    //region Functions

    private void switchSection(SectionType sectionType) {
        AtomicReference<Pane> content = new AtomicReference<>();

        switch (sectionType) {
            case CONTENT:
                content.set(contentPane);
                break;
            case SETTINGS:
                content.set(settingsPane);
                break;
            case PLAYER:
                content.set(playerPane);
                break;
            case LOADER:
                content.set(loaderPane);
                break;
            case OVERLAY:
                content.set(overlayPane);
                break;
        }

        setAnchor(content.get());

        Platform.runLater(() -> {
            rootPane.getChildren().removeIf(e -> e != notificationPane);
            rootPane.getChildren().add(0, content.get());
        });
    }

    private void setAnchor(Pane pane) {
        AnchorPane.setTopAnchor(pane, 0d);
        AnchorPane.setRightAnchor(pane, 0d);
        AnchorPane.setBottomAnchor(pane, 0d);
        AnchorPane.setLeftAnchor(pane, 0d);
    }

    //endregion

    private enum SectionType {
        CONTENT,
        SETTINGS,
        PLAYER,
        LOADER,
        OVERLAY
    }
}
