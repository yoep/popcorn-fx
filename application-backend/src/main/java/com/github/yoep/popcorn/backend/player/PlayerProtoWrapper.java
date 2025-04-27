package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.google.protobuf.ByteString;
import javafx.scene.Node;
import lombok.extern.slf4j.Slf4j;

import java.io.InputStream;
import java.util.Objects;
import java.util.Optional;

@Slf4j
public record PlayerProtoWrapper(Player proto, FxChannel fxChannel) implements com.github.yoep.popcorn.backend.adapters.player.Player {
    @Override
    public String getId() {
        return proto.getId();
    }

    @Override
    public String getName() {
        return proto.getName();
    }

    @Override
    public String getDescription() {
        return proto.getDescription();
    }

    @Override
    public Optional<InputStream> getGraphicResource() {
        return Optional.ofNullable(proto.getGraphicResource())
                .map(ByteString::newInput);
    }

    @Override
    public Player.State getState() {
        return proto.getState();
    }

    @Override
    public boolean isEmbeddedPlaybackSupported() {
        return false;
    }

    @Override
    public Optional<Node> getEmbeddedPlayer() {
        return Optional.empty();
    }

    @Override
    public void dispose() {
        // no-op
    }

    @Override
    public void addListener(PlayerListener listener) {

    }

    @Override
    public void removeListener(PlayerListener listener) {

    }

    @Override
    public void play(Player.PlayRequest request) {
        // no-op
    }

    @Override
    public void resume() {
        fxChannel.send(PlayerResumeRequest.newBuilder()
                .setPlayerId(getId())
                .build());
    }

    @Override
    public void pause() {
        fxChannel.send(PlayerPauseRequest.newBuilder()
                .setPlayerId(getId())
                .build());
    }

    @Override
    public void stop() {
        fxChannel.send(PlayerStopRequest.newBuilder()
                .setPlayerId(getId())
                .build());
    }

    @Override
    public void seek(long time) {
        fxChannel.send(PlayerSeekRequest.newBuilder()
                .setPlayerId(getId())
                .setTime(time)
                .build());
    }

    @Override
    public void volume(int volume) {

    }

    @Override
    public int getVolume() {
        // TODO
        return 100;
    }

    @Override
    public boolean equals(Object o) {
        if (!(o instanceof com.github.yoep.popcorn.backend.adapters.player.Player that)) return false;

        return Objects.equals(getId(), that.getId());
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(getId());
    }
}
