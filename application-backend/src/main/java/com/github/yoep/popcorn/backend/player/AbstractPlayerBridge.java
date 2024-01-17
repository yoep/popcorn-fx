package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;

@Getter
@ToString
@EqualsAndHashCode(callSuper = false, exclude = "description")
@Structure.FieldOrder({"id", "name", "description"})
public abstract class AbstractPlayerBridge extends Structure implements Player, Closeable {
    public String id;
    public String name;
    public String description;

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
