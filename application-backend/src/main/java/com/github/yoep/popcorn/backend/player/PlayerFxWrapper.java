package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import javafx.scene.Node;
import lombok.extern.slf4j.Slf4j;

import java.io.InputStream;
import java.util.Optional;

@Slf4j
public record PlayerFxWrapper(Player player, com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player proto) implements Player {
    @Override
    public String getId() {
        return player.getId();
    }

    @Override
    public String getName() {
        return player.getName();
    }

    @Override
    public String getDescription() {
        return player.getDescription();
    }

    @Override
    public Optional<InputStream> getGraphicResource() {
        return player.getGraphicResource();
    }

    @Override
    public com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player.State getState() {
        return player.getState();
    }

    @Override
    public boolean isEmbeddedPlaybackSupported() {
        return player.isEmbeddedPlaybackSupported();
    }

    @Override
    public Optional<Node> getEmbeddedPlayer() {
        return player.getEmbeddedPlayer();
    }

    @Override
    public void dispose() {
        player.dispose();
    }

    @Override
    public void addListener(PlayerListener listener) {
        player.addListener(listener);
    }

    @Override
    public void removeListener(PlayerListener listener) {
        player.removeListener(listener);
    }

    @Override
    public void play(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player.PlayRequest request) {
        player.play(request);
    }

    @Override
    public void resume() {
        player.resume();
    }

    @Override
    public void pause() {
        player.pause();
    }

    @Override
    public void stop() {
        player.stop();
    }

    @Override
    public void seek(long time) {
        player.seek(time);
    }

    @Override
    public void volume(int volume) {
        player.volume(volume);
    }

    @Override
    public int getVolume() {
        return player.getVolume();
    }
}
