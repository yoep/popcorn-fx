package com.github.yoep.popcorn.ui.view.controllers;

import com.github.spring.boot.javafx.ui.scale.ScaleAwareImpl;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayVideoEvent;
import com.github.yoep.popcorn.backend.events.ShowDetailsEvent;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.events.CloseLoadEvent;
import com.github.yoep.popcorn.ui.events.LoadEvent;
import com.github.yoep.popcorn.ui.stage.BorderlessStageHolder;
import com.github.yoep.popcorn.ui.view.services.UrlService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Cursor;
import javafx.scene.Node;
import javafx.scene.Scene;
import javafx.scene.input.*;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.collections4.CollectionUtils;
import org.apache.commons.io.FilenameUtils;
import org.apache.commons.lang3.StringUtils;
import org.springframework.boot.ApplicationArguments;

import java.io.File;
import java.io.IOException;
import java.net.URL;
import java.util.List;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@RequiredArgsConstructor
public class MainController extends ScaleAwareImpl implements Initializable {
    static final String TV_STYLESHEET = "/styles/tv.css";
    static final String MOUSE_DISABLED_STYLE_CLASS = "mouse-disabled";

    private static final KeyCodeCombination PASTE_KEY_COMBINATION = new KeyCodeCombination(KeyCode.V, KeyCombination.CONTROL_DOWN);
    private static final KeyCodeCombination UI_ENLARGE_KEY_COMBINATION_1 = new KeyCodeCombination(KeyCode.ADD, KeyCombination.CONTROL_DOWN);
    private static final KeyCodeCombination UI_ENLARGE_KEY_COMBINATION_2 = new KeyCodeCombination(KeyCode.PLUS, KeyCombination.CONTROL_DOWN);
    private static final KeyCodeCombination UI_ENLARGE_KEY_COMBINATION_3 = new KeyCodeCombination(KeyCode.EQUALS, KeyCombination.CONTROL_DOWN,
            KeyCombination.SHIFT_DOWN);
    private static final KeyCodeCombination UI_REDUCE_KEY_COMBINATION_1 = new KeyCodeCombination(KeyCode.SUBTRACT, KeyCombination.CONTROL_DOWN);
    private static final KeyCodeCombination UI_REDUCE_KEY_COMBINATION_2 = new KeyCodeCombination(KeyCode.MINUS, KeyCombination.CONTROL_DOWN);

    private final EventPublisher eventPublisher;
    private final ViewLoader viewLoader;
    private final ApplicationArguments arguments;
    private final UrlService urlService;
    private final ApplicationConfig applicationConfig;

