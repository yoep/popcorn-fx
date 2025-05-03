package com.github.yoep.popcorn.backend.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.CleanTorrentsDirectoryRequest;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Handle;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Torrent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.TorrentEvent;
import lombok.ToString;

import java.util.Objects;
import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedQueue;

@ToString
public class FXTorrentService implements TorrentService {
    private final FxChannel fxChannel;

    private final Queue<TorrentListenerHolder> torrentListeners = new ConcurrentLinkedQueue<>();

    public FXTorrentService(FxChannel fxChannel) {
        Objects.requireNonNull(fxChannel, "fxChannel cannot be null");
        this.fxChannel = fxChannel;
        init();
    }

    @Override
    public void addListener(Handle handle, TorrentListener listener) {
        Objects.requireNonNull(handle, "handle cannot be null");
        Objects.requireNonNull(listener, "listener cannot be null");
        torrentListeners.add(new TorrentListenerHolder(handle, listener));
    }

    @Override
    public void removeListener(Handle handle, TorrentListener listener) {
        Objects.requireNonNull(handle, "handle cannot be null");
        torrentListeners.removeIf(e ->
                Objects.equals(e.handle.getHandle(), handle.getHandle())
                        && e.torrentListener == listener);
    }

    @Override
    public void cleanup() {
        fxChannel.send(CleanTorrentsDirectoryRequest.getDefaultInstance());
    }

    private void init() {
        fxChannel.subscribe(FxChannel.typeFrom(TorrentEvent.class), TorrentEvent.parser(), this::onTorrentEvent);
    }

    private void onTorrentEvent(TorrentEvent event) {
        var handle = event.getTorrentHandle();
        if (event.getEvent() == TorrentEvent.Event.STATS) {
            var stats = event.getStats().getStats();
            var downloadStatus = torrentStatsToDownloadStatus(stats);
            torrentListeners.stream()
                    .filter(e -> Objects.equals(e.handle.getHandle(), handle.getHandle()))
                    .forEach(e -> e.torrentListener.onDownloadStatus(downloadStatus));
        }
    }

    private static DownloadStatus torrentStatsToDownloadStatus(Torrent.Stats stats) {
        return new DownloadStatus() {
            @Override
            public float progress() {
                return stats.getProgress();
            }

            @Override
            public int seeds() {
                return (int) stats.getTotalPeers();
            }

            @Override
            public int peers() {
                return (int) stats.getTotalPeers();
            }

            @Override
            public int downloadSpeed() {
                return (int) stats.getDownloadUsefulRate();
            }

            @Override
            public int uploadSpeed() {
                return (int) stats.getUploadUsefulRate();
            }

            @Override
            public long downloaded() {
                return stats.getTotalDownloadedUseful();
            }

            @Override
            public long totalSize() {
                return stats.getTotalSize();
            }
        };
    }

    private record TorrentListenerHolder(Handle handle, TorrentListener torrentListener) {
    }
}
