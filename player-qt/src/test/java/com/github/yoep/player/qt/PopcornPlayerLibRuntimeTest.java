package com.github.yoep.player.qt;

import com.sun.jna.Platform;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class PopcornPlayerLibRuntimeTest {

    @Test
    void testGetLibraryName_whenInvoked_shouldReturnTheCorrectLibraryName() {
        var expectedName = Platform.isWindows() ?
                PopcornPlayerLibRuntime.LIBRARY_NAME_WINDOWS :
                PopcornPlayerLibRuntime.LIBRARY_NAME_UNIX;

        var result = PopcornPlayerLibRuntime.getLibraryName();

        assertEquals(expectedName, result);
    }
}