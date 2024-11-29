package com.github.yoep.popcorn.backend.adapters.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.stream.Collectors;

@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"infoHash", "magnetUri", "name", "directoryName", "totalFiles", "files"})
public class TorrentInfoWrapper extends Structure implements TorrentInfo, Closeable {
    public static class ByValue extends TorrentInfoWrapper implements Structure.ByValue {
        public ByValue() {
        }

        public ByValue(TorrentInfo info) {
            super(info);
        }
    }

    public static class ByReference extends TorrentInfoWrapper implements Structure.ByReference {
        public ByReference() {
        }

        public ByReference(TorrentInfo info) {
            super(info);
        }
    }

    public String infoHash;
    public String magnetUri;
    public String name;
    public String directoryName;
    public int totalFiles;
    public TorrentFileInfoSet.ByValue files;

    private TorrentInfo info;

    public TorrentInfoWrapper() {
    }

    public TorrentInfoWrapper(TorrentInfo info) {
        Objects.requireNonNull(info, "info cannot be null");
        this.magnetUri = info.getMagnetUri();
        this.name = info.getName();
        this.directoryName = info.getDirectoryName();
        this.totalFiles = info.getTotalFiles();
        this.files = new TorrentFileInfoSet.ByValue(info.getFiles().stream()
                .map(e -> new TorrentFileInfoWrapper.ByReference(info, e))
                .collect(Collectors.toList()));
        this.info = info;
        write();
    }

    @Override
    public List<TorrentFileInfo> getFiles() {
        return Optional.ofNullable(this.files)
                .map(TorrentFileInfoSet::getFiles)
                .map(e -> e.stream()
                        .map(file -> (TorrentFileInfo) file)
                        .collect(Collectors.toList()))
                .orElse(Collections.emptyList());
    }

    @Override
    public TorrentFileInfo getLargestFile() {
        return null;
    }

    @Override
    public Optional<TorrentFileInfo> getByFilename(String filename) {
        return getFiles().stream()
                .filter(e -> Objects.equals(e.getFilename(), filename))
                .findFirst();
    }

    @Override
    public void close() {
        setAutoSynch(false);
        Optional.ofNullable(files)
                .ifPresent(TorrentFileInfoSet::close);
    }
}
