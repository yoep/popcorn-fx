package com.github.yoep.popcorn.backend.events;

import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.ToString;

import java.io.Closeable;

@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({})
public class PlayVideoEventC extends Structure implements Closeable {
    public String url;
    public String title;
    public String thumb;

    @Override
    public void close() {
        setAutoSynch(false);
    }
    
    
}
