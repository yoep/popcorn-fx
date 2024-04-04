package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.FxLib;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class SubtitleTest {
    @Mock
    private FxLib fxLib;

    @BeforeEach
    void setUp() {
        FxLib.INSTANCE.set(fxLib);
    }

    @Test
    void testCloseReference() {
        var subtitle = new Subtitle.ByReference();

        subtitle.close();

        verify(fxLib).dispose_subtitle(subtitle);
    }
}