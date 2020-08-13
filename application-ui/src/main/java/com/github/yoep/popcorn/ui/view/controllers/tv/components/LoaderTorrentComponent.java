package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.events.ActivityManager;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractLoaderTorrentComponent;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.torrent.adapter.TorrentService;
import com.github.yoep.torrent.adapter.TorrentStreamService;
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
                                  TorrentStreamService torrentStreamService,
                                  ActivityManager activityManager,
                                  TaskExecutor taskExecutor,
                                  SubtitleService subtitleService,
                                  ImageService imageService,
                                  SettingsService settingsService) {
        super(localeText, torrentService, torrentStreamService, activityManager, imageService, subtitleService, taskExecutor, settingsService);
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        activityManager.register(LoadMediaTorrentEvent.class, this::startTorrent);
    }

    //endregion

    //region Functions

    @Override
    protected void startTorrent(LoadMediaTorrentEvent activity) {
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
