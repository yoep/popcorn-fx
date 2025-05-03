package com.github.yoep.player.popcorn.listeners;

import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;

public interface PlayerControlsListener {
    void onFullscreenStateChanged(boolean isFullscreenEnabled);

    void onSubtitleStateChanged(boolean isSubtitlesEnabled);

    void onPlayerStateChanged(Player.State state);

    void onPlayerTimeChanged(long time);

    void onPlayerDurationChanged(long duration);

    void onDownloadStatusChanged( DownloadStatus progress);

    void onVolumeChanged(int volume);
}
