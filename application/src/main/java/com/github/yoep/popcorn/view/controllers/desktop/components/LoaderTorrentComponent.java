package com.github.yoep.popcorn.view.controllers.desktop.components;

import com.frostwire.jlibtorrent.TorrentInfo;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.messages.TorrentMessage;
import com.github.yoep.popcorn.subtitles.Subtitle;
import com.github.yoep.popcorn.subtitles.SubtitleService;
import com.github.yoep.popcorn.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.subtitles.models.SubtitleMatcher;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.popcorn.torrent.listeners.TorrentListener;
import com.github.yoep.popcorn.torrent.models.StreamStatus;
import com.github.yoep.popcorn.torrent.models.Torrent;
import com.github.yoep.popcorn.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.task.TaskExecutor;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import java.util.Optional;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

@Slf4j
public class LoaderTorrentComponent extends AbstractLoaderComponent {
    private final ActivityManager activityManager;
    private final TaskExecutor taskExecutor;
    private final SubtitleService subtitleService;
    private final ImageService imageService;

    private String title;
    private Media media;
    private SubtitleInfo subtitleInfo;
    private Subtitle subtitle;
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
    @FXML
    private BackgroundImageCover backgroundImage;

    //region Constructors

    public LoaderTorrentComponent(ActivityManager activityManager,
                                  TaskExecutor taskExecutor,
                                  TorrentService torrentService,
                                  SubtitleService subtitleService,
                                  LocaleText localeText,
                                  ImageService imageService) {
        super(localeText, torrentService);
        this.activityManager = activityManager;
        this.taskExecutor = taskExecutor;
        this.imageService = imageService;
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
        this.subtitleInfo = activity.getSubtitle().orElse(null);
        this.quality = activity.getQuality();
        this.mediaTorrent = activity.getTorrent();

        this.torrentThread = new Thread(() -> {
            // reset the progress bar to "infinite" animation
            resetProgress();
            Platform.runLater(() -> {
                loadBackgroundImage();
                progressStatus.setVisible(false);
                statusText.setText(localeText.get(TorrentMessage.STARTING));
            });

            // check if the torrent stream is initialized, of not, wait for it to be initialized before proceeding
            if (!torrentService.isInitialized())
                waitForTorrentStream();

            // check if the subtitle needs to be downloaded
            if (subtitleInfo != null) {
                var torrentInfo = torrentService.getTorrentInfo(mediaTorrent.getUrl());

                downloadSubtitles(torrentInfo
                        .map(this::getLargestFilename)
                        .orElse(null));
            }

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
        var url = torrentService.getStreamUrl(torrent);
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
        var url = torrentService.getStreamUrl(torrent);
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
            public Optional<Subtitle> getSubtitle() {
                return Optional.ofNullable(subtitle)
                        .or(() -> Optional.of(Subtitle.none()));
            }
        });
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

    private void downloadSubtitles(String filename) {
        // check if the given subtitle is the special "none" subtitle, if so, ignore the subtitle download
        if (subtitleInfo == null || subtitleInfo.isNone())
            return;

        // update the status text to "downloading subtitle"
        Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.DOWNLOADING_SUBTITLE)));

        // download the subtitle locally, this will cache the subtitle for later use in the video player
        try {
            subtitle = subtitleService.downloadAndParse(subtitleInfo, SubtitleMatcher.from(filename, quality)).get();
        } catch (InterruptedException | ExecutionException ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    private String getLargestFilename(TorrentInfo torrentInfo) {
        var files = torrentInfo.files();
        String filename = null;
        long largestSize = 0;

        for (int i = 0; i < torrentInfo.numFiles(); i++) {
            var name = files.fileName(i);
            var size = files.fileSize(i);

            if (size > largestSize) {
                filename = name;
                largestSize = size;
            }
        }

        return filename;
    }

    private void loadBackgroundImage() {
        backgroundImage.reset();
        imageService.loadFanart(media).whenComplete((bytes, throwable) -> {
            if (throwable == null) {
                bytes.ifPresent(e -> backgroundImage.setBackgroundImage(e));
            } else {
                log.error(throwable.getMessage(), throwable);
            }
        });
    }

    private void reset() {
        this.title = null;
        this.media = null;
        this.subtitleInfo = null;
        this.mediaTorrent = null;
        this.quality = null;
        this.torrentThread = null;
        this.backgroundImage.reset();
    }

    private void close() {
        // stop the current torrent operation thread if one is present & alive
        if (torrentThread != null && torrentThread.getState() != Thread.State.TERMINATED)
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
