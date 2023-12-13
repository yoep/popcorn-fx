package com.github.yoep.popcorn.backend.playlists;

import com.sun.jna.Structure;
import lombok.Data;
import lombok.EqualsAndHashCode;
import lombok.ToString;

import java.io.Closeable;
import java.io.IOException;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

@Data
@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"items", "itemsLen"})
public class Playlist extends Structure implements Closeable {
    public static class ByValue extends Playlist implements Structure.ByValue {
    }
    
    public PlaylistItem.ByReference items;
    public int itemsLen;
    
    public List<PlaylistItem> getItems() {
        return Optional.ofNullable(items)
                .map(e -> (PlaylistItem[]) e.toArray(itemsLen))
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }
    
    @Override
    public void close() throws IOException {
        setAutoSynch(false);
    }
}
