package com.github.yoep.popcorn.controllers.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.LoadTorrentActivity;
import com.github.yoep.popcorn.activities.PlayVideoActivity;
import com.github.yoep.popcorn.activities.PlayerCloseActivity;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.TorrentInfo;
import com.github.yoep.popcorn.messages.TorrentMessage;
import com.github.yoep.popcorn.subtitle.SubtitleService;
import com.github.yoep.popcorn.subtitle.models.Subtitle;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.popcorn.torrent.listeners.TorrentListener;
import com.github.yoep.popcorn.torrent.models.StreamStatus;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FileUtils;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;

import javax.annotation.PostConstruct;
import java.util.Optional;

@Slf4j
@Component
@RequiredArgsConstructor
public class LoaderComponent {
    private final ActivityManager activityManager;
    private final TaskExecutor taskExecutor;
    private final TorrentService torrentService;
    private final SubtitleService subtitleService;
    private final LocaleText localeText;

    private Media media;
    private Subtitle subtitle;
    private TorrentInfo mediaTorrent;
    private String quality;
    private Thread torrentThread;

    @FXML
    private Label statusText;
    @FXML
    private Label progressPercentage;
    @FXML
    private Label downloadText;
    @FXML
    private Label uploadText;
    @FXML
    private Label activePeersText;
    @FXML
    private ProgressBar progressBar;
    @FXML
    private Pane progressStatus;

    @PostConstruct
    private void init() {
        activityManager.register(LoadTorrentActivity.class, this::startTorrent);
        torrentService.addListener(createTorrentListener());
    }

    private TorrentListener createTorrentListener() {
        return new TorrentListener() {
            @Override
            public void onLoadError(String message) {
                log.warn("Torrent loading failed: {}", message);
                Platform.runLater(() -> {
                    statusText.setText(localeText.get(TorrentMessage.FAILED));
                    progressBar.setProgress(1);
                    progressBar.getStyleClass().add("error");
                });
            }

            @Override
            public void onStreamStarted(com.github.yoep.popcorn.torrent.models.Torrent torrent) {
                log.debug("Torrent is starting");
                Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.STARTING)));
            }

            @Override
            public void onStreamError(com.github.yoep.popcorn.torrent.models.Torrent torrent, Exception e) {
                log.warn("Torrent stream error: " + e.getMessage(), e);
            }

            @Override
            public void onStreamReady(com.github.yoep.popcorn.torrent.models.Torrent torrent) {
                log.debug("Torrent is ready to be displayed");
                Platform.runLater(() -> {
                    statusText.setText(localeText.get(TorrentMessage.READY));
                    progressBar.setProgress(1);
                });
                invokePlayMediaActivity(torrent);
            }

            @Override
            public void onStreamProgress(com.github.yoep.popcorn.torrent.models.Torrent torrent, StreamStatus status) {
                Platform.runLater(() -> {
                    progressStatus.setVisible(true);
                    progressBar.setProgress(status.getProgress());
                    statusText.setText(localeText.get(TorrentMessage.DOWNLOADING));
                    progressPercentage.setText(String.format("%1$,.2f", status.getProgress() * 100) + "%");
                    downloadText.setText(FileUtils.byteCountToDisplaySize(status.getDownloadSpeed()) + "/s");
                    uploadText.setText(FileUtils.byteCountToDisplaySize(status.getUploadSpeed()) + "/s");
                    activePeersText.setText(String.valueOf(status.getSeeds()));
                });
            }

            @Override
            public void onStreamStopped() {
                log.debug("Torrent stream has stopped");
            }
        };
    }

    private void startTorrent(LoadTorrentActivity activity) {
        // store the requested media locally for later use
        this.media = activity.getMedia();
        this.subtitle = activity.getSubtitle().orElse(null);
        this.quality = activity.getQuality();
        this.mediaTorrent = activity.getTorrent();

        this.torrentThread = new Thread(() -> {
            // reset the progress bar to "infinite" animation
            Platform.runLater(() -> {
                progressStatus.setVisible(false);
                progressBar.setProgress(-1);
                progressBar.getStyleClass().remove("error");
            });

            // check if the torrent stream is initialized, of not, wait for it to be initialized before proceeding
            if (!torrentService.isInitialized())
                waitForTorrentStream();

            // check if the subtitles need to be downloaded
            if (subtitle != null)
                downloadSubtitles();

            // update the status text to "connecting"
            Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.CONNECTING)));

            log.trace("Starting torrent service stream for \"{}\"", mediaTorrent.getUrl());
            torrentService.startStream(mediaTorrent.getUrl());
        });

        taskExecutor.execute(this.torrentThread);
    }

    private void waitForTorrentStream() {
        Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.INITIALIZING)));

        try {
            while (!torrentService.isInitialized()) {
                Thread.sleep(100);
            }
        } catch (InterruptedException e) {
            log.error("Unexpectedly quit of wait for torrent stream monitor", e);
        }
    }

    private void invokePlayMediaActivity(com.github.yoep.popcorn.torrent.models.Torrent torrent) {
        activityManager.register(new PlayVideoActivity() {
            @Override
            public String getUrl() {
                return torrent.getVideoFile().getAbsolutePath();
            }

            @Override
            public Optional<String> getQuality() {
                return Optional.ofNullable(quality);
            }

            @Override
            public Media getMedia() {
                return media;
            }

            @Override
            public Optional<Subtitle> getSubtitle() {
                return Optional.ofNullable(subtitle);
            }
        });
    }

    private void downloadSubtitles() {
        // check if the given subtitle is the special "none" subtitle, if so, ignore the subtitle download
        if (subtitle.isNone())
            return;

        Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.DOWNLOADING_SUBTITLES)));

        subtitleService.download(subtitle);
    }

    private void reset() {
        this.media = null;
        this.subtitle = null;
        this.mediaTorrent = null;
        this.quality = null;
        this.torrentThread = null;
    }

    private void close() {
        // stop the current torrent operation thread if one is present & alive
        if (torrentThread != null && torrentThread.isAlive())
            torrentThread.interrupt();

        torrentService.stopStream();
        activityManager.register(new PlayerCloseActivity() {
            @Override
            public Media getMedia() {
                return media;
            }

            @Override
            public long getTime() {
                return PlayerCloseActivity.UNKNOWN;
            }

            @Override
            public long getLength() {
                return PlayerCloseActivity.UNKNOWN;
            }
        });

        reset();
    }

    @FXML
    private void onCancelClicked() {
        close();
    }
}
