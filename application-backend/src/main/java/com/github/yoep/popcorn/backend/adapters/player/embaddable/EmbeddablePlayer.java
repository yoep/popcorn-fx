package com.github.yoep.popcorn.backend.adapters.player.embaddable;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import javafx.scene.Node;
import org.springframework.lang.NonNull;

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
    @NonNull
    Node getEmbeddedPlayer();

    /**
     * Set the display/layout mode of the {@link EmbeddablePlayer}.
     * This can be optionally (if supported) be used by the player to change the layout based
     * on the current user preferences.
     *
     * @param mode The display/layout mode to use.
     */
    void setLayoutMode(LayoutMode mode);
}
