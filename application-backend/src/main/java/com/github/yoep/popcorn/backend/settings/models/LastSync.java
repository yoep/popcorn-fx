package com.github.yoep.popcorn.backend.settings.models;

import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.time.Instant;
import java.time.LocalDateTime;
import java.time.ZoneOffset;

@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"time", "state"})
public class LastSync extends Structure implements Closeable {
    public static class ByReference extends LastSync implements Structure.ByReference {
    }

    public Long time;
    public TrackingSyncState state;

    public LocalDateTime getTime() {
        return LocalDateTime.ofInstant(Instant.ofEpochSecond(time), ZoneOffset.UTC);
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
