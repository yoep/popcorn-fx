package com.github.yoep.popcorn.controllers.sections;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.providers.models.FileMedia;
import com.github.yoep.popcorn.providers.models.Media;
import com.github.yoep.popcorn.subtitle.models.SubtitleInfo;
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
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Controller;

import javax.annotation.PostConstruct;
import java.io.File;
import java.io.IOException;
import java.net.URL;
import java.nio.file.Files;
import java.util.List;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@Controller
@RequiredArgsConstructor
public class ContentSectionController implements Initializable {
    private static final KeyCodeCombination PASTE_KEY_COMBINATION = new KeyCodeCombination(KeyCode.V, KeyCombination.CONTROL_DOWN);

    private final ActivityManager activityManager;
    private final ViewLoader viewLoader;
    private final TaskExecutor taskExecutor;

    private Pane listPane;
    private Pane detailsPane;
    private ContentType activeType;

    @FXML
    private Pane rootPane;

    //region Methods

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        switchContent(ContentType.LIST);
        initializeSceneEvents();
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializeListeners();
        initializePanes();
    }

    private void initializeListeners() {
        activityManager.register(ShowDetailsActivity.class, activity -> switchContent(ContentType.DETAILS));
        activityManager.register(CloseDetailsActivity.class, activity -> switchContent(ContentType.LIST));
        activityManager.register(CategoryChangedActivity.class, activity -> switchContent(ContentType.LIST));
    }

    private void initializePanes() {
        // load the list pane on the main thread
        // this blocks Spring from completing the startup stage while this pane is being loaded
        listPane = viewLoader.load("sections/list.section.fxml");
        setAnchor(listPane);

        // load the details pane on a different thread
        taskExecutor.execute(() -> {
            detailsPane = viewLoader.load("sections/details.section.fxml");
            setAnchor(detailsPane);
        });
    }

    //endregion

    //region Functions

    private void initializeSceneEvents() {
        rootPane.setOnKeyReleased(event -> {
            if (PASTE_KEY_COMBINATION.match(event)) {
                onContentPasted();
            }
        });

        rootPane.setOnDragOver(this::onDragOver);
        rootPane.setOnDragDropped(this::onDragDropped);
    }

    private void switchContent(ContentType contentType) {
        if (activeType == contentType)
            return;

        AtomicReference<Pane> pane = new AtomicReference<>();
        this.activeType = contentType;

        switch (contentType) {
            case LIST:
                pane.set(listPane);
                break;
            case DETAILS:
                pane.set(detailsPane);
                break;
        }

        Platform.runLater(() -> {
            if (rootPane.getChildren().size() > 1)
                rootPane.getChildren().remove(0);

            rootPane.getChildren().add(0, pane.get());
        });
    }

    private void setAnchor(Pane pane) {
        AnchorPane.setTopAnchor(pane, 50d);
        AnchorPane.setRightAnchor(pane, 0d);
        AnchorPane.setBottomAnchor(pane, 0d);
        AnchorPane.setLeftAnchor(pane, 0d);
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

        }
    }

    private void onDragOver(DragEvent event) {
        List<File> files = event.getDragboard().getFiles();

        if (CollectionUtils.isNotEmpty(files)) {
            log.trace("Processing drag content");
            File file = files.get(0);

            try {
                String contentType = Files.probeContentType(file.toPath());
                String format = contentType.split("/")[0];

                if (format.equals("video"))
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
            public Optional<String> getQuality() {
                return Optional.empty();
            }

            @Override
            public Media getMedia() {
                return FileMedia.builder()
                        .title(title)
                        .build();
            }

            @Override
            public Optional<SubtitleInfo> getSubtitle() {
                return Optional.empty();
            }
        });
    }

    //endregion

    private enum ContentType {
        LIST,
        DETAILS
    }
}
