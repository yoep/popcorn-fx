package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.activities.ActivityManager;
import com.github.yoep.popcorn.ui.activities.CloseLoadActivity;
import com.github.yoep.popcorn.ui.activities.LoadUrlActivity;
import com.github.yoep.popcorn.ui.activities.ShowTorrentDetailsActivity;
import com.github.yoep.popcorn.ui.messages.TorrentMessage;
import com.github.yoep.torrent.adapter.TorrentService;
import com.github.yoep.torrent.adapter.TorrentStreamService;
import com.github.yoep.torrent.adapter.model.TorrentInfo;
import com.github.yoep.torrent.adapter.state.SessionState;
import javafx.application.Platform;
import javafx.fxml.FXML;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.task.TaskExecutor;

import javax.annotation.PostConstruct;

@Slf4j
public class LoaderUrlComponent extends AbstractLoaderComponent {
    private final ActivityManager activityManager;
    private final TaskExecutor taskExecutor;

    private Thread torrentThread;

    //region Constructors

    public LoaderUrlComponent(LocaleText localeText,
                              TorrentService torrentService,
                              TorrentStreamService torrentStreamService,
                              ActivityManager activityManager,
                              TaskExecutor taskExecutor) {
        super(localeText, torrentService, torrentStreamService);
        this.activityManager = activityManager;
        this.taskExecutor = taskExecutor;
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializeListeners();
    }

    private void initializeListeners() {
        activityManager.register(LoadUrlActivity.class, this::onLoadUrl);
    }

    //endregion

    //region Functions

    private void onLoadUrl(LoadUrlActivity activity) {
        var magnetUri = activity.getUrl();

        this.torrentThread = new Thread(() -> {
            // reset the progress bar to infinite
            resetProgress();

            // check if the torrent stream is initialized, of not, wait for it to be initialized before proceeding
            if (torrentService.getSessionState() != SessionState.RUNNING)
                waitForTorrentStream();

            // update the status text to "connecting"
            Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.CONNECTING)));

            log.debug("Resolving torrent information for \"{}\"", magnetUri);
            torrentService.getTorrentInfo(magnetUri).whenComplete((torrentInfo, throwable) -> {
                if (throwable == null) {
                    activityManager.register(new ShowTorrentDetailsActivity() {
                        @Override
                        public String getMagnetUri() {
                            return magnetUri;
                        }

                        @Override
                        public TorrentInfo getTorrentInfo() {
                            return torrentInfo;
                        }
                    });
                } else {
                    log.error("Failed to retrieve torrent, " + throwable.getMessage(), throwable);
                    updateProgressToErrorState();
                }
            });
        });

        taskExecutor.execute(this.torrentThread);
    }

    private void close() {
        if (torrentThread != null && torrentThread.isAlive())
            torrentThread.interrupt();

        activityManager.register(new CloseLoadActivity() {
        });
    }

    @FXML
    private void onCancelClicked() {
        close();
    }

    //endregion
}
