package com.github.yoep.popcorn.controllers.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.messages.TorrentMessage;
import com.github.yoep.popcorn.providers.models.Media;
import com.github.yoep.popcorn.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.subtitle.SubtitleService;
import com.github.yoep.popcorn.subtitle.models.SubtitleInfo;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.popcorn.torrent.listeners.TorrentListener;
import com.github.yoep.popcorn.torrent.models.StreamStatus;
import com.github.yoep.popcorn.torrent.models.Torrent;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import java.util.Optional;

@Slf4j
@Component
public class LoaderTorrentComponent extends AbstractLoaderComponent {
    private final ActivityManager activityManager;
    private final TaskExecutor taskExecutor;
    private final SubtitleService subtitleService;

    private String title;
    private Media media;
    private SubtitleInfo subtitle;
    private MediaTorrentInfo mediaTorrent;
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

    //region Constructors

    public LoaderTorrentComponent(ActivityManager activityManager,
                                  TaskExecutor taskExecutor,
                                  TorrentService torrentService,
                                  SubtitleService subtitleService,
                                  LocaleText localeText) {
        super(localeText, torrentService);
        this.activityManager = activityManager;
        this.taskExecutor = taskExecutor;
        this.subtitleService = subtitleService;
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

    private TorrentListener createTorrentListener() {
        return new TorrentListener() {
            @Override
            public void onLoadError(String message) {
                log.warn("Torrent loading failed: {}", message);
                updateProgressToErrorState();
            }

            @Override
            public void onStreamStarted(Torrent torrent) {
                log.debug("Torrent is starting");
                Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.STARTING)));
            }

            @Override
            public void onStreamError(Torrent torrent, Exception e) {
                log.warn("Torrent stream error: " + e.getMessage(), e);
            }

            @Override
            public void onStreamReady(Torrent torrent) {
                log.debug("Torrent is ready to be displayed");
                Platform.runLater(() -> {
                    statusText.setText(localeText.get(TorrentMessage.READY));
                    progressBar.setProgress(1);
                });
                invokePlayActivity(torrent);
            }

            @Override
            public void onStreamProgress(Torrent torrent, StreamStatus status) {
                Platform.runLater(() -> {
                    progressStatus.setVisible(true);
                    progressBar.setProgress(status.getProgress());
                    statusText.setText(localeText.get(TorrentMessage.DOWNLOADING));
                    progressPercentage.setText(String.format("%1$,.2f", status.getProgress() * 100) + "%");
                    downloadText.setText(StreamStatus.toDisplaySize(status.getDownloadSpeed()) + "/s");
                    uploadText.setText(StreamStatus.toDisplaySize(status.getUploadSpeed()) + "/s");
                    activePeersText.setText(String.valueOf(status.getSeeds()));
                });
            }

            @Override
            public void onStreamStopped() {
                log.debug("Torrent stream has stopped");
            }
        };
    }

    private void startTorrent(LoadMediaTorrentActivity activity) {
        Assert.notNull(activity.getMedia(), "LoadMediaTorrentActivity#getMedia cannot be null");
        log.debug("Starting media torrent stream for {}", activity.getMedia());

        // store the requested media locally for later use
        this.title = activity.getMedia().getTitle();
        this.media = activity.getMedia();
        this.subtitle = activity.getSubtitle().orElse(null);
        this.quality = activity.getQuality();
        this.mediaTorrent = activity.getTorrent();

        this.torrentThread = new Thread(() -> {
            // reset the progress bar to "infinite" animation
            resetProgress();
            Platform.runLater(() -> progressStatus.setVisible(false));

            // check if the torrent stream is initialized, of not, wait for it to be initialized before proceeding
            if (!torrentService.isInitialized())
                waitForTorrentStream();

            // check if the subtitles need to be downloaded
            if (subtitle != null)
                downloadSubtitles();

            // update the status text to "connecting"
            Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.CONNECTING)));

            log.trace("Calling torrent service stream for \"{}\"", mediaTorrent.getUrl());
            torrentService.startStream(mediaTorrent.getUrl());
        });

        taskExecutor.execute(this.torrentThread);
    }

    private void startTorrent(LoadUrlTorrentActivity activity) {
        log.debug("Starting url torrent stream for {}", activity.getFilename());
        this.title = activity.getFilename();

        this.torrentThread = new Thread(() -> {
            // reset the progress bar to "infinite" animation
            resetProgress();
            Platform.runLater(() -> progressStatus.setVisible(false));

            // check if the torrent stream is initialized, of not, wait for it to be initialized before proceeding
            if (!torrentService.isInitialized())
                waitForTorrentStream();

            //TODO: add subtitle download based on the filename

            // update the status text to "connecting"
            Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.CONNECTING)));

            log.trace("Calling torrent service stream for \"{}\"", activity.getFilename());
            torrentService.startStream(activity.getTorrentInfo(), activity.getFileIndex());
        });

        taskExecutor.execute(this.torrentThread);
    }

    private void invokePlayActivity(Torrent torrent) {
        // check if the media is known
        // if so, use the play media activity, else the play video activity
        if (media != null) {
            invokePlayMediaActivity(torrent);
        } else {
            invokePlayVideoActivity(torrent);
        }

        // reset this load after invoking the activity for memory cleanup
        reset();
    }

    private void invokePlayVideoActivity(Torrent torrent) {
        // store information locally
        var url = torrent.getVideoFile().getAbsolutePath();
        var title = this.title;

        activityManager.register(new PlayVideoActivity() {
            @Override
            public String getUrl() {
                return url;
            }

            @Override
            public String getTitle() {
                return title;
            }

            @Override
            public boolean isSubtitlesEnabled() {
                return true;
            }
        });
    }

    private void invokePlayMediaActivity(Torrent torrent) {
        // store information locally
        var url = torrent.getVideoFile().getAbsolutePath();
        var title = this.title;
        var media = this.media;
        var quality = this.quality;
        var subtitle = this.subtitle;

        activityManager.register(new PlayMediaActivity() {
            @Override
            public String getUrl() {
                return url;
            }

            @Override
            public String getTitle() {
                return title;
            }

            @Override
            public boolean isSubtitlesEnabled() {
                return true;
            }

            @Override
            public Media getMedia() {
                return media;
            }

            @Override
            public String getQuality() {
                return quality;
            }

            @Override
            public Optional<SubtitleInfo> getSubtitle() {
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
        this.title = null;
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
        activityManager.register(new CloseLoadActivity() {
        });

        reset();
    }

    @FXML
    private void onCancelClicked() {
        close();
    }

    //endregion
}
