package com.github.yoep.video.youtube.conditions;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.boot.ApplicationArguments;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class OnYoutubeVideoEnabledTest {
    @Mock
    private ApplicationArguments applicationArguments;

    @Test
    void testMatches_whenDisableOptionIsNotPresent_shouldReturnTrue() {
        when(applicationArguments.containsOption(OnYoutubeVideoEnabled.DISABLE_OPTION)).thenReturn(false);

        var result = OnYoutubeVideoEnabled.matches(applicationArguments);

        assertTrue(result);
    }

    @Test
    void testMatches_whenBeanFactoryIsNotPresent_shouldReturnTrue() {
        var result = OnYoutubeVideoEnabled.matches(applicationArguments);

        assertTrue(result);
    }

    @Test
    void testMatches_whenDisableOptionIsPresent_shouldReturnFalse() {
        when(applicationArguments.containsOption(OnYoutubeVideoEnabled.DISABLE_OPTION)).thenReturn(true);

        var result = OnYoutubeVideoEnabled.matches(applicationArguments);

        assertFalse(result);
    }
}