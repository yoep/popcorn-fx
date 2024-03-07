package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.sun.jna.Memory;
import com.sun.jna.Pointer;
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
@Structure.FieldOrder({"id", "name", "description", "graphicResource", "graphicResourceLen", "playerState", "embeddedPlaybackSupported"})
public class PlayerWrapper extends Structure implements Player, Closeable {
    public static class ByReference extends PlayerWrapper implements Structure.ByReference {
        public ByReference() {
        }

        public ByReference(Player player) {
            super(player);
        }

        @Override
        public void close() {
            super.close();
//            FxLib.INSTANCE.get().dispose_player(this);
        }
    }

    public String id;
    public String name;
    public String description;
    public Pointer graphicResource;
    public int graphicResourceLen;
    public PlayerState playerState;
    public byte embeddedPlaybackSupported;

    @Setter(AccessLevel.PACKAGE)
    Player player;
    @Setter(AccessLevel.PACKAGE)
    PlayerWrapperPointer playerC;
    PlayerListener playerListener;

    private byte[] cachedGraphicResource;

    public PlayerWrapper() {
    }

    public PlayerWrapper(Player player) {
        Objects.requireNonNull(player, "player cannot be null");
        this.id = player.getId();
        this.name = player.getName();
        this.description = player.getDescription();
        player.getGraphicResource()
                .ifPresent(e -> {
                    try {
                        var bytes = e.getInputStream().readAllBytes();
                        this.graphicResource = new Memory(bytes.length);
                        this.graphicResource.write(0, bytes, 0, bytes.length);
                        this.graphicResourceLen = bytes.length;
                    } catch (IOException ex) {
                        log.error("Failed to read graphic resource data, {}", ex.getMessage(), ex);
                    }
                });
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
    public void read() {
        super.read();
        this.cachedGraphicResource = Optional.ofNullable(graphicResource)
                .map(e -> e.getByteArray(0, graphicResourceLen))
                .orElse(null);
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }

    //region Player

    @Override
    public Optional<Resource> getGraphicResource() {
        return Optional.ofNullable(cachedGraphicResource)
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

        Optional.ofNullable(playerC)
                .ifPresent(e -> FxLib.INSTANCE.get().dispose_player_pointer(e));
    }

    @Override
    public void addListener(PlayerListener listener) {

    }

    @Override
    public void removeListener(PlayerListener listener) {

    }

    @Override
    public void play(PlayRequest request) {
        Optional.ofNullable(player)
                .ifPresent(e -> player.play(request));
    }

    @Override
    public void resume() {
        Optional.ofNullable(playerC)
                .ifPresent(e -> FxLib.INSTANCE.get().player_resume(e));
    }

    @Override
    public void pause() {
        Optional.ofNullable(playerC)
                .ifPresent(e -> FxLib.INSTANCE.get().player_pause(e));
    }

    @Override
    public void stop() {
        Optional.ofNullable(playerC)
                .ifPresent(e -> FxLib.INSTANCE.get().player_stop(e));
    }

    @Override
    public void seek(long time) {
        Optional.ofNullable(playerC)
                .ifPresent(e -> FxLib.INSTANCE.get().player_seek(e, time));
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
