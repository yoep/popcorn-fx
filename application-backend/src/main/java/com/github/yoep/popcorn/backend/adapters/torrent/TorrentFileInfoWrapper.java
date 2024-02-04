package com.github.yoep.popcorn.backend.adapters.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.Objects;

@Getter
@ToString
@EqualsAndHashCode(exclude = {"fileSize", "torrentInfo", "fileInfo"}, callSuper = false)
@Structure.FieldOrder({"filename", "filePath", "fileSize", "fileIndex"})
public class TorrentFileInfoWrapper extends Structure implements Closeable, TorrentFileInfo {
    public static class ByValue extends TorrentFileInfoWrapper implements Structure.ByValue {

    }

    public static class ByReference extends TorrentFileInfoWrapper implements Structure.ByReference {
        public ByReference() {
        }

        public ByReference(TorrentInfo info, TorrentFileInfo fileInfo) {
            super(info, fileInfo);
        }
    }

    public String filename;
    public String filePath;
    public long fileSize;
    public int fileIndex;

    private TorrentInfo torrentInfo;
    private TorrentFileInfo fileInfo;

    public TorrentFileInfoWrapper() {
    }

    public TorrentFileInfoWrapper(TorrentInfo torrentInfo, TorrentFileInfo fileInfo) {
        Objects.requireNonNull(fileInfo, "fileInfo cannot be null");
        this.filename = fileInfo.getFilename();
        this.filePath = fileInfo.getFilePath();
        this.fileSize = fileInfo.getFileSize();
        this.fileIndex = fileInfo.getFileIndex();
        this.torrentInfo = torrentInfo;
        this.fileInfo = fileInfo;
        write();
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }

    //region TorrentFileInfo

    @Override
    public Long getFileSize() {
        return fileSize;
    }

    @Override
    public int getFileIndex() {
        return fileIndex;
    }

    @Override
    public int compareTo(TorrentFileInfo other) {
        return getFilename().toLowerCase().compareTo(other.getFilename().toLowerCase());
    }

    //endregion
}
