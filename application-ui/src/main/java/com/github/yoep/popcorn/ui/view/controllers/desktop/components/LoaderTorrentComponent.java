package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamService;
import com.github.yoep.popcorn.backend.adapters.torrent.state.SessionState;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.events.LoadUrlTorrentEvent;
import com.github.yoep.popcorn.ui.messages.TorrentMessage;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractLoaderTorrentComponent;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.event.EventListener;
import org.springframework.core.task.TaskExecutor;

import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

@Slf4j
public class LoaderTorrentComponent extends AbstractLoaderTorrentComponent {

    //region Constructors

    public LoaderTorrentComponent(LocaleText localeText, TorrentService torrentService, TorrentStreamService torrentStreamService,
                                  ApplicationEventPublisher eventPublisher, ImageService imageService, SubtitleService subtitleService,
                                  TaskExecutor taskExecutor, SettingsService settingsService) {
        super(localeText, torrentService, torrentStreamService, eventPublisher, imageService, subtitleService, taskExecutor, settingsService);
    }

    //endregion

    //region Methods

    @EventListener
    public void onLoadMediaTorrent(LoadMediaTorrentEvent event) {
        startTorrent(event);
    }

    @EventListener
    public void onLoadUrlTorrent(LoadUrlTorrentEvent event) {
        startTorrent(event);
    }

    //endregion

    //region Functions

    private void startTorrent(LoadUrlTorrentEvent activity) {
        log.debug("Starting url torrent stream for {}", activity.getTorrentFileInfo().getFilename());
        var filename = activity.getTorrentFileInfo().getFilename();
        var torrentSettings = getTorrentSettings();
        this.title = filename;


        this.torrentThread = new Thread(() -> {
            // reset the progress bar to "infinite" animation
            resetProgress();
            Platform.runLater(() -> {
                progressStatus.setVisible(false);
                statusText.setText(localeText.get(TorrentMessage.STARTING));
            });

            // check if the torrent stream is initialized, of not, wait for it to be initialized before proceeding
            if (torrentService.getSessionState() != SessionState.RUNNING)
                waitForTorrentStream();

            // update the status to retrieving subtitles and request the subtitles for the filename
            Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.RETRIEVING_SUBTITLES)));
            retrieveSubtitles(filename);

            // download the default subtitle that was determined in the last step
            downloadSubtitles(filename);

            // update the status text to "connecting"
            Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.CONNECTING)));

            log.trace("Calling torrent service stream for \"{}\"", filename);
            torrentService.create(activity.getTorrentFileInfo(), torrentSettings.getDirectory(), true).whenComplete(this::onTorrentCreated);
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
