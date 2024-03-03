package com.github.yoep.torrent.frostwire.wrappers;

import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import org.springframework.util.Assert;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

import static java.util.Arrays.asList;

@ToString
@EqualsAndHashCode
public class TorrentInfoWrapper implements TorrentInfo {
    private transient final com.frostwire.jlibtorrent.TorrentInfo nativeInfo;
    private final String torrentDirectoryName;

    /**
     * The cached files of this {@link TorrentInfo}.
     */
    private List<TorrentFileInfo> files;

    //region Constructors

    public TorrentInfoWrapper(com.frostwire.jlibtorrent.TorrentInfo nativeInfo) {
        Assert.notNull(nativeInfo, "nativeInfo cannot be null");
        this.nativeInfo = nativeInfo;
        this.torrentDirectoryName = nativeInfo.files().name();
    }

    TorrentInfoWrapper(com.frostwire.jlibtorrent.TorrentInfo nativeInfo, String torrentDirectoryName, TorrentFileInfo... files) {
        this.nativeInfo = nativeInfo;
        this.torrentDirectoryName = torrentDirectoryName;
        this.files = asList(files);
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
    public String getMagnetUri() {
        return nativeInfo.makeMagnetUri();
    }

    @Override
    public String getName() {
        return nativeInfo.name();
    }

    @Override
    public String getDirectoryName() {
        return torrentDirectoryName;
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
        var filepathWithoutTorrentDirectory = getSimplifiedFilePath(filename);

        // on the first attempt,
        // we'll try to match the torrent file based on the given filename without the torrent directory
        var torrentFileInfo = getFiles().stream()
                .filter(e -> getSimplifiedFilePath(e.getFilePath()).equalsIgnoreCase(filepathWithoutTorrentDirectory))
                .findFirst();

        if (torrentFileInfo.isPresent()) {
            return torrentFileInfo;
        }

        var filepathWithTorrentDirectory = getSimplifiedFilePath(torrentDirectory + filename);

        // on the second attempt,
        // we'll try to match the torrent file based on the given filename with the torrent directory included
        return getFiles().stream()
                .filter(e -> getSimplifiedFilePath(e.getFilePath()).equalsIgnoreCase(filepathWithTorrentDirectory))
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
        return this.torrentDirectoryName;
    }

    private TorrentFileInfo createFileInfo(int index) {
        return new TorrentFileInfoWrapper(this, index);
    }

    //endregion
}
