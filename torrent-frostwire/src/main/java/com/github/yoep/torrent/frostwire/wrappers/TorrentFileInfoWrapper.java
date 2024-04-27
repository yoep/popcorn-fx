package com.github.yoep.torrent.frostwire.wrappers;

import com.frostwire.jlibtorrent.FileStorage;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import lombok.EqualsAndHashCode;
import lombok.ToString;

import java.util.Objects;

@ToString(exclude = "torrentInfo")
@EqualsAndHashCode(exclude = "torrentInfo")
public class TorrentFileInfoWrapper implements TorrentFileInfo {
    private final TorrentInfo torrentInfo;
    private transient final FileStorage fileStorage;
    private final int index;

    //region Constructors

    TorrentFileInfoWrapper(TorrentInfoWrapper infoWrapper, int index) {
        Objects.requireNonNull(infoWrapper, "torrentInfo cannot be null");
        this.torrentInfo = infoWrapper;
        this.fileStorage = infoWrapper.getNative().files();
        this.index = index;
    }

    //endregion

    //region Getters

    @Override
    public String getFilename() {
        return fileStorage.fileName(index);
    }

    @Override
    public String getFilePath() {
        return fileStorage.filePath(index);
    }

    @Override
    public Long getFileSize() {
        return fileStorage.fileSize(index);
    }

    @Override
    public int getFileIndex() {
        return index;
    }

    @Override
    public TorrentInfo getTorrentInfo() {
        return torrentInfo;
    }

    @Override
    public int compareTo(TorrentFileInfo other) {
        return getFilename().toLowerCase().compareTo(other.getFilename().toLowerCase());
    }

    //endregion
}
