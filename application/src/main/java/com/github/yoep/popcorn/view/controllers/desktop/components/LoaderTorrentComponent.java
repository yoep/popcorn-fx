package com.github.yoep.popcorn.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.LoadMediaTorrentActivity;
import com.github.yoep.popcorn.activities.LoadUrlTorrentActivity;
import com.github.yoep.popcorn.messages.TorrentMessage;
import com.github.yoep.popcorn.subtitles.SubtitleService;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.popcorn.view.controllers.common.components.AbstractLoaderTorrentComponent;
import com.github.yoep.popcorn.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.task.TaskExecutor;

import javax.annotation.PostConstruct;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

@Slf4j
public class LoaderTorrentComponent extends AbstractLoaderTorrentComponent {
    //region Constructors

    public LoaderTorrentComponent(ActivityManager activityManager,
                                  TaskExecutor taskExecutor,
                                  TorrentService torrentService,
                                  SubtitleService subtitleService,
                                  LocaleText localeText,
                                  ImageService imageService) {
        super(localeText, torrentService, activityManager, imageService, subtitleService, taskExecutor);
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        activityManager.register(LoadMediaTorrentActivity.class, this::startTorrent);
        activityManager.register(LoadUrlTorrentActivity.class, this::startTorrent);
        torrentService.addListener(createTorrentListener());
    }

    //endregion

    //region Functions

    private void startTorrent(LoadUrlTorrentActivity activity) {
        log.debug("Starting url torrent stream for {}", activity.getFilename());
        this.title = activity.getFilename();

        this.torrentThread = new Thread(() -> {
            // reset the progress bar to "infinite" animation
            resetProgress();
            Platform.runLater(() -> {
                progressStatus.setVisible(false);
                statusText.setText(localeText.get(TorrentMessage.STARTING));
            });

            // check if the torrent stream is initialized, of not, wait for it to be initialized before proceeding
            if (!torrentService.isInitialized())
                waitForTorrentStream();

            // update the status to retrieving subtitles and request the subtitles for the filename
            Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.RETRIEVING_SUBTITLES)));
            retrieveSubtitles(activity.getFilename());

            // download the default subtitle that was determined in the last step
            downloadSubtitles(activity.getFilename());

            // update the status text to "connecting"
            Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.CONNECTING)));

            log.trace("Calling torrent service stream for \"{}\"", activity.getFilename());
            torrentService.startStream(activity.getTorrentInfo(), activity.getFileIndex());
        });

        taskExecutor.execute(this.torrentThread);
    }

    private void retrieveSubtitles(String filename) {
        // retrieve the subtitles for the filename and update the subtitle to the default one
        try {
            var subtitles = subtitleService
                    .retrieveSubtitles(filename)
                    .get(10, TimeUnit.SECONDS);

            subtitleInfo = subtitleService.getDefault(subtitles);
        } catch (InterruptedException | ExecutionException | TimeoutException ex) {
            log.warn(ex.getMessage(), ex);
        }
    }

    @FXML
    private void onCancelClicked() {
        close();
    }

    //endregion
}
