package com.github.yoep.popcorn.backend.adapters.player.embaddable;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import javafx.scene.Node;

/**
 * The embeddable player extends upon the normal {@link Player} for retrieving the graphical node which
 * can be used for displaying the player directly within the application.
 */
public interface EmbeddablePlayer extends Player {
    /**
     * Get the graphical {@link Node} of the player which should be included in the application UI.
     * This allows the player to be directly displayed within the application.
     *
     * @return Returns the embeddable node for the player playback.
     */
    Node getEmbeddedPlayer();
}
