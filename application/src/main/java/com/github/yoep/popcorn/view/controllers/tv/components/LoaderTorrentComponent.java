package com.github.yoep.popcorn.view.controllers.tv.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.LoadMediaTorrentActivity;
import com.github.yoep.popcorn.subtitles.SubtitleService;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.popcorn.view.controllers.common.components.AbstractLoaderTorrentComponent;
import com.github.yoep.popcorn.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.task.TaskExecutor;

import javax.annotation.PostConstruct;

@Slf4j
public class LoaderTorrentComponent extends AbstractLoaderTorrentComponent {

    @FXML
    private Button cancelButton;

    //region Constructors

    public LoaderTorrentComponent(LocaleText localeText,
                                  TorrentService torrentService,
                                  ActivityManager activityManager,
                                  TaskExecutor taskExecutor,
                                  SubtitleService subtitleService,
                                  ImageService imageService) {
        super(localeText, torrentService, activityManager, imageService, subtitleService, taskExecutor);
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        activityManager.register(LoadMediaTorrentActivity.class, this::startTorrent);
        torrentService.addListener(createTorrentListener());
    }

    //endregion

    //region Functions

    @Override
    protected void startTorrent(LoadMediaTorrentActivity activity) {
        Platform.runLater(() -> cancelButton.requestFocus());

        super.startTorrent(activity);
    }

    @FXML
    private void onCancelClicked() {
        close();
    }

    @FXML
    private void onCancelKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            close();
        }
    }

    //endregion
}
