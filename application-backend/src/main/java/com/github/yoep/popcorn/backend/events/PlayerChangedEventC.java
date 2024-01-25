package com.github.yoep.popcorn.backend.events;

import com.sun.jna.Pointer;
import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.Optional;

@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"oldPlayerId", "newPlayerId", "newPlayerName"})
public class PlayerChangedEventC extends Structure implements Closeable {
    public static class ByValue extends PlayerChangedEventC implements Structure.ByValue {
    }

    public Pointer oldPlayerId;
    public String newPlayerId;
    public String newPlayerName;

    private String cachedOldPlayerId;

    public Optional<String> getOldPlayerId() {
        return Optional.ofNullable(cachedOldPlayerId);
    }

    @Override
    public void read() {
        super.read();
        cachedOldPlayerId = Optional.ofNullable(oldPlayerId)
                .map(e -> e.getString(0))
                .orElse(null);
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
