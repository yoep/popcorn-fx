package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamService;
import com.github.yoep.popcorn.backend.adapters.torrent.state.SessionState;
import com.github.yoep.popcorn.ui.events.CloseLoadEvent;
import com.github.yoep.popcorn.ui.events.LoadUrlEvent;
import com.github.yoep.popcorn.ui.events.ShowTorrentDetailsEvent;
import com.github.yoep.popcorn.ui.messages.TorrentMessage;
import javafx.application.Platform;
import javafx.fxml.FXML;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.event.EventListener;
import org.springframework.core.task.TaskExecutor;

@Slf4j
public class LoaderUrlComponent extends AbstractLoaderComponent {
    private final ApplicationEventPublisher eventPublisher;
    private final TaskExecutor taskExecutor;

    private Thread torrentThread;

    //region Constructors

    public LoaderUrlComponent(LocaleText localeText, TorrentService torrentService, TorrentStreamService torrentStreamService,
                              ApplicationEventPublisher eventPublisher, TaskExecutor taskExecutor) {
        super(localeText, torrentService, torrentStreamService);
        this.eventPublisher = eventPublisher;
        this.taskExecutor = taskExecutor;
    }

    //endregion

    //region Methods

    @EventListener
    public void onLoadUrl(LoadUrlEvent event) {
        var magnetUri = event.getUrl();

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
                    eventPublisher.publishEvent(new ShowTorrentDetailsEvent(this, magnetUri, torrentInfo));
                } else {
                    log.error("Failed to retrieve torrent, " + throwable.getMessage(), throwable);
                    updateProgressToErrorState();
                }
            });
        });

        taskExecutor.execute(this.torrentThread);
    }

    //endregion

    //region Functions

    private void close() {
        if (torrentThread != null && torrentThread.isAlive())
            torrentThread.interrupt();

        eventPublisher.publishEvent(new CloseLoadEvent(this));
    }

    @FXML
    private void onCancelClicked() {
        close();
    }

    //endregion
}
