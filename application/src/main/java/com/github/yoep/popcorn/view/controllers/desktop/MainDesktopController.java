package com.github.yoep.popcorn.view.controllers.desktop;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.view.controllers.MainController;
import com.github.yoep.popcorn.view.controllers.common.AbstractMainController;
import com.github.yoep.popcorn.view.services.UrlService;
import javafx.application.Platform;
import javafx.scene.input.*;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import lombok.Builder;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.collections4.CollectionUtils;
import org.apache.commons.io.FilenameUtils;
import org.apache.commons.lang3.StringUtils;
import org.springframework.boot.ApplicationArguments;
import org.springframework.core.task.TaskExecutor;

import javax.annotation.PostConstruct;
import java.io.File;
import java.io.IOException;
import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
public class MainDesktopController extends AbstractMainController implements MainController {
    private static final KeyCodeCombination PASTE_KEY_COMBINATION = new KeyCodeCombination(KeyCode.V, KeyCombination.CONTROL_DOWN);

    private Pane overlayPane;

    //region Constructors

    @Builder
    public MainDesktopController(ActivityManager activityManager,
                                 ViewLoader viewLoader,
                                 TaskExecutor taskExecutor,
                                 ApplicationArguments arguments,
                                 UrlService urlService) {
        super(activityManager, viewLoader, arguments, urlService, taskExecutor);
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        super.initialize(url, resourceBundle);
        initializeSceneEvents();

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
        taskExecutor.execute(() -> overlayPane = viewLoader.load("sections/overlay.section.fxml"));
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
