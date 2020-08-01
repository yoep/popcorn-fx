package com.github.yoep.torrent.frostwire.wrappers;

import com.github.yoep.torrent.adapter.model.TorrentFileInfo;
import com.github.yoep.torrent.adapter.model.TorrentInfo;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import org.springframework.util.Assert;

import java.util.ArrayList;
import java.util.List;

@ToString
@EqualsAndHashCode
public class TorrentInfoWrapper implements TorrentInfo {
    private final com.frostwire.jlibtorrent.TorrentInfo nativeInfo;

    //region Constructors

    public TorrentInfoWrapper(com.frostwire.jlibtorrent.TorrentInfo nativeInfo) {
        Assert.notNull(nativeInfo, "nativeInfo cannot be null");
        this.nativeInfo = nativeInfo;
    }

    //endregion

    //region Getters

    /**
     * Get the underlying native torrent info from frostwire.
     *
     * @return Returns the native torrent info.
     */
    public com.frostwire.jlibtorrent.TorrentInfo getNative() {
        return nativeInfo;
    }

    @Override
    public String getName() {
        return nativeInfo.name();
    }

    @Override
    public int getTotalFiles() {
        return nativeInfo.numFiles();
    }

    @Override
    public List<TorrentFileInfo> getFiles() {
        var files = new ArrayList<TorrentFileInfo>();
        var totalFiles = getTotalFiles();

        for (int i = 0; i < totalFiles; i++) {
            files.add(createFileInfo(i));
        }

        return files;
    }

    @Override
    public TorrentFileInfo getLargestFile() {
        var largestFileInfo = (TorrentFileInfo) null;
        var largestSize = 0L;

        for (TorrentFileInfo file : getFiles()) {
            if (file.getFileSize() > largestSize) {
                largestFileInfo = file;
                largestSize = file.getFileSize();
            }
        }

        return largestFileInfo;
    }

    //endregion

    //region Functions

    private TorrentFileInfo createFileInfo(int index) {
        return new TorrentFileInfoWrapper(this, index);
    }

    //endregion
}
