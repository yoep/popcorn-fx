package com.github.yoep.popcorn.controllers;

import com.github.spring.boot.javafx.ui.scale.ScaleAwareImpl;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.spring.boot.javafx.view.ViewManager;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.settings.models.UISettings;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.input.*;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.collections4.CollectionUtils;
import org.apache.commons.io.FilenameUtils;
import org.apache.commons.lang3.StringUtils;
import org.springframework.boot.ApplicationArguments;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;

import javax.annotation.PostConstruct;
import java.io.File;
import java.io.IOException;
import java.net.URL;
import java.nio.file.Files;
import java.util.List;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;
import java.util.regex.Pattern;

@Slf4j
@Component
@RequiredArgsConstructor
public class MainController extends ScaleAwareImpl implements Initializable {
    private static final KeyCodeCombination PASTE_KEY_COMBINATION = new KeyCodeCombination(KeyCode.V, KeyCombination.CONTROL_DOWN);
    private static final Pattern URL_TYPE_PATTERN = Pattern.compile("([a-zA-Z]*):?(.*)");

    private final ActivityManager activityManager;
    private final ViewLoader viewLoader;
    private final ViewManager viewManager;
    private final TaskExecutor taskExecutor;
    private final SettingsService settingsService;
    private final ApplicationArguments arguments;

    private Pane contentPane;
    private Pane settingsPane;
    private Pane playerPane;
    private Pane loaderPane;
    private Pane overlayPane;

    @FXML
    private Pane rootPane;

    //region Methods

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        switchSection(SectionType.CONTENT);
        initializeSceneEvents();
        initializeStageListeners();

        processApplicationArguments();
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializePanes();
        initializeListeners();
    }

    private void initializePanes() {
        // load the content pane on the main thread
        // this blocks Spring from completing the startup stage while this pane is being loaded
        contentPane = viewLoader.load("sections/content.section.fxml");

        // load the other panes on a different thread
        taskExecutor.execute(() -> settingsPane = viewLoader.load("sections/settings.section.fxml"));
        taskExecutor.execute(() -> playerPane = viewLoader.load("sections/player.section.fxml"));
        taskExecutor.execute(() -> loaderPane = viewLoader.load("sections/loader.section.fxml"));
        taskExecutor.execute(() -> overlayPane = viewLoader.load("sections/overlay.section.fxml"));
    }

    private void initializeListeners() {
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

    private void processApplicationArguments() {
        var nonOptionArgs = arguments.getNonOptionArgs();

        if (nonOptionArgs.size() > 0)
            processUrl(nonOptionArgs.get(0));
    }

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
            rootPane.getChildren().clear();
            rootPane.getChildren().add(content.get());
        });
    }

    private void onContentPasted() {
        Clipboard clipboard = Clipboard.getSystemClipboard();
        String url = clipboard.getUrl();
        List<File> files = clipboard.getFiles();

        if (CollectionUtils.isNotEmpty(files)) {
            log.trace("Processing clipboard files");
            processFiles(files);
        } else if (StringUtils.isNotEmpty(url)) {
            log.trace("Processing clipboard url");
            processUrl(url);
        } else {
            log.trace("Processing clipboard string");
            processUrl(clipboard.getString());
        }
    }

    private void onDragOver(DragEvent event) {
        List<File> files = event.getDragboard().getFiles();

        if (CollectionUtils.isNotEmpty(files)) {
            log.trace("Processing drag content");
            File file = files.get(0);

            try {
                if (isVideoFile(file))
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

    private void processUrl(String url) {
        var matcher = URL_TYPE_PATTERN.matcher(url);

        if (matcher.matches()) {
            var type = matcher.group(1);
            log.trace("Found type \"{}\" for url {}", type, url);

            if (isWebUrl(type)) {
                log.debug("Opening web url: {}", url);
                activityManager.register(new PlayVideoActivity() {
                    @Override
                    public String getUrl() {
                        return url;
                    }

                    @Override
                    public String getTitle() {
                        return "";
                    }

                    @Override
                    public boolean isSubtitlesEnabled() {
                        return false;
                    }
                });
            } else if (isMagnetLink(type)) {
                log.debug("Opening magnet link: {}", url);
                activityManager.register((LoadUrlActivity) () -> url);
            } else {
                var file = new File(url);

                // check if the url is a valid file
                if (file.exists()) {
                    try {
                        if (isVideoFile(file)) {
                            log.debug("Opening video file: {}", url);
                            activityManager.register(new PlayVideoActivity() {
                                @Override
                                public String getUrl() {
                                    return url;
                                }

                                @Override
                                public String getTitle() {
                                    return FilenameUtils.getBaseName(url);
                                }

                                @Override
                                public boolean isSubtitlesEnabled() {
                                    return false;
                                }
                            });
                        }
                    } catch (IOException ex) {
                        log.error("Failed to process url, " + ex.getMessage(), ex);
                    }
                } else {
                    log.warn("Failed to process url, file \"{}\" does not exist", url);
                }
            }
        } else {
            log.warn("Failed to process url, url \"{}\" is invalid", url);
        }
    }

    private boolean isWebUrl(String type) {
        return type.equalsIgnoreCase("http") || type.equalsIgnoreCase("https");
    }

    private boolean isMagnetLink(String type) {
        return type.equalsIgnoreCase("magnet");
    }

    private boolean isVideoFile(File file) throws IOException {
        var contentType = Files.probeContentType(file.toPath());
        var format = contentType.split("/")[0];

        return format.equalsIgnoreCase("video");
    }

    private void setAnchor(Pane pane) {
        AnchorPane.setTopAnchor(pane, 0d);
        AnchorPane.setRightAnchor(pane, 0d);
        AnchorPane.setBottomAnchor(pane, 0d);
        AnchorPane.setLeftAnchor(pane, 0d);
    }

    private UISettings getUiSettings() {
        return settingsService.getSettings().getUiSettings();
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
