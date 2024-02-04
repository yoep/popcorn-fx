package com.github.yoep.popcorn.backend.lib;

import com.github.yoep.popcorn.backend.FxLib;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class ByteArrayTest {
    @Mock
    private FxLib fxLib;

    @BeforeEach
    void setUp() {
        FxLib.INSTANCE.set(fxLib);
    }

    @Test
    void testClose() {
        var array = new ByteArray();

        array.close();

        verify(fxLib).dispose_byte_array(array);
    }
}