package testing.java.com.github.yoep.popcorn.backend.player;

import javafx.scene.input.KeyCode;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class PlayerActionTest {
    @Test
    void testFromKey_whenKeyIsPlayBackStateKey_shouldReturnTheTogglePlaybackAction() {
        var result = PlayerAction.FromKey(KeyCode.SPACE);

        assertTrue(result.isPresent(), "Expected an action to have been found for the key");
        assertEquals(PlayerAction.TOGGLE_PLAYBACK_STATE, result.get());
    }

    @Test
    void testFromKey_whenKeyIsReverse_shouldReturnTheReverseAction() {
        var result = PlayerAction.FromKey(KeyCode.LEFT);

        assertTrue(result.isPresent(), "Expected an action to have been found for the key");
        assertEquals(PlayerAction.REVERSE, result.get());
    }

    @Test
    void testFromKey_whenKeyIsForward_shouldReturnTheForwardAction() {
        var result = PlayerAction.FromKey(KeyCode.RIGHT);

        assertTrue(result.isPresent(), "Expected an action to have been found for the key");
        assertEquals(PlayerAction.FORWARD, result.get());
    }
}