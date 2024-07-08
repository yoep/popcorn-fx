package com.github.yoep.popcorn.backend.playlists.ffi;

import com.sun.jna.Structure;
import lombok.Data;
import lombok.EqualsAndHashCode;
import lombok.ToString;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

import static java.util.Arrays.asList;

@Data
@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"items", "len"})
public class Playlist extends Structure implements Closeable {
    public static class ByReference extends Playlist implements Structure.ByReference {
        public ByReference() {
        }

        public ByReference(PlaylistItem... items) {
            super(Arrays.asList(items));
        }
    }

    public PlaylistItem.ByReference items;
    public int len;

    private List<PlaylistItem> cachedItems;

    public Playlist() {

    }

    public Playlist(List<PlaylistItem> items) {
        this.cachedItems = items;
        this.items = new PlaylistItem.ByReference();
        this.len = items.size();
        var array = (PlaylistItem.ByReference[]) this.items.toArray(this.len);

        for (int i = 0; i < this.len; i++) {
            var item = items.get(i);
            array[i].url = item.url;
            array[i].title = item.title;
            array[i].caption = item.caption;
            array[i].thumb = item.thumb;
            array[i].quality = item.quality;
            array[i].parentMedia = item.parentMedia;
            array[i].media = item.media;
            array[i].autoResumeTimestamp = item.autoResumeTimestamp;
            array[i].subtitlesEnabled = item.subtitlesEnabled;
            array[i].subtitleInfo = item.subtitleInfo;
            array[i].torrentInfo = item.torrentInfo;
            array[i].torrentFileInfo = item.torrentFileInfo;
        }

        write();
        close();
    }

    public List<PlaylistItem> getItems() {
        return Optional.ofNullable(items)
                .map(e -> (PlaylistItem[]) e.toArray(len))
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    @Override
    public void read() {
        super.read();
        cachedItems = Optional.ofNullable(items)
                .map(e -> asList((PlaylistItem[]) e.toArray(len)))
                .orElse(Collections.emptyList());
    }

    @Override
    public void close() {
        setAutoSynch(false);
        Optional.ofNullable(items)
                .map(e -> (PlaylistItem[]) e.toArray(len))
                .ifPresent(e -> Arrays.stream(e)
                        .forEach(PlaylistItem::close));
    }
}
