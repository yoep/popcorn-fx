package com.github.yoep.popcorn.backend.torrent;

import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.FxChannelException;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import lombok.extern.slf4j.Slf4j;

import java.util.List;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class TorrentCollectionService {
    private final FxChannel fxChannel;

    public TorrentCollectionService(FxChannel fxChannel) {
        Objects.requireNonNull(fxChannel, "fxChannel cannot be null");
        this.fxChannel = fxChannel;
    }

    /**
     * Check if the given magnet uri has already been added to the torrent collection.
     *
     * @param magnetUri The magnet uri.
     * @return Returns true if the magnet has already been added.
     */
    public CompletableFuture<Boolean> isStored(String magnetUri) {
        Objects.requireNonNull(magnetUri, "magnetUri cannot be empty");
        return fxChannel.send(IsMagnetUriStoredRequest.newBuilder()
                        .setMagnetUri(magnetUri)
                        .build(), IsMagnetUriStoredResponse.parser())
                .thenApply(IsMagnetUriStoredResponse::getIsStored);
    }

    /**
     * Get the current stored torrent collection.
     *
     * @return Returns the stored torrents.
     */
    public CompletableFuture<List<MagnetInfo>> getStoredTorrents() {
        return fxChannel.send(GetTorrentCollectionRequest.getDefaultInstance(), GetTorrentCollectionResponse.parser())
                .thenApply(response -> {
                    if (response.getResult() == Response.Result.OK) {
                        return response.getTorrentsList();
                    } else {
                        throw new FxChannelException(String.format("Failed to retrieve torrent collection, %s", response.getError().getType()));
                    }
                });
    }

    /**
     * Add the given torrent to the torrent collection.
     *
     * @param torrent The torrent info to add.
     */
    public void addTorrent(Torrent.Info torrent) {
        Objects.requireNonNull(torrent, "torrent cannot be null");
        fxChannel.send(AddTorrentCollectionRequest.newBuilder()
                        .setType(FxChannel.typeFrom(Torrent.Info.class))
                        .setTorrentInfo(torrent)
                        .build(), AddTorrentCollectionResponse.parser())
                .thenAccept(response -> {
                    if (response.getResult() == Response.Result.OK) {
                        log.info("Added torrent to collection, {}", torrent);
                    } else {
                        log.error("Failed to add torrent to collection, {}", response.getError());
                    }
                });
    }

    /**
     * Remove the given magnet uri from the torrent collection.
     *
     * @param magnetUri The magnet uri to remove.
     */
    public void removeTorrent(String magnetUri) {
        Objects.requireNonNull(magnetUri, "magnetUri cannot be null");
        fxChannel.send(RemoveTorrentCollectionRequest.newBuilder()
                .setType(FxChannel.typeFrom(MagnetInfo.class))
                .setMagnetInfo(MagnetInfo.newBuilder()
                        .setMagnetUri(magnetUri)
                        .build())
                .build());
    }
}
