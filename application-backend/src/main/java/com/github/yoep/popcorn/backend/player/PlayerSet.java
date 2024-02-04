package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.ToString;

import java.io.Closeable;
import java.util.*;

import static java.util.Arrays.asList;

@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"players", "len"})
public class PlayerSet extends Structure implements Closeable {
    public PlayerWrapper.ByReference players;
    public int len;

    private List<Player> cachedPlayers = new ArrayList<>();

    public List<Player> getPlayers() {
        return cachedPlayers;
    }

    @Override
    public void read() {
        super.read();
        cachedPlayers = Optional.ofNullable(players)
                .map(e -> asList((Player[]) e.toArray(len)))
                .orElse(Collections.emptyList());
    }

    @Override
    public void close() {
        setAutoSynch(false);
        Optional.ofNullable(players)
                .map(e -> (PlayerWrapper[]) e.toArray(len))
                .stream()
                .flatMap(Arrays::stream)
                .forEach(PlayerWrapper::close);
    }
}
