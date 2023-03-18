package com.github.yoep.popcorn.backend;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.mock;

class PopcornFxInstanceTest {
    @Test
    void testSetInstance() {
        var instance = mock(PopcornFx.class);

        PopcornFxInstance.INSTANCE.set(instance);
        var result = PopcornFxInstance.INSTANCE.get();

        assertEquals(instance, result);
    }
}