package com.github.yoep.popcorn.backend.media.tracking;

import com.github.yoep.popcorn.backend.PopcornFx;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class TraktTrackingServiceTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @Mock
    private AuthorizationOpenCallback callback;
    @InjectMocks
    private TraktTrackingService service;

    @Test
    void testIsAuthorized() {
        when(fxLib.tracking_is_authorized(isA(PopcornFx.class))).thenReturn((byte) 1);

        var result = service.isAuthorized();

        assertTrue(result);
        verify(fxLib).tracking_is_authorized(instance);
    }

    @Test
    void testAuthorize() {
        service.authorize();

        verify(fxLib).tracking_authorize(instance);
    }

    @Test
    void testDisconnect() {
        service.disconnect();

        verify(fxLib).tracking_disconnect(instance);
    }

    @Test
    void testInit() {
        verify(fxLib).register_tracking_authorization_open(instance, callback);
    }
}