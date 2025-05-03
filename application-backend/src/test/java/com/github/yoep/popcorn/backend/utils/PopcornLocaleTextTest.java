package com.github.yoep.popcorn.backend.utils;

import com.github.yoep.popcorn.backend.messages.UpdateMessage;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Locale;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class PopcornLocaleTextTest {
    @Mock
    private ResourceBundleMessageSource resourceBundle;
    @InjectMocks
    private PopcornLocaleText popcornLocaleText;

    @Test
    void testGetResourceBundle() {
        var result = popcornLocaleText.getResourceBundle();

        assertEquals(resourceBundle, result);
    }

    @Test
    void testGet_Message() {
        var message = UpdateMessage.DOWNLOADING;
        var expectedResult = "Downloading";
        when(resourceBundle.getString(message.getKey())).thenReturn(expectedResult);

        var result = popcornLocaleText.get(message);

        assertEquals(expectedResult, result);
    }

    @Test
    void testGet_MessageArgs() {
        var message = UpdateMessage.NEW_VERSION;
        when(resourceBundle.getString(message.getKey())).thenReturn("Version {0}");

        var result = popcornLocaleText.get(message, "1.0.0");

        assertEquals("Version 1.0.0", result);
    }

    @Test
    void testUpdateLocale() {
        popcornLocaleText.updateLocale(Locale.CANADA);

        verify(resourceBundle).setLocale(Locale.CANADA);
    }
}