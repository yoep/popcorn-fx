package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import com.google.protobuf.ByteString;
import lombok.extern.slf4j.Slf4j;

import java.io.IOException;
import java.io.InputStream;
import java.util.Optional;

@Slf4j
public record PlayerWrapper(Player proto) implements com.github.yoep.popcorn.backend.adapters.player.Player {
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

    public static PlayerWrapper from(com.github.yoep.popcorn.backend.adapters.player.Player player) {
        return new PlayerWrapper(Player.newBuilder()
                .setId(player.getId())
                .setName(player.getName())
                .setDescription(player.getDescription())
                .setGraphicResource(player.getGraphicResource()
                        .map(stream -> {
                            try {
                                return ByteString.readFrom(stream);
                            } catch (IOException e) {
                                log.error("Failed to read image stream", e);
                                return ByteString.empty();
                            }
                        })
                        .orElse(ByteString.empty()))
                .setState(player.getState())
                .build());
    }
}
