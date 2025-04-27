package com.github.yoep.popcorn.backend.adapters.player;

import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import javafx.scene.Node;

import java.io.InputStream;
import java.util.Optional;

import static com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player.State;

/**
 * The player is an embedded/non-embedded video player which supports playback of streaming videos.
 */
public interface Player {
    /**
     * Get the unique ID of the player.
     *
     * @return Returns the unique ID of the player.
     */
    String getId();

    /**
     * Get the unique name of the player.
     *
     * @return Returns the name of the player.
     */

    String getName();

    /**
     * Get the description of the player.
     * This summarizes the capabilities of the player.
     *
     * @return Returns the description of the player.
     */

    String getDescription();

    /**
     * Get the graphical icon resource for the player.
     *
     * @return Returns the graphical resource of the player if it exists, else {@link Optional#empty()}.
     */
    Optional<InputStream> getGraphicResource();

    /**
     * Get the current state of the player.
     *
     * @return Returns the current player state.
     */

    State getState();

    /**
     * Dispose the player resources.
     * This method is most of the time invoked when the application is being closed.
     */
    void dispose();

    /**
     * Register a new listener for the player.
     *
     * @param listener The listener to register.
     */
    void addListener(PlayerListener listener);

    /**
     * Remove a listener from the player.
     *
     * @param listener The listener to remove.
     */
    void removeListener(PlayerListener listener);

    /**
     * Start a new media playback within the player.
     *
     * @param request The new media playback request.
     */
    void play(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player.PlayRequest request);

    /**
     * Resume the video playback in the player.
     */
    void resume();

    /**
     * Pause the video playback in the player.
     */
    void pause();

    /**
     * Stop the video playback in the player.
     */
    void stop();

    /**
     * Seek the given time (millis) in the player.
     *
     * @param time The time to seek in milliseconds.
     */
    void seek(long time);

    /**
     * The new volume of the player.
     * The level value must be between 0 and 100.
     *
     * @param volume The volume level of the player (0 = mute, 100 = max).
     */
    void volume(int volume);

    /**
     * Retrieve the current volume level of the player.
     *
     * @return The volume level between 0 and 100.
     */
    int getVolume();

    /**
     * Check if the player supports embedded playback in the application.
     * If so, the graphical node of the play can be retrieved by using {@link #getEmbeddedPlayer()}.
     * Otherwise, the player will always use an external interface/media device for displaying the video player/playback.
     *
     * @return Returns true if the embedded playback is supported, else false.
     */
    boolean isEmbeddedPlaybackSupported();

    /**
     * Get the graphical {@link Node} of the player which should be included in the application UI.
     * This allows the player to be directly displayed within the application.
     *
     * @return Returns the embeddable node for the player playback.
     */
    Optional<Node> getEmbeddedPlayer();
}
