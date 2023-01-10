package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.media.providers.models.ShowOverview;
import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.ToString;

import java.io.Closeable;
import java.util.List;

import static java.util.Arrays.asList;

@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"shows", "len", "cap"})
public class ShowSet extends Structure implements Closeable {
    public ShowOverview.ByReference shows;
    public int len;
    public int cap;

    public List<ShowOverview> getShows() {
        return asList((ShowOverview[]) shows.toArray(len));
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
