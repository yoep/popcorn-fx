package com.github.yoep.popcorn.view.controllers.common;

import com.github.spring.boot.javafx.ui.scale.ScaleAwareImpl;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.spring.boot.javafx.view.ViewManager;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.PlayVideoActivity;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.settings.models.UISettings;
import com.github.yoep.popcorn.view.controllers.MainController;
import com.github.yoep.popcorn.view.services.UrlService;
import javafx.fxml.FXML;
import javafx.scene.input.*;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.collections4.CollectionUtils;
import org.apache.commons.io.FilenameUtils;
import org.apache.commons.lang3.StringUtils;
import org.springframework.boot.ApplicationArguments;
import org.springframework.core.task.TaskExecutor;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import java.io.File;
import java.io.IOException;
import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;

@Slf4j
public abstract class AbstractMainController extends ScaleAwareImpl implements MainController {
    private static final KeyCodeCombination PASTE_KEY_COMBINATION = new KeyCodeCombination(KeyCode.V, KeyCombination.CONTROL_DOWN);

    protected final ActivityManager activityManager;
    protected final ViewLoader viewLoader;
    protected final ViewManager viewManager;
    protected final ApplicationArguments arguments;
    protected final SettingsService settingsService;
    protected final UrlService urlService;
    protected final TaskExecutor taskExecutor;

    protected Pane contentPane;
    protected Pane settingsPane;
    protected Pane playerPane;
    protected Pane loaderPane;
    protected Pane notificationPane;

    @FXML
    protected Pane rootPane;

    //region Constructors

    protected AbstractMainController(ActivityManager activityManager,
                                     ViewLoader viewLoader,
                                     ViewManager viewManager,
                                     ApplicationArguments arguments,
                                     SettingsService settingsService,
                                     UrlService urlService,
                                     TaskExecutor taskExecutor) {
        Assert.notNull(activityManager, "activityManager cannot be null");
        Assert.notNull(viewLoader, "viewLoader cannot be null");
        Assert.notNull(viewManager, "viewManager cannot be null");
        Assert.notNull(arguments, "arguments cannot be null");
        Assert.notNull(settingsService, "settingsService cannot be null");
        Assert.notNull(urlService, "urlService cannot be null");
        Assert.notNull(taskExecutor, "taskExecutor cannot be null");
        this.activityManager = activityManager;
        this.viewLoader = viewLoader;
        this.viewManager = viewManager;
        this.arguments = arguments;
        this.settingsService = settingsService;
        this.urlService = urlService;
        this.taskExecutor = taskExecutor;
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeSceneEvents();
        initializeStageListeners();
        initializeNotificationPane();
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializePanesInternal();
        initializePanes();
        initializeListeners();
    }

    private void initializePanesInternal() {
        // load the content & notification pane on the main thread
        // this blocks Spring from completing the startup stage while these panes are being loaded
        contentPane = viewLoader.load("sections/content.section.fxml");
        notificationPane = viewLoader.load("common/sections/notification.section.fxml");

        // load the other panes on a different thread
        taskExecutor.execute(() -> {
            settingsPane = viewLoader.load("sections/settings.section.fxml");
            playerPane = viewLoader.load("sections/player.section.fxml");
            loaderPane = viewLoader.load("sections/loader.section.fxml");
        });
    }

    protected abstract void initializePanes();

    protected abstract void initializeListeners();

    //endregion

    //region Functions

    protected boolean processApplicationArguments() {
        var nonOptionArgs = arguments.getNonOptionArgs();

        if (nonOptionArgs.size() > 0) {
            log.debug("Retrieved the following non-option argument: {}", nonOptionArgs);

            // try to process the url that has been passed along the application during startup
            // if the url is processed with success, wait for the activity event to change the section
            // otherwise, we still show the content section
            return urlService.process(nonOptionArgs.get(0));
        }

        return false;
    }

    private void initializeSceneEvents() {
        rootPane.setOnKeyPressed(event -> {
            if (PASTE_KEY_COMBINATION.match(event)) {
                event.consume();
                onContentPasted();
            }
        });

        rootPane.setOnDragOver(this::onDragOver);
        rootPane.setOnDragDropped(this::onDragDropped);
    }

    private void initializeStageListeners() {
        viewManager.getPrimaryStage().ifPresent(stage -> stage.maximizedProperty().addListener((observable, oldValue, newValue) -> {
            var uiSettings = getUiSettings();

            log.trace("Stage maximized state is being changed from \"{}\" to \"{}\"", oldValue, newValue);
            uiSettings.setMaximized(newValue);
        }));
    }

    private void initializeNotificationPane() {
        AnchorPane.setTopAnchor(notificationPane, 55.0);
        AnchorPane.setRightAnchor(notificationPane, 20.0);

        rootPane.getChildren().add(notificationPane);
    }

    private void onContentPasted() {
        var clipboard = Clipboard.getSystemClipboard();
        var url = clipboard.getUrl();
        var files = clipboard.getFiles();

        if (CollectionUtils.isNotEmpty(files)) {
            log.trace("Processing clipboard files");
            processFiles(files);
        } else if (StringUtils.isNotEmpty(url)) {
            log.trace("Processing clipboard url");
            urlService.process(url);
        } else if (StringUtils.isNotEmpty(clipboard.getString())) {
            log.trace("Processing clipboard string");
            urlService.process(clipboard.getString());
        } else {
            log.debug("Ignoring content pasted action, not content available on the clipboard");
        }
    }

    private void onDragOver(DragEvent event) {
        List<File> files = event.getDragboard().getFiles();

        if (CollectionUtils.isNotEmpty(files)) {
            log.trace("Processing drag content");
            File file = files.get(0);

            try {
                if (urlService.isVideoFile(file))
                    event.acceptTransferModes(TransferMode.ANY);
            } catch (IOException ex) {
                log.error("Failed to detect drag content type, " + ex.getMessage(), ex);
            }
        }
    }

    private void onDragDropped(DragEvent event) {
        List<File> files = event.getDragboard().getFiles();

        if (CollectionUtils.isNotEmpty(files)) {
            processFiles(files);
        }
    }

    private void processFiles(List<File> files) {
        File file = files.get(0);
        String title = FilenameUtils.getBaseName(file.getName());

        activityManager.register(new PlayVideoActivity() {
            @Override
            public String getUrl() {
                return file.getAbsolutePath();
            }

            @Override
            public String getTitle() {
                return title;
            }

            @Override
            public boolean isSubtitlesEnabled() {
                return false;
            }
        });
    }

    protected UISettings getUiSettings() {
        return settingsService.getSettings().getUiSettings();
    }

    //endregion
}
