package com.github.yoep.popcorn.controllers.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.CloseLoadActivity;
import com.github.yoep.popcorn.activities.LoadUrlActivity;
import com.github.yoep.popcorn.activities.ShowTorrentDetailsActivity;
import com.github.yoep.popcorn.messages.TorrentMessage;
import com.github.yoep.popcorn.torrent.TorrentService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;

import javax.annotation.PostConstruct;

@Slf4j
@Component
public class LoaderUrlComponent extends AbstractLoaderComponent {
    private final ActivityManager activityManager;
    private final TaskExecutor taskExecutor;

    private Thread torrentThread;

    //region Constructors

    public LoaderUrlComponent(LocaleText localeText, TorrentService torrentService, ActivityManager activityManager, TaskExecutor taskExecutor) {
        super(localeText, torrentService);
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
        this.torrentThread = new Thread(() -> {
            // reset the progress bar to infinite
            resetProgress();

            // check if the torrent stream is initialized, of not, wait for it to be initialized before proceeding
            if (!torrentService.isInitialized())
                waitForTorrentStream();

            // update the status text to "connecting"
            Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.CONNECTING)));

            log.debug("Resolving torrent information for \"{}\"", activity.getUrl());
            torrentService.getTorrentInfo(activity.getUrl()).ifPresentOrElse(
                    torrentInfo -> activityManager.register((ShowTorrentDetailsActivity) () -> torrentInfo),
                    this::updateProgressToErrorState);
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
