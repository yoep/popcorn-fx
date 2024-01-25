package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.lib.ByteArray;
import com.sun.jna.Structure;
import lombok.*;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.ByteArrayResource;
import org.springframework.core.io.Resource;

import java.io.Closeable;
import java.io.IOException;
import java.util.Objects;
import java.util.Optional;

@Slf4j
@Getter
@ToString
@EqualsAndHashCode(callSuper = false, exclude = "description")
@Structure.FieldOrder({"id", "name", "description", "graphicResource", "playerState", "embeddedPlaybackSupported"})
public class PlayerWrapper extends Structure implements Player, Closeable {
    public static class ByReference extends PlayerWrapper implements Structure.ByReference {
    }

    public String id;
    public String name;
    public String description;
    public ByteArray.ByReference graphicResource;
    public PlayerState playerState;
    public byte embeddedPlaybackSupported;

    @Setter(AccessLevel.PACKAGE)
    Player player;
    @Setter(AccessLevel.PACKAGE)
    PlayerWrapperPointer playerC;
    PlayerListener playerListener;

    public PlayerWrapper() {
    }

    public PlayerWrapper(Player player) {
        Objects.requireNonNull(player, "player cannot be null");
        this.id = player.getId();
        this.name = player.getName();
        this.description = player.getDescription();
        this.graphicResource = player.getGraphicResource()
                .map(e -> {
                    try {
                        var bytes = e.getInputStream().readAllBytes();
                        return new ByteArray.ByReference(bytes);
                    } catch (IOException ex) {
                        log.error("Failed to read graphic resource data, {}", ex.getMessage(), ex);
                    }

                    return null;
                })
                .orElse(null);
        this.playerState = player.getState();
        this.embeddedPlaybackSupported = (byte) (player.isEmbeddedPlaybackSupported() ? 1 : 0);
        this.player = player;
    }

    void setListener(PlayerListener playerListener) {
        if (this.player != null) {
            this.player.addListener(playerListener);
            this.playerListener = playerListener;
        }
    }

    @Override
    public void close() {
        setAutoSynch(false);
        Optional.ofNullable(graphicResource)
                .ifPresent(ByteArray::close);
    }

    //region Player

    @Override
    public Optional<Resource> getGraphicResource() {
        return Optional.ofNullable(graphicResource)
                .map(ByteArray::getBytes)
                .map(ByteArrayResource::new);
    }

    @Override
    public PlayerState getState() {
        return playerState;
    }

    @Override
    public boolean isEmbeddedPlaybackSupported() {
        return embeddedPlaybackSupported == 1;
    }

    @Override
    public void dispose() {
        if (player != null) {
            player.removeListener(playerListener);
            player.dispose();
        }
    }

    @Override
    public void addListener(PlayerListener listener) {

    }

    @Override
    public void removeListener(PlayerListener listener) {

    }

    @Override
    public void play(PlayRequest request) {

    }

    @Override
    public void resume() {

    }

    @Override
    public void pause() {

    }

    @Override
    public void stop() {

    }

    @Override
    public void seek(long time) {

    }

    @Override
    public void volume(int volume) {

    }

    @Override
    public int getVolume() {
        return 0;
    }

    //endregion
}