    @FXML
    AnchorPane root;
    Pane contentPane;
    Pane playerPane;
    Pane loaderPane;
    Pane notificationPane;
    SectionType currentSection;

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializePanes();
        initializeNotificationPane();
        initializeSceneListeners();
        initializeSection();
        initializeOptions();
        initializeStageHeader();
        initializeSceneEvents();
        initializeTvStylesheet();
        initializeListeners();
    }

    private void initializeStageHeader() {
        BorderlessStageHolder.getWrapper()
                .ifPresent(e -> e.setHeader(28));
    }

    private void initializeSceneEvents() {
        root.setOnDragOver(this::onDragOver);
        root.setOnDragDropped(this::onDragDropped);
    }

    private void initializeSceneListeners() {
        root.setOnKeyPressed(this::onKeyPressed);
    }

    private void initializeSection() {
        if (!processApplicationArguments())
            switchSection(SectionType.CONTENT);
    }

    private void initializeOptions() {
        if (applicationConfig.isMouseDisabled()) {
            log.trace("Hiding the mouse on the main scene");
            root.getStyleClass().add(MOUSE_DISABLED_STYLE_CLASS);
            root.setCursor(Cursor.NONE);
            root.sceneProperty().addListener((observable, oldValue, newValue) -> {
                if (newValue != null) {
                    newValue.setCursor(Cursor.NONE);
                }
            });
            log.trace("Disabling mouse events on the root pane");
            root.addEventFilter(MouseEvent.ANY, this::handleRootMouseEvent);
            root.addEventFilter(KeyEvent.KEY_PRESSED, this::handleRootKeyEvent);
        }
    }

    private void initializeListeners() {
        eventPublisher.register(ShowDetailsEvent.class, event -> {
            switchSection(SectionType.CONTENT);
            return event;
        });
        eventPublisher.register(PlayVideoEvent.class, event -> {
            switchSection(SectionType.PLAYER);
            return event;
        });
        eventPublisher.register(LoadEvent.class, event -> {
            switchSection(SectionType.LOADER);
            return event;
        });
        eventPublisher.register(ClosePlayerEvent.class, event -> {
            switchSection(SectionType.CONTENT);
            return event;
        });
        eventPublisher.register(CloseLoadEvent.class, event -> {
            switchSection(SectionType.CONTENT);
            return event;
        });
    }

    private void initializeTvStylesheet() {
        if (applicationConfig.isTvMode()) {
            root.getStylesheets().add(TV_STYLESHEET);
        }
    }

    //endregion

    //region Functions

    /**
     * Initializes/loads the panes required for this controller.
     */
    private void initializePanes() {
        // load the content & notification pane on the main thread
        // this blocks Spring from completing the startup stage while these panes are being loaded
        contentPane = viewLoader.load("common/sections/content.section.fxml");
        notificationPane = viewLoader.load("common/sections/notification.section.fxml");

        anchor(contentPane);

        // load the other panes on a different thread
        new Thread(() -> {
            playerPane = viewLoader.load("common/sections/player.section.fxml");
            loaderPane = viewLoader.load("common/sections/loader.section.fxml");

            anchor(playerPane);
            anchor(loaderPane);
        }, "MainController.loader").start();
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
        var file = files.get(0);
        var title = FilenameUtils.getBaseName(file.getName());

        eventPublisher.publishEvent(new PlayVideoEvent(this, file.getAbsolutePath(), title, false));
    }

    /**
     * Invoked when a key has been pressed on the main section view.
     *
     * @param event The key event of the main section view.
     */
    protected void onKeyPressed(KeyEvent event) {
        if (PASTE_KEY_COMBINATION.match(event)) {
            event.consume();
            onContentPasted();
        }

        if (UI_ENLARGE_KEY_COMBINATION_1.match(event) || UI_ENLARGE_KEY_COMBINATION_2.match(event) || UI_ENLARGE_KEY_COMBINATION_3.match(event)) {
            event.consume();
            applicationConfig.increaseUIScale();
        } else if (UI_REDUCE_KEY_COMBINATION_1.match(event) || UI_REDUCE_KEY_COMBINATION_2.match(event)) {
            event.consume();
            applicationConfig.decreaseUIScale();
        }
    }

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

    protected void switchSection(SectionType sectionType) {
        // check if the requested section is the current section
        // if so, ignore the switch action
        if (currentSection == sectionType)
            return;

        var content = new AtomicReference<Pane>();

        this.currentSection = sectionType;

        switch (sectionType) {
            case CONTENT -> content.set(contentPane);
            case PLAYER -> content.set(playerPane);
            case LOADER -> content.set(loaderPane);
        }

        Platform.runLater(() -> {
            root.getChildren().removeIf(e -> e != notificationPane);
            root.getChildren().add(0, content.get());
        });
    }

    private void initializeNotificationPane() {
        AnchorPane.setTopAnchor(notificationPane, 55.0);
        AnchorPane.setRightAnchor(notificationPane, 20.0);

        root.getChildren().add(notificationPane);
    }

    private void anchor(Pane pane) {
        AnchorPane.setTopAnchor(pane, 0d);
        AnchorPane.setRightAnchor(pane, 0d);
        AnchorPane.setBottomAnchor(pane, 0d);
        AnchorPane.setLeftAnchor(pane, 0d);
    }

    private void handleRootMouseEvent(MouseEvent event) {
        event.consume();
        if (event.getEventType() == MouseEvent.MOUSE_CLICKED) {
            Optional.ofNullable(root.getScene())
                    .map(Scene::getFocusOwner)
                    .ifPresent(focussedNode -> {
                        var keyEvent = mapMouseEventToKeyEvent(event, focussedNode);
                        focussedNode.fireEvent(keyEvent);
                    });
        }
    }

    private void handleRootKeyEvent(KeyEvent event) {
        if (event.getCode() == KeyCode.UNDEFINED) {
            event.consume();
            Optional.ofNullable(root.getScene())
                    .map(Scene::getFocusOwner)
                    .ifPresent(focussedNode -> {
                        var keyEvent = new KeyEvent(focussedNode, focussedNode, KeyEvent.KEY_PRESSED, KeyCode.BACK_SPACE.getChar(),
                                KeyCode.BACK_SPACE.getName(), KeyCode.BACK_SPACE
                                , false, false, false, false);
                        focussedNode.fireEvent(keyEvent);
                    });
        }
    }

    private KeyEvent mapMouseEventToKeyEvent(MouseEvent event, Node targetNode) {
        return switch (event.getButton()) {
            case BACK, SECONDARY ->
                    new KeyEvent(targetNode, targetNode, KeyEvent.KEY_PRESSED, KeyCode.BACK_SPACE.getChar(), KeyCode.BACK_SPACE.getName(), KeyCode.BACK_SPACE
                            , false, false, false, false);
            case MIDDLE ->
                    new KeyEvent(targetNode, targetNode, KeyEvent.KEY_PRESSED, KeyCode.HOME.getChar(), KeyCode.HOME.getName(), KeyCode.HOME, false, false,
                            false, false);
            default -> new KeyEvent(targetNode, targetNode, KeyEvent.KEY_PRESSED, KeyCode.ENTER.getChar(), KeyCode.ENTER.getName(), KeyCode.ENTER, false, false,
                    false, false);
        };
    }

    //endregion

    private enum SectionType {
        CONTENT,
        PLAYER,
        LOADER
    }
}
