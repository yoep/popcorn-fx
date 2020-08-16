package com.github.yoep.popcorn.ui.view.controllers.desktop;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.ui.events.PlayVideoEvent;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.stage.BorderlessStageHolder;
import com.github.yoep.popcorn.ui.view.controllers.MainController;
import com.github.yoep.popcorn.ui.view.controllers.common.AbstractMainController;
import com.github.yoep.popcorn.ui.view.services.UrlService;
import javafx.scene.input.*;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.collections4.CollectionUtils;
import org.apache.commons.io.FilenameUtils;
import org.apache.commons.lang3.StringUtils;
import org.springframework.boot.ApplicationArguments;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.core.task.TaskExecutor;

import java.io.File;
import java.io.IOException;
import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;

@Slf4j
public class MainDesktopController extends AbstractMainController implements MainController {
    private static final KeyCodeCombination PASTE_KEY_COMBINATION = new KeyCodeCombination(KeyCode.V, KeyCombination.CONTROL_DOWN);

    //region Constructors

    public MainDesktopController(ApplicationEventPublisher eventPublisher,
                                 ViewLoader viewLoader,
                                 TaskExecutor taskExecutor,
                                 ApplicationArguments arguments,
                                 UrlService urlService,
                                 SettingsService settingsService) {
        super(eventPublisher, viewLoader, arguments, urlService, settingsService, taskExecutor);
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        super.initialize(url, resourceBundle);
        initializeStageHeader();
        initializeSceneEvents();
    }

    private void initializeStageHeader() {
        var stageWrapper = BorderlessStageHolder.getWrapper();

        stageWrapper.setHeader(28);
    }

    private void initializeSceneEvents() {
        rootPane.setOnDragOver(this::onDragOver);
        rootPane.setOnDragDropped(this::onDragDropped);
    }

    //endregion

    //region Functions

    @Override
    protected void onKeyPressed(KeyEvent event) {
        super.onKeyPressed(event);

        if (PASTE_KEY_COMBINATION.match(event)) {
            event.consume();
            onContentPasted();
        }
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

    //endregion


}
