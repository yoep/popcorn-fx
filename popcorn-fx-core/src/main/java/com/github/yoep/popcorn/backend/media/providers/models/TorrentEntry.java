package com.github.yoep.popcorn.backend.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.sun.jna.Structure;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

@Getter
@ToString
@JsonIgnoreProperties({"autoAllocate", "stringEncoding", "typeMapper", "fields", "pointer"})
@Structure.FieldOrder({"language", "quality", "len", "cap"})
public class TorrentEntry extends Structure implements Closeable {
    public static class ByReference extends TorrentEntry implements Structure.ByReference {
    }

    public String language;
    public TorrentQuality.ByReference quality;
    public int len;
    public int cap;

    private List<TorrentQuality> cache;

    public List<TorrentQuality> getQualities() {
        if (cache == null) {
            cache = Optional.ofNullable(quality)
                    .map(e -> e.toArray(len))
                    .map(e -> (TorrentQuality[]) e)
                    .map(Arrays::asList)
                    .orElse(Collections.emptyList());
        }

        return cache;
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
