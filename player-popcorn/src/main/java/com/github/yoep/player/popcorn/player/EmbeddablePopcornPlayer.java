package com.github.yoep.player.popcorn.player;

import com.github.yoep.player.adapter.PlayRequest;
import com.github.yoep.player.adapter.embaddable.DownloadProgress;
import com.github.yoep.player.adapter.embaddable.EmbeddablePlayer;
import com.github.yoep.player.adapter.embaddable.LayoutMode;
import com.github.yoep.player.adapter.listeners.PlayerListener;
import com.github.yoep.player.adapter.state.PlayerState;
import javafx.scene.Node;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.Resource;

import java.util.Optional;

@Slf4j
@RequiredArgsConstructor
public class EmbeddablePopcornPlayer implements EmbeddablePlayer {
    private final PopcornPlayer popcornPlayer;
    private final Node embeddablePlayer;

    //region EmbeddablePlayer

    @Override
    public String getId() {
        return popcornPlayer.getId();
    }

    @Override
    public String getName() {
        return popcornPlayer.getName();
    }

    @Override
    public Optional<Resource> getGraphicResource() {
        return popcornPlayer.getGraphicResource();
    }

    @Override
    public PlayerState getState() {
        return popcornPlayer.getState();
    }

    @Override
    public boolean isEmbeddedPlaybackSupported() {
        return true;
    }

    @Override
    public void dispose() {
        popcornPlayer.dispose();
    }

    @Override
    public void addListener(PlayerListener listener) {
        popcornPlayer.addListener(listener);
    }

    @Override
    public void removeListener(PlayerListener listener) {
        popcornPlayer.removeListener(listener);
    }

    @Override
    public void play(PlayRequest request) {
        popcornPlayer.play(request);
    }

    @Override
    public void resume() {
        popcornPlayer.resume();
    }

    @Override
    public void pause() {
        popcornPlayer.pause();
    }

    @Override
    public void stop() {
        popcornPlayer.stop();
    }

    @Override
    public void seek(long time) {
        popcornPlayer.seek(time);
    }

    @Override
    public void volume(int volume) {
        popcornPlayer.volume(volume);
    }

    @Override
    public Node getEmbeddedPlayer() {
        return embeddablePlayer;
    }

    @Override
    public void setLayoutMode(LayoutMode mode) {
        //TODO: implement
    }

    @Override
    public void updateDownloadProgress(DownloadProgress downloadStatus) {

    }

    //endregion
}
