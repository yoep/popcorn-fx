package com.github.yoep.popcorn.backend.media.resume;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.sun.jna.Pointer;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class AutoResumeServiceTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @InjectMocks
    private AutoResumeService service;

    @Test
    void testGetResumeTimeStamp_whenResumeTimestampFound() {
        var id = "tt1111";
        var filename = "lorem.mp4";
        var pointer = mock(Pointer.class);
        var expectedResult = 120000L;
        when(fxLib.auto_resume_timestamp(instance, id, filename)).thenReturn(pointer);
        when(pointer.getLong(0)).thenReturn(expectedResult);

        var result = service.getResumeTimestamp(id, filename);

        assertEquals(Optional.of(expectedResult), result);
    }

    @Test
    void testGetResumeTimeStamp_whenResumeTimestampNotFound() {
        var filename = "ipsum-unknown.mp4";
        when(fxLib.auto_resume_timestamp(instance, null, filename)).thenReturn(null);

        var result = service.getResumeTimestamp(filename);

        assertEquals(Optional.empty(), result);
    }
}