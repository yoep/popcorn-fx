package com.github.yoep.player.popcorn;

import com.github.yoep.player.adapter.PlayRequest;
import com.github.yoep.player.adapter.Player;
import com.github.yoep.player.adapter.state.PlayerState;
import javafx.beans.property.ReadOnlyObjectProperty;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import org.springframework.core.io.ClassPathResource;
import org.springframework.core.io.Resource;

import java.util.Optional;

@ToString
@EqualsAndHashCode
public class PopcornPlayer implements Player {
    public static final String PLAYER_ID = "internalPlayer";
    public static final String PLAYER_NAME = "Popcorn Time";

    private static final Resource GRAPHIC_RESOURCE = new ClassPathResource("/internal-popcorn-icon.png");

    //region Player

    @Override
    public String getId() {
        return PLAYER_ID;
    }

    @Override
    public String getName() {
        return PLAYER_NAME;
    }

    @Override
    public Optional<Resource> getGraphicResource() {
        return Optional.of(GRAPHIC_RESOURCE);
    }

    @Override
    public PlayerState getState() {
        return null;
    }

    @Override
    public ReadOnlyObjectProperty<PlayerState> stateProperty() {
        return null;
    }

    @Override
    public boolean isEmbeddedPlaybackSupported() {
        return true;
    }

    @Override
    public void dispose() {

    }

    @Override
    public void play(PlayRequest url) {

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

    //endregion

    //region Functions



    //endregion
}
