package com.github.yoep.popcorn.ui.player;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.embaddable.DownloadProgress;
import com.github.yoep.popcorn.backend.adapters.player.embaddable.EmbeddablePlayer;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.AbstractTorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.TorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.events.PlayTorrentEvent;
import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.util.Optional;

/**
 * The player torrent service is responsible for handling torrent events and sending them to the active player.
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class PlayerTorrentService {
    private final PlayerManagerService playerService;

    private final TorrentListener listener = createListener();

    private Torrent torrent;
    private EmbeddablePlayer player;

    //region Methods

    @EventListener
    public void onPlayTorrent(PlayTorrentEvent event) {
        // unsubscribe the listener from the previous torrent if one is present
        unsubscribePreviousTorrent();
        // store the torrent from the event for later use
        this.torrent = event.getTorrent();
        // check if we need to subscribe back to the torrent based on the player info
        Optional.ofNullable(player)
                .ifPresent(e -> subscribeListener());
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        playerService.activePlayerProperty().addListener((observable, oldValue, newValue) -> onPlayerChanged(newValue));
    }

    //endregion

    //region Functions

    private void subscribeListener() {
        if (torrent == null)
            return;

        this.torrent.addListener(listener);
    }

    private void unsubscribePreviousTorrent() {
        if (torrent == null)
            return;

        torrent.removeListener(listener);
    }

    private void onPlayerChanged(Player player) {
        if (player.isEmbeddedPlaybackSupported()) {
            this.player = (EmbeddablePlayer) player;
            subscribeListener();
        } else {
            this.player = null;
        }
    }

    private void onTorrentProgressChanged(DownloadStatus status) {
        player.updateDownloadProgress(new SimpleDownloadProgress(status));
    }

    private TorrentListener createListener() {
        return new AbstractTorrentListener() {
            @Override
            public void onDownloadProgress(DownloadStatus status) {
                onTorrentProgressChanged(status);
            }
        };
    }

    //endregion

    @ToString
    @EqualsAndHashCode
    @RequiredArgsConstructor
    static class SimpleDownloadProgress implements DownloadProgress {
        private final DownloadStatus status;

        @Override
        public float getProgress() {
            return status.getProgress();
        }

        @Override
        public int getSeeds() {
            return status.getSeeds();
        }

        @Override
        public int getDownloadSpeed() {
            return status.getDownloadSpeed();
        }

        @Override
        public int getUploadSpeed() {
            return status.getUploadSpeed();
        }

        @Override
        public long getDownloaded() {
            return status.getDownloaded();
        }

        @Override
        public long getTotalSize() {
            return status.getTotalSize();
        }
    }
}
