package com.github.yoep.torrent.frostwire.wrappers;

import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import org.springframework.util.Assert;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

@ToString
@EqualsAndHashCode
public class TorrentInfoWrapper implements TorrentInfo {
    private transient final com.frostwire.jlibtorrent.TorrentInfo nativeInfo;

    /**
     * The cached files of this {@link TorrentInfo}.
     */
    private List<TorrentFileInfo> files;

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
        // check if a cache is already present
        // if not, load the torrent files into the cache and return the cache
        if (files == null) {
            files = internalGetFiles();
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

    @Override
    public Optional<TorrentFileInfo> getByFilename(String filename) {
        var torrentDirectory = getTorrentDirectoryName();
        var expectedFilePath = getSimplifiedFilePath(torrentDirectory + filename);

        return getFiles().stream()
                .filter(e -> getSimplifiedFilePath(e.getFilePath()).equalsIgnoreCase(expectedFilePath))
                .findFirst();
    }

    //endregion

    //region Functions

    private List<TorrentFileInfo> internalGetFiles() {
        var files = new ArrayList<TorrentFileInfo>();
        var totalFiles = getTotalFiles();

        for (int i = 0; i < totalFiles; i++) {
            files.add(createFileInfo(i));
        }

        return files;
    }

    private String getSimplifiedFilePath(String filePath) {
        return filePath.replaceAll("[\\\\/]", "").trim();
    }

    private String getTorrentDirectoryName() {
        return nativeInfo.files().name();
    }

    private TorrentFileInfo createFileInfo(int index) {
        return new TorrentFileInfoWrapper(this, index);
    }

    //endregion
}
