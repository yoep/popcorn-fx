package com.github.yoep.popcorn.backend.adapters.torrent;

import com.sun.jna.Structure;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.*;

@Getter
@ToString
@Structure.FieldOrder({"files", "len"})
public class TorrentFileInfoSet extends Structure implements Closeable {
    public static class ByValue extends TorrentFileInfoSet implements Structure.ByValue {
        public ByValue() {
        }

        public ByValue(List<TorrentFileInfoWrapper> files) {
            super(files);
        }
    }

    public static class ByReference extends TorrentFileInfoSet implements Structure.ByReference {
        public ByReference() {
        }

        public ByReference(List<TorrentFileInfoWrapper> files) {
            super(files);
        }
    }

    public TorrentFileInfoWrapper.ByReference files;
    public int len;

    private List<TorrentFileInfoWrapper> cachedFiles;

    public TorrentFileInfoSet() {
    }

    public TorrentFileInfoSet(List<TorrentFileInfoWrapper> files) {
        Objects.requireNonNull(files, "files cannot be null");
        this.files = new TorrentFileInfoWrapper.ByReference();
        this.len = files.size();
        this.cachedFiles = files;
        var array = (TorrentFileInfoWrapper.ByReference[]) this.files.toArray(this.len);

        for (int i = 0; i < array.length; i++) {
            var file = files.get(i);
            array[i].filename = file.filename;
            array[i].fileIndex = file.fileIndex;
            array[i].filePath = file.filePath;
            array[i].fileSize = file.fileSize;
        }
        write();
    }

    public List<TorrentFileInfoWrapper> getFiles() {
        return cachedFiles;
    }

    @Override
    public void read() {
        super.read();
        cachedFiles = Optional.ofNullable(files)
                .map(e -> e.toArray(len))
                .map(e -> (TorrentFileInfoWrapper[])e)
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    @Override
    public void close() {
        setAutoSynch(false);
        Optional.ofNullable(files)
                .map(e -> e.toArray(len))
                .map(e -> (TorrentFileInfoWrapper[])e)
                .stream()
                .flatMap(Arrays::stream)
                .forEach(TorrentFileInfoWrapper::close);
    }
}
