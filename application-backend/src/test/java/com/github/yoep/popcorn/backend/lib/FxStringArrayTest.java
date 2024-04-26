package com.github.yoep.popcorn.backend.lib;

import com.github.yoep.popcorn.backend.FxLib;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class FxStringArrayTest {
    @Mock
    private FxLib fxLib;

    @BeforeEach
    void setUp() {
        FxLib.INSTANCE.set(fxLib);
    }

    @Test
    void testClose() {
        var array = new FxStringArray.ByReference();

        array.close();

        verify(fxLib).dispose_string_array(array);
    }
}