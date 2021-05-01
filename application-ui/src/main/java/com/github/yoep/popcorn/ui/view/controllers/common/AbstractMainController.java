package com.github.yoep.popcorn.ui.view.controllers.common;

import com.github.spring.boot.javafx.ui.scale.ScaleAwareImpl;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.ui.events.*;
import com.github.yoep.popcorn.ui.settings.OptionsService;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.view.controllers.MainController;
import com.github.yoep.popcorn.ui.view.services.UrlService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.Cursor;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyCodeCombination;
import javafx.scene.input.KeyCombination;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import lombok.AccessLevel;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.ApplicationArguments;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.event.EventListener;
import org.springframework.core.task.TaskExecutor;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@RequiredArgsConstructor(access = AccessLevel.PROTECTED)
public abstract class AbstractMainController extends ScaleAwareImpl implements MainController {
    private static final KeyCodeCombination UI_ENLARGE_KEY_COMBINATION_1 = new KeyCodeCombination(KeyCode.ADD, KeyCombination.CONTROL_DOWN);
    private static final KeyCodeCombination UI_ENLARGE_KEY_COMBINATION_2 = new KeyCodeCombination(KeyCode.PLUS, KeyCombination.CONTROL_DOWN);
    private static final KeyCodeCombination UI_ENLARGE_KEY_COMBINATION_3 = new KeyCodeCombination(KeyCode.EQUALS, KeyCombination.CONTROL_DOWN,
            KeyCombination.SHIFT_DOWN);
    private static final KeyCodeCombination UI_REDUCE_KEY_COMBINATION_1 = new KeyCodeCombination(KeyCode.SUBTRACT, KeyCombination.CONTROL_DOWN);
    private static final KeyCodeCombination UI_REDUCE_KEY_COMBINATION_2 = new KeyCodeCombination(KeyCode.MINUS, KeyCombination.CONTROL_DOWN);

    protected final ApplicationEventPublisher eventPublisher;
    protected final ViewLoader viewLoader;
    protected final ApplicationArguments arguments;
    protected final UrlService urlService;
    protected final SettingsService settingsService;
    protected final OptionsService optionsService;
    protected final TaskExecutor taskExecutor;

    protected Pane contentPane;
    protected Pane playerPane;
    protected Pane loaderPane;
    protected Pane notificationPane;
    protected SectionType currentSection;

    @FXML
    protected AnchorPane rootPane;

    //region Methods

    @EventListener(ShowDetailsEvent.class)
    public void onShowDetails() {
        switchSection(SectionType.CONTENT);
    }

    @EventListener(PlayVideoEvent.class)
    public void onPlayVideo() {
        switchSection(SectionType.PLAYER);
    }

    @EventListener(LoadEvent.class)
    public void onLoad() {
        switchSection(SectionType.LOADER);
    }

    @EventListener(ClosePlayerEvent.class)
    public void onClosePlayer() {
        switchSection(SectionType.CONTENT);
    }

    @EventListener(CloseLoadEvent.class)
    public void onCloseLoad() {
        switchSection(SectionType.CONTENT);
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeNotificationPane();
        initializeSceneListeners();
        initializeSection();
        initializeOptions();
    }

    private void initializeSceneListeners() {
        rootPane.setOnKeyPressed(this::onKeyPressed);
    }

    private void initializeSection() {
        if (!processApplicationArguments())
            switchSection(SectionType.CONTENT);
    }

    private void initializeOptions() {
        var options = optionsService.options();

        if (options.isMouseDisabled()) {
            rootPane.setCursor(Cursor.NONE);
            rootPane.sceneProperty().addListener((observable, oldValue, newValue) -> {
                if (newValue != null) {
                    newValue.setCursor(Cursor.NONE);
                }
            });
        }
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializePanes();
    }

    /**
     * Initializes/loads the panes required for this controller.
     */
    protected void initializePanes() {
        // load the content & notification pane on the main thread
        // this blocks Spring from completing the startup stage while these panes are being loaded
        contentPane = viewLoader.load("sections/content.section.fxml");
        notificationPane = viewLoader.load("common/sections/notification.section.fxml");

        anchor(contentPane);

        // load the other panes on a different thread
        taskExecutor.execute(() -> {
            playerPane = viewLoader.load("common/sections/player.section.fxml");
            loaderPane = viewLoader.load("sections/loader.section.fxml");

            anchor(playerPane);
            anchor(loaderPane);
        });
    }

    //endregion

    //region Functions

    /**
     * Invoked when a key has been pressed on the main section view.
     *
     * @param event The key event of the main section view.
     */
    protected void onKeyPressed(KeyEvent event) {
        if (UI_ENLARGE_KEY_COMBINATION_1.match(event) || UI_ENLARGE_KEY_COMBINATION_2.match(event) || UI_ENLARGE_KEY_COMBINATION_3.match(event)) {
            event.consume();
            settingsService.increaseUIScale();
        } else if (UI_REDUCE_KEY_COMBINATION_1.match(event) || UI_REDUCE_KEY_COMBINATION_2.match(event)) {
            event.consume();
            settingsService.decreaseUIScale();
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

    private void initializeNotificationPane() {
        AnchorPane.setTopAnchor(notificationPane, 55.0);
        AnchorPane.setRightAnchor(notificationPane, 20.0);

        rootPane.getChildren().add(notificationPane);
    }

    private void anchor(Pane pane) {
        AnchorPane.setTopAnchor(pane, 0d);
        AnchorPane.setRightAnchor(pane, 0d);
        AnchorPane.setBottomAnchor(pane, 0d);
        AnchorPane.setLeftAnchor(pane, 0d);
    }

    //endregion

    private enum SectionType {
        CONTENT,
        PLAYER,
        LOADER
    }
}
